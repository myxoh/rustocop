use super::*;

define_cops! {
    IncompatibleIoSelectWithFiberScheduler => "Lint/IncompatibleIoSelectWithFiberScheduler" => compatibility_prism_call(incompatible_io_select),
}

fn incompatible_io_select(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) != b"select" || !root_constant(node.receiver(), b"IO") {
        return;
    }
    let arguments = node
        .arguments()
        .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
        .unwrap_or_default();
    if arguments.is_empty() || arguments.len() > 4 {
        return;
    }
    let read = single_io(arguments.first());
    let write = single_io(arguments.get(1));
    let read_slot_valid = read.is_some() || empty_io_slot(arguments.first());
    let write_slot_valid = write.is_some() || empty_io_slot(arguments.get(1));
    let exception_slot_valid = empty_io_slot(arguments.get(2));
    if !read_slot_valid
        || !write_slot_valid
        || !exception_slot_valid
        || read.is_some() == write.is_some()
    {
        return;
    }
    let (io, method) = if let Some(io) = read {
        (io, "wait_readable")
    } else {
        (write.expect("one side is present"), "wait_writable")
    };
    let timeout = arguments
        .get(3)
        .map(|argument| context.source_file().node(argument));
    let replacement = format!(
        "{}.{method}{}",
        context.source_file().node(&io),
        timeout.map_or_else(String::new, |timeout| format!("({timeout})"))
    );
    let original = context.source_file().at(&node.location());
    let message = format!("Use `{replacement}` instead of `{original}`.");
    if context.ancestors().iter().any(assignment_node) {
        context.report_call(node, message);
    } else {
        context.replace_call(node, message, replacement);
    }
}

fn single_io<'pr>(node: Option<&Node<'pr>>) -> Option<Node<'pr>> {
    let array = node?.as_array_node()?;
    (array.elements().len() == 1)
        .then(|| array.elements().first())
        .flatten()
}

fn empty_io_slot(node: Option<&Node<'_>>) -> bool {
    node.is_none_or(|node| {
        node.as_nil_node().is_some()
            || node
                .as_array_node()
                .is_some_and(|array| array.elements().is_empty())
    })
}

fn assignment_node(node: &Node<'_>) -> bool {
    node.as_multi_write_node().is_some()
        || node.as_local_variable_write_node().is_some()
        || node.as_instance_variable_write_node().is_some()
        || node.as_class_variable_write_node().is_some()
        || node.as_global_variable_write_node().is_some()
        || node.as_constant_write_node().is_some()
        || node.as_constant_path_write_node().is_some()
}
