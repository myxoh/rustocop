use super::*;

define_cops! {
    EmptyStringInsideInterpolation => "Style/EmptyStringInsideInterpolation" => any_node(empty_string_inside_interpolation),
}

fn empty_string_inside_interpolation(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let Some(interpolation) = context
        .ancestors()
        .iter()
        .rev()
        .find_map(Node::as_embedded_statements_node)
    else {
        return;
    };
    let style = context
        .policy()
        .enforced_style("trailing_conditional")
        .to_string();
    if style == "ternary" {
        if let Some(condition) = node.as_if_node() {
            trailing_if_to_ternary(&condition, false, &interpolation, context);
        } else if let Some(condition) = node.as_unless_node() {
            trailing_unless_to_ternary(&condition, &interpolation, context);
        }
    } else if let Some(condition) = node.as_if_node() {
        empty_branch_to_modifier(&condition, context);
    }
}

fn empty_branch_to_modifier(node: &ruby_prism::IfNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(if_value) = only_statement(node.statements()) else {
        return;
    };
    let Some(else_value) = node
        .subsequent()
        .and_then(|subsequent| subsequent.as_else_node())
        .and_then(|clause| only_statement(clause.statements()))
    else {
        return;
    };
    let (value, keyword) =
        if empty_value(&else_value, context.source_file()) && literal_value(&if_value) {
            (if_value, "if")
        } else if empty_value(&if_value, context.source_file()) && literal_value(&else_value) {
            (else_value, "unless")
        } else {
            return;
        };
    let replacement = format!(
        "{} {keyword} {}",
        context.source_file().node(&value),
        context.source_file().node(&node.predicate())
    );
    context.replace(
        "Do not return empty strings in string interpolation.",
        node.location(),
        node.location(),
        replacement,
    );
}

fn trailing_if_to_ternary(
    node: &ruby_prism::IfNode<'_>,
    _unless: bool,
    interpolation: &ruby_prism::EmbeddedStatementsNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    if node.subsequent().is_some() || node.end_keyword_loc().is_some() {
        return;
    }
    let Some(value) = only_statement(node.statements()) else {
        return;
    };
    let replacement = format!(
        "{} ? {} : ''",
        context.source_file().node(&node.predicate()),
        context.source_file().node(&value)
    );
    context.replace(
        "Do not use trailing conditionals in string interpolation.",
        interpolation.location(),
        node.location(),
        replacement,
    );
}

fn trailing_unless_to_ternary(
    node: &ruby_prism::UnlessNode<'_>,
    interpolation: &ruby_prism::EmbeddedStatementsNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    if node.else_clause().is_some() || node.end_keyword_loc().is_some() {
        return;
    }
    let Some(value) = only_statement(node.statements()) else {
        return;
    };
    let replacement = format!(
        "{} ? '' : {}",
        context.source_file().node(&node.predicate()),
        context.source_file().node(&value)
    );
    context.replace(
        "Do not use trailing conditionals in string interpolation.",
        interpolation.location(),
        node.location(),
        replacement,
    );
}

fn empty_value(node: &Node<'_>, file: SourceFile<'_>) -> bool {
    node.as_nil_node().is_some()
        || node
            .as_string_node()
            .is_some_and(|_| matches!(file.node(node), "''" | "\"\""))
}

fn literal_value(node: &Node<'_>) -> bool {
    node.as_string_node().is_some()
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_symbol_node().is_some()
}
