use super::*;

define_cops! {
    RedundantException => "Style/RedundantException" => call(redundant_exception),
}

fn redundant_exception(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !matches!(call_name(node), b"raise" | b"fail") || node.receiver().is_some() {
        return;
    }
    let Some(arguments) = node.arguments() else {
        return;
    };
    let arguments = arguments.arguments();
    let keyword = String::from_utf8_lossy(call_name(node));

    let (message, description, keep_parentheses) =
        if arguments.len() == 2 && root_constant(arguments.first(), b"RuntimeError") {
            (
                arguments.iter().nth(1).expect("two arguments checked"),
                "Redundant `RuntimeError` argument can be removed.",
                node.opening_loc().is_some(),
            )
        } else if arguments.len() == 1 {
            let Some(constructor) = arguments
                .first()
                .and_then(|argument| argument.as_call_node())
            else {
                return;
            };
            if call_name(&constructor) != b"new"
                || !root_constant(constructor.receiver(), b"RuntimeError")
            {
                return;
            }
            let Some(message) = only_argument(&constructor) else {
                return;
            };
            (
                message,
                "Redundant `RuntimeError.new` call can be replaced with just the message.",
                false,
            )
        } else {
            return;
        };

    let mut rendered = context.source_file().node(&message).to_string();
    if !string_like(&message) {
        rendered.push_str(".to_s");
    }
    let replacement = if keep_parentheses {
        format!("{keyword}({rendered})")
    } else {
        format!("{keyword} {rendered}")
    };
    context.replace_call(node, description, replacement);
}

fn string_like(node: &Node<'_>) -> bool {
    node.as_string_node().is_some()
        || node.as_interpolated_string_node().is_some()
        || node.as_x_string_node().is_some()
        || node.as_interpolated_x_string_node().is_some()
}
