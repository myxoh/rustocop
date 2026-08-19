use std::collections::HashSet;

use super::*;

define_cops! {
    EmptyInPattern => "Lint/EmptyInPattern" => node(as_in_node, empty_in_pattern),
    DuplicateCaseCondition => "Lint/DuplicateCaseCondition" => node(as_case_node, duplicate_case_condition),
    EmptyCaseCondition => "Style/EmptyCaseCondition" => node(as_case_node, empty_case_condition),
    MixedCaseRange => "Lint/MixedCaseRange" => any_node(mixed_case_range),
    UnifiedInteger => "Lint/UnifiedInteger" => any_node(unified_integer),
    ExponentialNotation => "Style/ExponentialNotation" => node(as_float_node, exponential_notation),
    HashLikeCase => "Style/HashLikeCase" => node(as_case_node, hash_like_case),
    FileNull => "Style/FileNull" => node(as_string_node, file_null),
}

const MIXED_CASE_RANGE_MESSAGE: &str = "Ranges from upper to lower case ASCII letters may include unintended characters. Instead of `A-z` (which also includes several symbols) specify each range individually: `A-Za-z` and individually specify any symbols.";

fn duplicate_case_condition(node: &ruby_prism::CaseNode<'_>, context: &mut CopContext<'_, '_>) {
    let mut seen = HashSet::new();
    for condition in node.conditions().iter() {
        let Some(branch) = condition.as_when_node() else {
            continue;
        };
        for value in branch.conditions().iter() {
            let source = context.source_file().node(&value);
            if !seen.insert(source) {
                context.report("Duplicate `when` condition detected.", value.location());
            }
        }
    }
}

fn empty_case_condition(node: &ruby_prism::CaseNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.predicate().is_some() || node.conditions().is_empty() {
        return;
    }
    let keyword = node.case_keyword_loc();
    let file = context.source_file();
    if !context.source()[file.line_start(keyword.start_offset())..keyword.start_offset()]
        .trim()
        .is_empty()
    {
        return;
    }

    let line_end = file.line_end(keyword.end_offset());
    let tail = &context.source()[keyword.end_offset()..line_end];
    let case_edit = if let Some(comment) = tail.find('#') {
        keyword.start_offset()..keyword.end_offset() + comment
    } else {
        file.line_range(keyword.start_offset())
    };
    let mut edits = vec![(case_edit, String::new())];
    for (index, condition) in node.conditions().iter().enumerate() {
        let Some(branch) = condition.as_when_node() else {
            return;
        };
        let branch_keyword = branch.keyword_loc();
        edits.push((
            branch_keyword.start_offset()..branch_keyword.end_offset(),
            if index == 0 { "if" } else { "elsif" }.to_string(),
        ));
        let conditions = branch.conditions().iter().collect::<Vec<_>>();
        for pair in conditions.windows(2) {
            edits.push((
                pair[0].location().end_offset()..pair[1].location().start_offset(),
                " || ".to_string(),
            ));
        }
    }
    context.replace_many(
        "Do not use empty `case` condition, instead use an `if` expression.",
        &keyword,
        edits,
    );
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

fn unified_integer(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let class = [b"Fixnum".as_slice(), b"Bignum".as_slice()]
        .into_iter()
        .find(|class| node_is_root_constant(node, class));
    let Some(class) = class else {
        return;
    };
    let class = String::from_utf8_lossy(class);
    let message = format!("Use `Integer` instead of `{class}`.");
    if !context.target_ruby_version().at_least(2, 4) {
        context.report(message, node.location());
        return;
    }
    let edit = node
        .as_constant_path_node()
        .map(|constant| constant.name_loc())
        .unwrap_or_else(|| node.location());
    context.replace(message, node.location(), edit, "Integer");
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
    if node.else_clause().is_some() || node.predicate().is_none() {
        return;
    }
    let branches = node.conditions().iter().collect::<Vec<_>>();
    if branches.len() < context.config_usize("MinBranchesCount", 3) {
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

fn branch_trailing_source<'source>(
    context: &'source CopContext<'_, '_>,
    start: usize,
) -> &'source str {
    let source = context.source();
    let tail = source.get(start..).unwrap_or_default();
    let mut length = 0;
    for line in tail.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if length > 0
            && (trimmed.starts_with("in ")
                || trimmed.starts_with("else")
                || trimmed.starts_with("end"))
        {
            break;
        }
        length += line.len();
        if !trimmed.is_empty() && !trimmed.starts_with('#') && length > line.len() {
            break;
        }
    }
    &tail[..length]
}

fn file_null(node: &ruby_prism::StringNode<'_>, context: &mut CopContext<'_, '_>) {
    if context
        .parent()
        .is_some_and(|parent| parent.as_array_node().is_some() || parent.as_assoc_node().is_some())
    {
        return;
    }
    let value = String::from_utf8_lossy(node.unescaped());
    let lower = value.to_ascii_lowercase();
    let null = lower == "/dev/null" || lower == "nul:" || lower == "nul";
    if !null || lower == "nul" && !context.source().to_ascii_lowercase().contains("/dev/null") {
        return;
    }
    context.replace(
        format!("Use `File::NULL` instead of `{value}`."),
        node.location(),
        node.location(),
        "File::NULL",
    );
}
