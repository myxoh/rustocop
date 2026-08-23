use super::*;

pub(super) fn redundant_self_assignment(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let mutating = [
        "append",
        "clear",
        "collect!",
        "compare_by_identity",
        "concat",
        "delete_if",
        "fill",
        "initialize_copy",
        "insert",
        "keep_if",
        "map!",
        "merge!",
        "prepend",
        "push",
        "rehash",
        "replace",
        "reverse!",
        "rotate!",
        "shuffle!",
        "sort!",
        "sort_by!",
        "transform_keys!",
        "transform_values!",
        "unshift",
        "update",
    ];
    let source = context.source();
    let variable = if let Some(write) = node.as_local_variable_write_node() {
        Some((write.name_loc(), write.value(), write.location()))
    } else if let Some(write) = node.as_instance_variable_write_node() {
        Some((write.name_loc(), write.value(), write.location()))
    } else if let Some(write) = node.as_class_variable_write_node() {
        Some((write.name_loc(), write.value(), write.location()))
    } else if let Some(write) = node.as_global_variable_write_node() {
        Some((write.name_loc(), write.value(), write.location()))
    } else {
        None
    };
    let (lhs, rhs, assignment, equals) = if let Some((name, value, assignment)) = variable {
        let between = &source[name.end_offset()..value.location().start_offset()];
        let Some(relative_equals) = between.find('=') else {
            return;
        };
        (
            &source[name.start_offset()..name.end_offset()],
            value,
            assignment,
            name.end_offset() + relative_equals,
        )
    } else if let Some(setter) = node.as_call_node() {
        let Some(equal) = setter.equal_loc() else {
            return;
        };
        let Some(arguments) = setter.arguments() else {
            return;
        };
        let arguments = arguments.arguments();
        if arguments.len() != 1 {
            return;
        }
        (
            source[setter.location().start_offset()..equal.start_offset()].trim_end(),
            arguments.first().expect("one setter argument"),
            setter.location(),
            equal.start_offset(),
        )
    } else {
        return;
    };
    let Some(call) = rhs.as_call_node() else {
        return;
    };
    let method = String::from_utf8_lossy(call.name().as_slice());
    if !mutating.contains(&method.as_ref()) {
        return;
    }
    let Some(receiver) = call.receiver() else {
        return;
    };
    if context.source_file().node(&receiver) != lhs {
        return;
    }
    context.replace(
        format!("Redundant self assignment detected. Method `{method}` modifies its receiver in place."),
        equals..equals + 1,
        assignment,
        context.source_file().node(&rhs),
    );
}
