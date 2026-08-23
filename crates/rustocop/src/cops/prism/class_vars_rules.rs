use super::*;

define_cops! {
    ClassVars => "Style/ClassVars" => any_node(class_vars),
}

fn class_vars(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(write) = node.as_class_variable_write_node() {
        let location = write.name_loc();
        let name = context.source_file().at(&location);
        context.report(
            format!("Replace class var {name} with a class instance var."),
            location,
        );
        return;
    }
    let Some(call) = node.as_call_node() else {
        return;
    };
    if call_name(&call) != b"class_variable_set" {
        return;
    }
    let Some(argument) = first_argument(&call) else {
        return;
    };
    let location = argument.location();
    let name = context.source_file().at(&location);
    context.report(
        format!("Replace class var {name} with a class instance var."),
        location,
    );
}
