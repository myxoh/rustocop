use super::*;

define_cops! {
    MultipleComparison => "Lint/MultipleComparison" => call(multiple_comparison),
    UnlessLogicalOperators => "Style/UnlessLogicalOperators" => node(as_unless_node, unless_logical_operators),
}

fn multiple_comparison(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !comparison_operator(call_name(node)) {
        return;
    }
    let (Some(inner), Some(right)) = (
        node.receiver().and_then(|receiver| receiver.as_call_node()),
        only_argument(node),
    ) else {
        return;
    };
    if !comparison_operator(call_name(&inner)) || argument_count(&inner) != 1 {
        return;
    }
    let (Some(left), Some(middle)) = (inner.receiver(), only_argument(&inner)) else {
        return;
    };
    if middle
        .as_call_node()
        .is_some_and(|call| matches!(call_name(&call), b"&" | b"|" | b"^"))
    {
        return;
    }
    let file = context.source_file();
    let replacement = format!(
        "{} {} {} && {} {} {}",
        file.node(&left),
        String::from_utf8_lossy(call_name(&inner)),
        file.node(&middle),
        file.node(&middle),
        String::from_utf8_lossy(call_name(node)),
        file.node(&right)
    );
    context.replace_call(
        node,
        "Use the `&&` operator to compare multiple values.",
        replacement,
    );
}

fn comparison_operator(name: &[u8]) -> bool {
    matches!(name, b"<" | b">" | b"<=" | b">=")
}

fn unless_logical_operators(node: &ruby_prism::UnlessNode<'_>, context: &mut CopContext<'_, '_>) {
    let predicate = context.source_file().node(&node.predicate());
    let operators = logical_operators(predicate);
    let style = context
        .policy()
        .enforced_style("forbid_mixed_logical_operators")
        .to_string();
    let message = if style == "forbid_logical_operators" && !operators.is_empty() {
        "Do not use any logical operator in an `unless`."
    } else if style == "forbid_mixed_logical_operators" && operators.len() > 1 {
        "Do not use mixed logical operators in an `unless`."
    } else {
        return;
    };
    context.report(message, node.location());
}

fn logical_operators(source: &str) -> Vec<&'static str> {
    let mut found = Vec::new();
    for (operator, present) in [
        ("&&", source.contains("&&")),
        ("||", source.contains("||")),
        ("and", contains_word(source, "and")),
        ("or", contains_word(source, "or")),
    ] {
        if present {
            found.push(operator);
        }
    }
    found
}

fn contains_word(source: &str, expected: &str) -> bool {
    source
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .any(|word| word == expected)
}
