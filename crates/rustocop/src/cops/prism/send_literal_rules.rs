use super::*;

define_cops! {
    SendWithLiteralMethodName => "Style/SendWithLiteralMethodName" => call(send_with_literal_method_name),
}

fn send_with_literal_method_name(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let dispatch = call_name(node);
    if dispatch != b"public_send"
        && (context.config_bool("AllowSend", true) || !matches!(dispatch, b"send" | b"__send__"))
    {
        return;
    }
    let Some(arguments) = node.arguments() else {
        return;
    };
    let arguments = arguments.arguments().iter().collect::<Vec<_>>();
    let Some(method) = arguments
        .first()
        .and_then(|argument| static_symbol(argument).or_else(|| static_string(argument)))
    else {
        return;
    };
    if !direct_method_name(&method) {
        return;
    }
    let method = String::from_utf8_lossy(&method);
    let remaining = arguments
        .iter()
        .skip(1)
        .map(|argument| context.source_file().node(argument))
        .collect::<Vec<_>>()
        .join(", ");
    let replacement = if remaining.is_empty() {
        method.to_string()
    } else if node.opening_loc().is_some() {
        format!("{method}({remaining})")
    } else {
        format!("{method} {remaining}")
    };
    let start = node
        .message_loc()
        .map_or(node.location().start_offset(), |selector| {
            selector.start_offset()
        });
    let offense = start..node.location().end_offset();
    context.replace(
        format!("Use `{method}` method call directly instead."),
        offense.clone(),
        offense,
        replacement,
    );
}

fn direct_method_name(name: &[u8]) -> bool {
    let Some(first) = name.first() else {
        return false;
    };
    if !matches!(first, b'a'..=b'z' | b'A'..=b'Z' | b'_') {
        return false;
    }
    let core = name
        .strip_suffix(b"?")
        .or_else(|| name.strip_suffix(b"!"))
        .unwrap_or(name);
    if core
        .iter()
        .any(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
    {
        return false;
    }
    !matches!(
        core,
        b"alias"
            | b"and"
            | b"begin"
            | b"break"
            | b"case"
            | b"class"
            | b"def"
            | b"defined"
            | b"do"
            | b"else"
            | b"elsif"
            | b"end"
            | b"ensure"
            | b"false"
            | b"for"
            | b"if"
            | b"in"
            | b"module"
            | b"next"
            | b"nil"
            | b"not"
            | b"or"
            | b"redo"
            | b"rescue"
            | b"retry"
            | b"return"
            | b"self"
            | b"super"
            | b"then"
            | b"true"
            | b"undef"
            | b"unless"
            | b"until"
            | b"when"
            | b"while"
            | b"yield"
            | b"BEGIN"
            | b"END"
            | b"__FILE__"
            | b"__LINE__"
            | b"__ENCODING__"
    )
}
