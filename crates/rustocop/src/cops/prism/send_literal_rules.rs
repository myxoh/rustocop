use super::*;

define_cops! {
    SendWithLiteralMethodName => "Style/SendWithLiteralMethodName" => compatibility_prism_call(send_with_literal_method_name),
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
    let selector = node.message_loc().expect("send dispatch has a selector");
    let send_end = node.closing_loc().map_or_else(
        || {
            arguments.last().map_or(selector.end_offset(), |argument| {
                argument.location().end_offset()
            })
        },
        |closing| closing.end_offset(),
    );
    let offense = selector.start_offset()..send_end;
    let message = format!("Use `{method}` method call directly instead.");
    if arguments.len() == 1 {
        context.replace(message, offense.clone(), offense, method);
    } else {
        let first = arguments[0].location();
        let second = arguments[1].location();
        context.replace_many(
            message,
            offense,
            vec![
                (
                    selector.start_offset()..selector.end_offset(),
                    method.to_string(),
                ),
                (first.start_offset()..second.start_offset(), String::new()),
            ],
        );
    }
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
    )
}
