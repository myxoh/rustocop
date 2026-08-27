use super::*;

define_cops! {
    UnexpectedBlockArity => "Lint/UnexpectedBlockArity" => compatibility_prism_call(unexpected_block_arity),
}

fn next_without_accumulator(node: &ruby_prism::NextNode<'_>, context: &mut CopContext<'_, '_>) {
    if node
        .arguments()
        .is_some_and(|arguments| !arguments.arguments().is_empty())
    {
        return;
    }
    let Some(block_index) = context
        .ancestors()
        .iter()
        .rposition(|ancestor| ancestor.as_block_node().is_some())
    else {
        return;
    };
    let Some(call) = context.ancestors()[..block_index]
        .iter()
        .rev()
        .find_map(Node::as_call_node)
    else {
        return;
    };
    if !matches!(call_name(&call), b"reduce" | b"inject") {
        return;
    }
    context.report(
        "Use `next` with an accumulator argument in a `reduce`.",
        node.keyword_loc(),
    );
}

fn unexpected_block_arity(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.receiver().is_none() {
        return;
    }
    let method = String::from_utf8_lossy(call_name(node));
    let Some(expected) = context
        .config_map("Methods")
        .and_then(|methods| methods.get(method.as_ref()))
        .and_then(|arity| arity.parse::<usize>().ok())
    else {
        return;
    };
    let Some(block) = node.block().and_then(|block| block.as_block_node()) else {
        return;
    };
    let source = context.source_file().at(&block.location());
    let Some(actual) = positional_arity(source) else {
        return;
    };
    if actual < expected {
        context.report_call(
            node,
            format!("`{method}` expects at least {expected} positional arguments, got {actual}."),
        );
    }
}

fn positional_arity(source: &str) -> Option<usize> {
    if let Some(first_pipe) = source.find('|') {
        let second_pipe = source[first_pipe + 1..].find('|')? + first_pipe + 1;
        let parameters = source[first_pipe + 1..second_pipe]
            .split(';')
            .next()
            .unwrap_or_default();
        let parameters = source_syntax::split_arguments(parameters, 0, parameters.len())
            .into_iter()
            .map(|range| parameters[range].trim())
            .collect::<Vec<_>>();
        if parameters
            .iter()
            .any(|parameter| parameter.starts_with('*') && !parameter.starts_with("**"))
        {
            return None;
        }
        return Some(
            parameters
                .iter()
                .filter(|parameter| {
                    !parameter.is_empty()
                        && !parameter.starts_with("**")
                        && !parameter.starts_with('&')
                        && !parameter.contains(':')
                })
                .count(),
        );
    }
    let numbered = maximum_numbered_parameter(source);
    if numbered > 0 {
        Some(numbered)
    } else if contains_word(source, "it") {
        Some(1)
    } else {
        Some(0)
    }
}

fn maximum_numbered_parameter(source: &str) -> usize {
    let bytes = source.as_bytes();
    let mut maximum = 0;
    let mut index = 0;
    while index + 1 < bytes.len() {
        if bytes[index] == b'_' && bytes[index + 1].is_ascii_digit() {
            let start = index + 1;
            let mut end = start;
            while bytes.get(end).is_some_and(u8::is_ascii_digit) {
                end += 1;
            }
            maximum = maximum.max(source[start..end].parse().unwrap_or(0));
            index = end;
        } else {
            index += 1;
        }
    }
    maximum
}

fn contains_word(source: &str, expected: &str) -> bool {
    source
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .any(|word| word == expected)
}
