use super::*;

define_cops! {
    NestedParenthesizedCalls => "Style/NestedParenthesizedCalls" => call(nested_parenthesized_calls),
}


fn nested_parenthesized_calls(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.opening_loc().is_none()
        || call_name(node) == b"[]"
        || call_name(node) == b"[]="
        || call_name(node).ends_with(b"=")
        || operator_method(call_name(node))
    {
        return;
    }
    let Some(arguments) = node.arguments() else {
        return;
    };
    let outer_argument_count = arguments.arguments().len();
    for argument in arguments.arguments().iter() {
        let Some(nested) = argument.as_call_node() else {
            continue;
        };
        let Some(nested_arguments) = nested.arguments() else {
            continue;
        };
        if nested.opening_loc().is_some()
            || nested.block().is_some()
            || call_name(&nested) == b"[]"
            || call_name(&nested).ends_with(b"=")
            || operator_method(call_name(&nested))
            || outer_argument_count == 1
                && nested_arguments.arguments().len() == 1
                && context.policy().allows_method(call_name(&nested))
        {
            continue;
        }
        let Some(first) = nested_arguments.arguments().first() else {
            continue;
        };
        let Some(last) = nested_arguments.arguments().last() else {
            continue;
        };
        let Some(selector) = nested.message_loc() else {
            continue;
        };
        let prefix = context
            .source_file()
            .slice(nested.location().start_offset()..selector.end_offset())
            .unwrap_or_default();
        let values = context
            .source_file()
            .slice(first.location().start_offset()..last.location().end_offset())
            .unwrap_or_default();
        let original = context.source_file().at(&nested.location());
        context.replace_call(
            &nested,
            format!("Add parentheses to nested method call `{original}`."),
            format!("{prefix}({values})"),
        );
    }
}

fn operator_method(name: &[u8]) -> bool {
    matches!(
        name,
        b"+" | b"-"
            | b"*"
            | b"/"
            | b"%"
            | b"**"
            | b"=="
            | b"!="
            | b"<"
            | b">"
            | b"<="
            | b">="
            | b"<=>"
            | b"=~"
            | b"!~"
            | b"&"
            | b"|"
            | b"^"
            | b"<<"
            | b">>"
    )
}
