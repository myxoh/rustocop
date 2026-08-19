use super::catalog_cop::{custom, replace, report};
use super::*;
use std::collections::HashSet;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        custom(
            "Lint/DuplicateRegexpCharacterClassElement",
            duplicate_character_class,
        ),
        replace(
            "Lint/RedundantRegexpQuantifiers",
            "{1}",
            "",
            "Use a single regexp atom instead of a `{1}` quantifier.",
        ),
        report(
            "Lint/UnescapedBracketInRegexp",
            "/[/",
            "Regular expression has an unescaped open bracket.",
        ),
        report(
            "Lint/AmbiguousRegexpLiteral",
            "puts /",
            "Ambiguous regexp literal. Parenthesize the method arguments.",
        ),
        custom(
            "Style/RedundantRegexpCharacterClass",
            redundant_character_class,
        ),
        report(
            "Lint/ArrayLiteralInRegexp",
            "Regexp.new([",
            "Passing an array to `Regexp.new` is invalid.",
        ),
        replace(
            "Style/RedundantRegexpEscape",
            "\\:",
            ":",
            "Redundant escape inside regexp literal.",
        ),
        custom("Style/RegexpLiteral", regexp_literal),
        replace(
            "Style/RedundantRegexpArgument",
            ".match(/foo/, 0)",
            ".match(/foo/)",
            "Remove the redundant regexp match position argument.",
        ),
        custom("Lint/OutOfRangeRegexpRef", out_of_range_ref),
        Box::new(SelectByRegexp),
    ]
}

struct SelectByRegexp;

enum BlockParameter {
    Named(Vec<u8>),
    Numbered,
    It,
}

struct RegexpSelection<'pr> {
    pattern: Node<'pr>,
    negated: bool,
}

impl Cop for SelectByRegexp {
    fn name(&self) -> &'static str {
        "Style/SelectByRegexp"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        let original = call.name().as_slice();
        if !matches!(original, b"select" | b"filter" | b"find_all" | b"reject") {
            return;
        }
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        if original == b"filter" && !cop_context.target_ruby_version().at_least(2, 6) {
            return;
        }
        let Some(block) = call.block().and_then(|block| block.as_block_node()) else {
            return;
        };
        if call.receiver().as_ref().is_some_and(hash_like_receiver) {
            return;
        }
        let Some(parameter) = select_block_parameter(&block) else {
            return;
        };
        let Some(body) = block.body().and_then(single_expression) else {
            return;
        };
        let Some(selection) = regexp_selection(body, &parameter) else {
            return;
        };
        let selecting = matches!(original, b"select" | b"filter" | b"find_all");
        let replacement = if selecting == selection.negated {
            "grep_v"
        } else {
            "grep"
        };
        if replacement == "grep_v" && !cop_context.target_ruby_version().at_least(2, 3) {
            return;
        }
        let original = String::from_utf8_lossy(original);
        let Some(selector) = call.message_loc() else {
            return;
        };
        cop_context.replace(
            format!("Prefer `{replacement}` to `{original}` with a regexp match."),
            call.location(),
            selector.start_offset()..block.location().end_offset(),
            format!(
                "{replacement}({})",
                source_at(source, &selection.pattern.location())
            ),
        );
    }
}

fn select_block_parameter(block: &ruby_prism::BlockNode<'_>) -> Option<BlockParameter> {
    let parameters = block.parameters()?;
    if let Some(numbered) = parameters.as_numbered_parameters_node() {
        return (numbered.maximum() == 1).then_some(BlockParameter::Numbered);
    }
    if parameters.as_it_parameters_node().is_some() {
        return Some(BlockParameter::It);
    }
    let block_parameters = parameters.as_block_parameters_node()?;
    let parameters = block_parameters.parameters()?;
    if parameters.requireds().len() != 1
        || !parameters.optionals().is_empty()
        || parameters.rest().is_some()
        || !parameters.posts().is_empty()
        || !parameters.keywords().is_empty()
        || parameters.keyword_rest().is_some()
        || parameters.block().is_some()
    {
        return None;
    }
    let parameter = parameters
        .requireds()
        .first()?
        .as_required_parameter_node()?;
    Some(BlockParameter::Named(parameter.name().as_slice().to_vec()))
}

fn regexp_selection<'pr>(
    body: Node<'pr>,
    parameter: &BlockParameter,
) -> Option<RegexpSelection<'pr>> {
    let (body, explicitly_negated) = unwrap_regexp_negation(body)?;
    let call = body
        .as_match_write_node()
        .map(|write| write.call())
        .or_else(|| body.as_call_node())?;
    let method = call.name().as_slice();
    if !matches!(method, b"match?" | b"=~" | b"!~") {
        return None;
    }
    if method == b"match?" && call.receiver().is_none() {
        return None;
    }
    let receiver = call.receiver()?;
    let arguments = call.arguments()?;
    let mut arguments = arguments.arguments().iter();
    let argument = arguments.next()?;
    if arguments.next().is_some() {
        return None;
    }
    let pattern = if is_select_parameter(&receiver, parameter) {
        argument
    } else if is_select_parameter(&argument, parameter) {
        receiver
    } else {
        return None;
    };
    Some(RegexpSelection {
        pattern,
        negated: explicitly_negated || method == b"!~",
    })
}

fn unwrap_regexp_negation(mut node: Node<'_>) -> Option<(Node<'_>, bool)> {
    let mut negated = false;
    if let Some(call) = node.as_call_node() {
        if call.name().as_slice() == b"!" {
            if call.arguments().is_some_and(|arguments| !arguments.arguments().is_empty()) {
                return None;
            }
            node = call.receiver()?;
            negated = true;
        }
    }
    if let Some(parentheses) = node.as_parentheses_node() {
        node = parentheses.body().and_then(single_expression)?;
    }
    Some((node, negated))
}

fn is_select_parameter(node: &Node<'_>, parameter: &BlockParameter) -> bool {
    match parameter {
        BlockParameter::Named(name) => node
            .as_local_variable_read_node()
            .is_some_and(|read| read.name().as_slice() == name),
        BlockParameter::Numbered => node
            .as_local_variable_read_node()
            .is_some_and(|read| read.name().as_slice() == b"_1"),
        BlockParameter::It => node.as_it_local_variable_read_node().is_some(),
    }
}

fn hash_like_receiver(node: &Node<'_>) -> bool {
    if node.as_hash_node().is_some() || node_is_root_constant(node, b"ENV") {
        return true;
    }
    node.as_call_node().is_some_and(|call| {
        matches!(call.name().as_slice(), b"to_h" | b"to_hash")
            || matches!(call.name().as_slice(), b"new" | b"[]")
                && call
                    .receiver()
                    .as_ref()
                    .is_some_and(|receiver| node_is_root_constant(receiver, b"Hash"))
    })
}

fn regexp_ranges(source: &str) -> impl Iterator<Item = (usize, usize)> + '_ {
    source.match_indices('/').filter_map(|(start, _)| {
        source[start + 1..]
            .find('/')
            .map(|relative| (start, start + 1 + relative))
    })
}

fn duplicate_character_class(context: &mut CopContext<'_, '_>) {
    let source = context.source().to_string();
    for (regexp_start, regexp_end) in regexp_ranges(&source) {
        let regexp = &source[regexp_start + 1..regexp_end];
        if regexp.contains('\\') || regexp.contains("&&") || regexp.contains("#{") {
            continue;
        }
        let Some(open) = regexp.find('[') else {
            continue;
        };
        let Some(close) = regexp[open + 1..].find(']').map(|at| open + 1 + at) else {
            continue;
        };
        let mut seen = HashSet::new();
        for (relative, character) in regexp[open + 1..close].char_indices() {
            if !seen.insert(character) {
                let start = regexp_start + 1 + open + 1 + relative;
                context.remove(
                    "Duplicate element inside regexp character class.",
                    start..start + character.len_utf8(),
                    start..start + character.len_utf8(),
                );
            }
        }
    }
}

fn redundant_character_class(context: &mut CopContext<'_, '_>) {
    let source = context.source().to_string();
    for (regexp_start, regexp_end) in regexp_ranges(&source) {
        let regexp = &source[regexp_start + 1..regexp_end];
        if regexp.starts_with('[')
            && regexp.ends_with(']')
            && regexp[1..regexp.len() - 1].chars().count() == 1
            && !matches!(&regexp[1..regexp.len() - 1], "+" | " " | "-" | "#")
        {
            context.replace(
                "Redundant single-element regexp character class.",
                regexp_start + 1..regexp_end,
                regexp_start + 1..regexp_end,
                &regexp[1..regexp.len() - 1],
            );
        }
    }
}

fn regexp_literal(context: &mut CopContext<'_, '_>) {
    let source = context.source().to_string();
    for quote in ['\'', '"'] {
        let needle = format!("Regexp.new({quote}");
        let mut search = 0;
        while let Some(relative) = source[search..].find(&needle) {
            let start = search + relative;
            let body_start = start + needle.len();
            let Some(close) = source[body_start..].find(quote).map(|at| body_start + at) else {
                break;
            };
            if source.as_bytes().get(close + 1) == Some(&b')') {
                context.replace(
                    "Use a regexp literal instead of `Regexp.new`.",
                    start..close + 2,
                    start..close + 2,
                    format!("/{}/", &source[body_start..close]),
                );
            }
            search = close + 1;
        }
    }
}

fn out_of_range_ref(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        for (at, _) in line.match_indices('$') {
            let digits = line[at + 1..]
                .bytes()
                .take_while(u8::is_ascii_digit)
                .count();
            if digits > 0
                && line[at + 1..at + 1 + digits]
                    .parse::<usize>()
                    .is_ok_and(|value| value > 9)
            {
                context.report(
                    "Back reference is out of range.",
                    offset + at..offset + at + digits + 1,
                );
            }
        }
    }
}
