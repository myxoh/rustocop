use super::*;

define_cops! {
    EmptyClassDefinition => "Style/EmptyClassDefinition" => any_node(empty_class_definition),
}

fn empty_class_definition(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if context.policy().enforced_style("class_keyword") == "class_new" {
        if let Some(class) = node.as_class_node() {
            prefer_class_new(&class, context);
        }
    } else {
        prefer_class_keyword(node, context);
    }
}

fn prefer_class_keyword(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (location, name, value) = if let Some(write) = node.as_constant_write_node() {
        (write.location(), write.name_loc(), write.value())
    } else if let Some(write) = node.as_constant_path_write_node() {
        (write.location(), write.target().location(), write.value())
    } else {
        return;
    };
    let Some(factory) = value.as_call_node() else {
        return;
    };
    if call_name(&factory) != b"new"
        || !root_constant(factory.receiver(), b"Class")
        || factory.block().is_some()
    {
        return;
    }
    let Some(parent) = only_argument(&factory) else {
        return;
    };
    if constant_path(&parent).is_none() || allowed_parent(&parent, context) {
        return;
    }
    let file = context.source_file();
    let indentation = file.indentation_text(location.start_offset());
    let replacement = format!(
        "class {} < {}\n{indentation}end",
        file.at(&name),
        file.node(&parent)
    );
    context.replace(
        "Use the `class` keyword instead of `Class.new` to define an empty class.",
        &location,
        &location,
        replacement,
    );
}

fn prefer_class_new(node: &ruby_prism::ClassNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(parent) = node.superclass() else {
        return;
    };
    if !empty_body(node.body()) || allowed_parent(&parent, context) {
        return;
    }
    let file = context.source_file();
    let location = node.location();
    let replacement = format!(
        "{} = Class.new({})",
        file.node(&node.constant_path()),
        file.node(&parent)
    );
    context.replace(
        "Use `Class.new` instead of the `class` keyword to define an empty class.",
        &location,
        &location,
        replacement,
    );
}

fn empty_body(body: Option<Node<'_>>) -> bool {
    body.is_none_or(|body| {
        body.as_statements_node()
            .is_some_and(|statements| statements.body().is_empty())
    })
}

fn allowed_parent(parent: &Node<'_>, context: &CopContext<'_, '_>) -> bool {
    let source = context.source_file().node(parent);
    context
        .config_values("AllowedParentClasses")
        .iter()
        .any(|allowed| allowed == source)
}
