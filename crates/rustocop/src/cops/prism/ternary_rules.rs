use super::*;

define_cops! {
    NestedTernaryOperator => "Style/NestedTernaryOperator" => node(as_if_node, nested_ternary_operator),
}

fn nested_ternary_operator(node: &ruby_prism::IfNode<'_>, context: &mut CopContext<'_, '_>) {
    if !is_ternary(node) {
        return;
    }
    let Some(outer) = context
        .ancestors()
        .iter()
        .filter_map(Node::as_if_node)
        .find(is_ternary)
    else {
        return;
    };
    let indirect = context
        .ancestors()
        .iter()
        .filter_map(Node::as_if_node)
        .filter(is_ternary)
        .count()
        > 1
        || context
            .ancestors()
            .iter()
            .any(|ancestor| ancestor.as_block_node().is_some());
    let (Some(then_branch), Some(else_branch)) = (
        outer
            .statements()
            .and_then(|statements| statements.body().first()),
        outer
            .subsequent()
            .and_then(|node| node.as_else_node())
            .and_then(|node| node.statements())
            .and_then(|statements| statements.body().first()),
    ) else {
        return;
    };
    let replacement = format!(
        "if {}\n{}\nelse\n{}\nend",
        context.source_file().node(&outer.predicate()),
        render_direct_ternary_branch(&then_branch, context.source_file()),
        render_direct_ternary_branch(&else_branch, context.source_file())
    );
    let message = "Ternary operators must not be nested. Prefer `if` or `else` constructs instead.";
    if indirect {
        context.replace_indirectly(message, node.location(), outer.location(), replacement);
    } else {
        context.replace(message, node.location(), outer.location(), replacement);
    }
}

fn is_ternary(node: &ruby_prism::IfNode<'_>) -> bool {
    node.if_keyword_loc().is_none()
        && node.then_keyword_loc().is_some()
        && node.end_keyword_loc().is_none()
}

fn unwrap_parenthesized_source(source: &str) -> &str {
    source
        .strip_prefix('(')
        .and_then(|source| source.strip_suffix(')'))
        .unwrap_or(source)
}

fn render_direct_ternary_branch(node: &Node<'_>, file: SourceFile<'_>) -> String {
    unwrap_parenthesized_source(file.node(node)).to_string()
}
