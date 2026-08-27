use super::*;

mod branch_rules;
use branch_rules::*;

define_cops! {
    HashLikeCase => "Style/HashLikeCase" => compatibility_prism_node(as_case_node, hash_like_case),
    EmptyCaseCondition => "Style/EmptyCaseCondition" => compatibility_prism_node(as_case_node, empty_case_condition),
    MixedCaseRange => "Lint/MixedCaseRange" => compatibility_prism_any_node(mixed_case_range),
    ExponentialNotation => "Style/ExponentialNotation" => compatibility_prism_node(as_float_node, exponential_notation),
}

const MIXED_CASE_RANGE_MESSAGE: &str = "Ranges from upper to lower case ASCII letters may include unintended characters. Instead of `A-z` (which also includes several symbols) specify each range individually: `A-Za-z` and individually specify any symbols.";

fn empty_case_condition(node: &ruby_prism::CaseNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.predicate().is_some()
        || node.conditions().is_empty()
        || context.parent().is_some_and(|parent| {
            parent.as_return_node().is_some()
                || parent.as_break_node().is_some()
                || parent.as_next_node().is_some()
                || parent.as_call_node().is_some()
        })
    {
        return;
    }

    let mut branches = Vec::new();
    for condition in node.conditions().iter() {
        let Some(branch) = condition.as_when_node() else {
            return;
        };
        if branch
            .statements()
            .is_some_and(|statements| contains_return(&statements.as_node()))
        {
            return;
        }
        branches.push(branch);
    }
    if node
        .else_clause()
        .and_then(|branch| branch.statements())
        .is_some_and(|statements| contains_return(&statements.as_node()))
    {
        return;
    }

    let keyword = node.case_keyword_loc();
    let first_when = branches[0].keyword_loc();
    let file = context.source_file();
    let mut edits = vec![(
        keyword.start_offset()..first_when.end_offset(),
        "if".to_string(),
    )];
    let comments = ruby_prism::parse(context.source().as_bytes())
        .comments()
        .filter(|comment| {
            let start = comment.location().start_offset();
            keyword.start_offset() <= start && start < first_when.start_offset()
        })
        .map(|comment| {
            format!(
                "{}{}\n",
                " ".repeat(file.column(keyword.start_offset())),
                file.at(&comment.location()).trim_end()
            )
        })
        .collect::<String>();
    if !comments.is_empty() {
        edits.push((
            file.line_start(keyword.start_offset())..file.line_start(keyword.start_offset()),
            comments,
        ));
    }

    for (index, branch) in branches.iter().enumerate() {
        let branch_keyword = branch.keyword_loc();
        if index > 0 {
            edits.push((
                branch_keyword.start_offset()..branch_keyword.end_offset(),
                "elsif".to_string(),
            ));
        }
        let conditions = branch.conditions().iter().collect::<Vec<_>>();
        for pair in conditions.windows(2) {
            edits.push((
                pair[0].location().end_offset()..pair[1].location().start_offset(),
                " || ".to_string(),
            ));
        }
        let case_begins_expression = context.source()
            [file.line_start(keyword.start_offset())..keyword.start_offset()]
            .trim()
            .is_empty();
        if !case_begins_expression {
            if let (Some(last), Some(then_keyword)) =
                (conditions.last(), branch.then_keyword_loc())
            {
                edits.push((
                    last.location().end_offset()..then_keyword.end_offset(),
                    "\n".to_string(),
                ));
            }
        }
    }
    context.replace_many(
        "Do not use empty `case` condition, instead use an `if` expression.",
        &keyword,
        edits,
    );
}

fn contains_return(node: &Node<'_>) -> bool {
    struct ReturnFinder(bool);

    impl<'pr> Visit<'pr> for ReturnFinder {
        fn visit_return_node(&mut self, _node: &ruby_prism::ReturnNode<'pr>) {
            self.0 = true;
        }
    }

    let mut finder = ReturnFinder(false);
    finder.visit(node);
    finder.0
}

fn mixed_case_range(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(range) = node.as_range_node() {
        let (Some(left), Some(right)) = (range.left(), range.right()) else {
            return;
        };
        let (Some(left), Some(right)) = (left.as_string_node(), right.as_string_node()) else {
            return;
        };
        if left.unescaped().len() == 1
            && right.unescaped().len() == 1
            && left.unescaped()[0].is_ascii_uppercase()
            && right.unescaped()[0].is_ascii_lowercase()
        {
            context.report(MIXED_CASE_RANGE_MESSAGE, range.location());
        }
        return;
    }
    if node.as_regular_expression_node().is_none()
        && node.as_interpolated_regular_expression_node().is_none()
    {
        return;
    }
    let location = node.location();
    let source = context.source_file().node(node);
    for relative in mixed_regex_ranges(source) {
        let start = location.start_offset() + relative;
        let bytes = &source.as_bytes()[relative..relative + 3];
        let replacement = expand_mixed_range(bytes[0], bytes[2]);
        context.replace(
            MIXED_CASE_RANGE_MESSAGE,
            start..start + 3,
            start..start + 3,
            replacement,
        );
    }
}

fn mixed_regex_ranges(source: &str) -> Vec<usize> {
    let bytes = source.as_bytes();
    let mut ranges = Vec::new();
    let mut class_depth = 0usize;
    let mut interpolation_depth = 0usize;
    let mut index = 0usize;
    while index < bytes.len() {
        if interpolation_depth > 0 {
            match bytes[index] {
                b'{' => interpolation_depth += 1,
                b'}' => interpolation_depth -= 1,
                b'\\' => index += 1,
                _ => {}
            }
            index += 1;
            continue;
        }
        if bytes[index..].starts_with(b"#{") {
            interpolation_depth = 1;
            index += 2;
            continue;
        }
        match bytes[index] {
            b'\\' => {
                index += 2;
                continue;
            }
            b'[' => class_depth += 1,
            b']' => class_depth = class_depth.saturating_sub(1),
            byte if class_depth == 1
                && byte.is_ascii_uppercase()
                && bytes.get(index + 1) == Some(&b'-')
                && bytes.get(index + 2).is_some_and(u8::is_ascii_lowercase) =>
            {
                ranges.push(index);
                index += 2;
            }
            _ => {}
        }
        index += 1;
    }
    ranges
}

fn expand_mixed_range(start: u8, end: u8) -> String {
    let mut expansion = String::new();
    expansion.push(start as char);
    if start != b'Z' {
        expansion.push_str("-Z");
    }
    if end != b'a' {
        expansion.push_str("a-");
    }
    expansion.push(end as char);
    expansion
}

fn exponential_notation(node: &ruby_prism::FloatNode<'_>, context: &mut CopContext<'_, '_>) {
    let location = node.location();
    let source = context
        .source_file()
        .slice(location.start_offset()..location.end_offset())
        .unwrap_or_default();
    let Some((mantissa_source, exponent_source)) = source.split_once(['e', 'E']) else {
        return;
    };
    let Ok(mantissa) = mantissa_source.replace('_', "").parse::<f64>() else {
        return;
    };
    let Ok(exponent) = exponent_source.replace('_', "").parse::<i32>() else {
        return;
    };
    let magnitude = mantissa.abs();
    let (offense, message) = match context.policy().enforced_style("scientific") {
        "scientific" if !(1.0..10.0).contains(&magnitude) => {
            (true, "Use a mantissa >= 1 and < 10.")
        }
        "engineering" if exponent % 3 != 0 || !(0.1..1000.0).contains(&magnitude) => (
            true,
            "Use an exponent divisible by 3 and a mantissa >= 0.1 and < 1000.",
        ),
        "integral" if mantissa_source.contains('.') || mantissa_source.ends_with('0') => {
            (true, "Use an integer as mantissa, without trailing zero.")
        }
        _ => (false, ""),
    };
    if !offense {
        return;
    }
    let start = location.start_offset();
    let start = if start > 0 && context.source().as_bytes().get(start - 1) == Some(&b'-') {
        start - 1
    } else {
        start
    };
    context.report(message, start..location.end_offset());
}

fn hash_like_case(node: &ruby_prism::CaseNode<'_>, context: &mut CopContext<'_, '_>) {
    use crate::rubocop::cop::mixin::policies::{
        meets_min_branches_count, min_branches_count,
    };

    if node.else_clause().is_some() || node.predicate().is_none() {
        return;
    }
    let branches = node.conditions().iter().collect::<Vec<_>>();
    let configured = context
        .config_value("MinBranchesCount")
        .and_then(|value| value.parse::<i64>().ok());
    let Ok(minimum) = min_branches_count(configured) else {
        return;
    };
    if !meets_min_branches_count(branches.len(), minimum) {
        return;
    }
    let mut condition_kind = None;
    let mut body_kind = None;
    for branch in branches {
        let Some(branch) = branch.as_when_node() else {
            return;
        };
        let conditions = branch.conditions();
        if conditions.len() != 1 {
            return;
        }
        let Some(condition) = conditions.first() else {
            return;
        };
        let Some(current_condition_kind) = string_or_symbol_kind(&condition) else {
            return;
        };
        let Some(statements) = branch.statements() else {
            return;
        };
        let body = statements.body();
        if body.len() != 1 {
            return;
        }
        let Some(value) = body.first() else {
            return;
        };
        let Some(current_body_kind) = literal_kind(&value) else {
            return;
        };
        if condition_kind.is_some_and(|kind| kind != current_condition_kind)
            || body_kind.is_some_and(|kind| kind != current_body_kind)
        {
            return;
        }
        condition_kind = Some(current_condition_kind);
        body_kind = Some(current_body_kind);
    }
    context.report(
        "Consider replacing `case-when` with a hash lookup.",
        node.location(),
    );
}

fn string_or_symbol_kind(node: &Node<'_>) -> Option<u8> {
    if node.as_string_node().is_some() {
        Some(1)
    } else if node.as_symbol_node().is_some() {
        Some(2)
    } else {
        None
    }
}

fn literal_kind(node: &Node<'_>) -> Option<u8> {
    string_or_symbol_kind(node)
        .or_else(|| {
            node.as_interpolated_string_node()
                .filter(|string| {
                    string
                        .parts()
                        .iter()
                        .all(|part| part.as_string_node().is_some())
                })
                .map(|_| 1)
        })
        .or_else(|| node.as_integer_node().map(|_| 3))
        .or_else(|| node.as_float_node().map(|_| 4))
        .or_else(|| node.as_true_node().map(|_| 5))
        .or_else(|| node.as_false_node().map(|_| 6))
        .or_else(|| node.as_nil_node().map(|_| 7))
}

fn empty_in_pattern(node: &ruby_prism::InNode<'_>, context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(2, 7) || node.statements().is_some() {
        return;
    }
    let offense = node.in_loc().start_offset()..node.pattern().location().end_offset();
    if context.config_bool("AllowComments", true)
        && branch_trailing_source(context, offense.end).contains('#')
    {
        return;
    }
    context.report("Avoid `in` branches without a body.", offense);
}
