use super::*;

define_cops! {
    SwapValues => "Style/SwapValues" => compatibility_prism_node(as_statements_node, swap_values),
}

struct SimpleAssignment<'pr> {
    location: ruby_prism::Location<'pr>,
    lhs: String,
    rhs: Node<'pr>,
}

fn swap_values(node: &ruby_prism::StatementsNode<'_>, context: &mut CopContext<'_, '_>) {
    let statements = node.body().iter().collect::<Vec<_>>();
    for window in statements.windows(3) {
        let Some(temporary) = simple_assignment(&window[0], context) else {
            continue;
        };
        let Some(x_assignment) = simple_assignment(&window[1], context) else {
            continue;
        };
        let Some(y_assignment) = simple_assignment(&window[2], context) else {
            continue;
        };
        let file = context.source_file();
        let temporary_value = file.node(&temporary.rhs);
        let x_value = file.node(&x_assignment.rhs);
        let y_value = file.node(&y_assignment.rhs);
        if x_assignment.lhs != temporary_value
            || y_assignment.lhs != x_value
            || y_value != temporary.lhs
        {
            continue;
        }

        let replacement = format!(
            "{}, {} = {}, {}",
            x_assignment.lhs, y_assignment.lhs, y_assignment.lhs, x_assignment.lhs
        );
        let x_line = source_line(context.source(), x_assignment.location.start_offset());
        let y_line = source_line(context.source(), y_assignment.location.start_offset());
        let edit = file.line_start(temporary.location.start_offset())
            ..file.line_end(y_assignment.location.end_offset());
        context.replace(
            format!(
                "Replace this and assignments at lines {x_line} and {y_line} with `{replacement}`."
            ),
            temporary.location,
            edit,
            replacement,
        );
    }
}

fn simple_assignment<'pr>(
    node: &Node<'pr>,
    context: &CopContext<'_, '_>,
) -> Option<SimpleAssignment<'pr>> {
    let file = context.source_file();
    let (location, name, rhs) = if let Some(write) = node.as_local_variable_write_node() {
        (write.location(), write.name_loc(), write.value())
    } else if let Some(write) = node.as_instance_variable_write_node() {
        (write.location(), write.name_loc(), write.value())
    } else if let Some(write) = node.as_class_variable_write_node() {
        (write.location(), write.name_loc(), write.value())
    } else if let Some(write) = node.as_global_variable_write_node() {
        (write.location(), write.name_loc(), write.value())
    } else if let Some(write) = node.as_constant_write_node() {
        (write.location(), write.name_loc(), write.value())
    } else if let Some(write) = node.as_constant_path_write_node() {
        (write.location(), write.target().location(), write.value())
    } else {
        return None;
    };
    Some(SimpleAssignment {
        lhs: file.at(&name).to_string(),
        location,
        rhs,
    })
}

fn source_line(source: &str, offset: usize) -> usize {
    source[..offset.min(source.len())]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}
