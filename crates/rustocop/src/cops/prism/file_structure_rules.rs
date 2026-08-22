use super::*;

define_cops! {
    OneClassPerFile => "Style/OneClassPerFile" => any_node(one_class_per_file),
}

fn one_class_per_file(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let Some(name_node) = definition_name(node) else {
        return;
    };
    if context.ancestors().iter().any(|ancestor| {
        ancestor.as_class_node().is_some()
            || ancestor.as_module_node().is_some()
            || ancestor.as_singleton_class_node().is_some()
    }) {
        return;
    }
    let Some(program) = context
        .ancestors()
        .iter()
        .rev()
        .find_map(Node::as_program_node)
    else {
        return;
    };
    let current_start = node.location().start_offset();
    let allowed = context.config_values("AllowedClasses");
    return_unless!(program
        .statements()
        .body()
        .iter()
        .any(|candidate| candidate.location().start_offset() == current_start));
    let definitions = program
        .statements()
        .body()
        .iter()
        .filter(|candidate| {
            candidate.location().start_offset() <= current_start
                && definition_name(candidate).is_some_and(|name| {
                    let source = context.source_file().at(&name.location());
                    let short_name = source.rsplit("::").next().unwrap_or(source);
                    !allowed.iter().any(|allowed_name| allowed_name == short_name)
                })
        })
        .count();
    if definitions > 1 {
        context.report(
            "Do not define multiple classes/modules at the top level in a single file.",
            node.location().start_offset()..name_node.location().end_offset(),
        );
    }
}

fn definition_name<'pr>(node: &Node<'pr>) -> Option<Node<'pr>> {
    if let Some(class) = node.as_class_node() {
        Some(class.constant_path())
    } else {
        node.as_module_node().map(|module| module.constant_path())
    }
}
