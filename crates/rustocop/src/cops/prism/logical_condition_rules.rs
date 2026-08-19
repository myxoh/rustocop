use super::*;

define_cops! {
    UnlessLogicalOperators => "Style/UnlessLogicalOperators" => node(as_unless_node, unless_logical_operators),
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
