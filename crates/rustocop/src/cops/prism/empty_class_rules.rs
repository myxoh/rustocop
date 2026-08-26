use super::*;

define_cops! {
    EmptyClass => "Lint/EmptyClass" => any_node(empty_class),
}

fn empty_class(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (body, superclass, message) = if let Some(class_node) = node.as_class_node() {
        (
            class_node.body(),
            class_node.superclass(),
            "Empty class detected.",
        )
    } else if let Some(class_node) = node.as_singleton_class_node() {
        (class_node.body(), None, "Empty metaclass detected.")
    } else {
        return;
    };
    if body.is_some() || superclass.is_some() {
        return;
    }
    if context.config_bool("AllowComments", false)
        && context
            .source_file()
            .node(node)
            .lines()
            .any(|line| line.trim_start().starts_with('#'))
    {
        return;
    }
    context.report(message, node.location());
}
