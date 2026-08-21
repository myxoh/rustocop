use super::*;

define_cops! {
    SendWithMixinArgument => "Lint/SendWithMixinArgument" => call(send_with_mixin_argument),
    CaseEquality => "Style/CaseEquality" => call(case_equality),
    ExactRegexpMatch => "Style/ExactRegexpMatch" => call(exact_regexp_match),
    IpAddresses => "Style/IpAddresses" => node(as_string_node, ip_addresses),
    LambdaCall => "Style/LambdaCall" => call(lambda_call),
    ObjectThen => "Style/ObjectThen" => call(object_then),
    ReverseFind => "Style/ReverseFind" => call(reverse_find),
    UnpackFirst => "Style/UnpackFirst" => call(unpack_first),
}

fn object_then(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.block().is_none() || argument_count(node) != 0 {
        return;
    }
    let style = context.policy().enforced_style("then");
    let (current, preferred) = if style == "then" && call_name(node) == b"yield_self" {
        if !context.target_ruby_version().at_least(2, 6) {
            return;
        }
        ("yield_self", "then")
    } else if style == "yield_self" && call_name(node) == b"then" {
        ("then", "yield_self")
    } else {
        return;
    };
    let Some(selector) = node.message_loc() else {
        return;
    };
    let replacement = if node.receiver().is_none() && preferred == "then" {
        "self.then"
    } else {
        preferred
    };
    context.replace(
        format!("Prefer `{preferred}` over `{current}`."),
        &selector,
        &selector,
        replacement,
    );
}

fn lambda_call(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) != b"call" || node.receiver().is_none() {
        return;
    }
    let style = context.policy().enforced_style("call");
    let anonymous = node.message_loc().is_none();
    if style == "call" && !anonymous || style == "braces" && anonymous {
        return;
    }
    let receiver = node.receiver().expect("checked receiver");
    let receiver_source = context.source_file().node(&receiver);
    let operator = node
        .call_operator_loc()
        .map(|operator| String::from_utf8_lossy(operator.as_slice()).into_owned())
        .unwrap_or_else(|| ".".to_string());
    let arguments = joined_arguments(node, context.source_file(), ", ");
    let preferred = if style == "call" {
        if arguments.is_empty() {
            format!("{receiver_source}{operator}call")
        } else {
            format!("{receiver_source}{operator}call({arguments})")
        }
    } else {
        format!("{receiver_source}{operator}({arguments})")
    };
    let current = context.source_file().node(&node.as_node());
    let message = format!("Prefer the use of `{preferred}` over `{current}`.");
    let nested_anonymous_receiver = context.ancestors().iter().rev().any(|ancestor| {
        ancestor.as_call_node().is_some_and(|call| {
            call_name(&call) == b"call"
                && call.message_loc().is_none()
                && call.receiver().is_some_and(|receiver| {
                    receiver.location().start_offset() == node.location().start_offset()
                        && receiver.location().end_offset() == node.location().end_offset()
                })
        })
    });
    if nested_anonymous_receiver {
        if !context.autocorrect_enabled() {
            context.report_call(node, message);
        }
    } else {
        context.replace_call(node, message, preferred);
    }
}

fn case_equality(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) != b"===" {
        return;
    }
    let (Some(receiver), Some(argument), Some(selector)) =
        (node.receiver(), only_argument(node), node.message_loc())
    else {
        return;
    };
    if receiver.as_regular_expression_node().is_some() {
        return;
    }
    let receiver_source = context.source_file().node(&receiver).to_string();
    let argument_source = context.source_file().node(&argument).to_string();
    let constant_path = constant_path(&receiver);
    let constant = constant_path.as_ref().is_some_and(|parts| {
        parts.last().is_some_and(|name| {
            name.first().is_some_and(u8::is_ascii_uppercase)
                && name.iter().any(u8::is_ascii_lowercase)
        })
    });
    if constant_path.is_some() && !constant {
        return;
    }
    let self_class = receiver.as_call_node().is_some_and(|call| {
        call_name(&call) == b"class"
            && argument_count(&call) == 0
            && call
                .receiver()
                .is_some_and(|receiver| receiver.as_self_node().is_some())
    });
    if constant && context.config_bool("AllowOnConstant", false)
        || self_class && context.config_bool("AllowOnSelfClass", false)
    {
        return;
    }
    let message = "Avoid the use of the case equality operator `===`.";
    let range_receiver = receiver.as_range_node().is_some()
        || receiver
            .as_parentheses_node()
            .and_then(|parentheses| parentheses.body())
            .and_then(|body| body.as_statements_node())
            .and_then(|statements| statements.body().first())
            .is_some_and(|expression| expression.as_range_node().is_some());
    let replacement = if range_receiver {
        Some(format!("{receiver_source}.include?({argument_source})"))
    } else if constant || self_class {
        Some(format!("{argument_source}.is_a?({receiver_source})"))
    } else {
        None
    };
    if let Some(replacement) = replacement {
        context.replace(message, &selector, node.location(), replacement);
    } else {
        context.report(message, &selector);
    }
}

fn reverse_find(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(4, 0)
        || !matches!(call_name(node), b"find" | b"detect")
    {
        return;
    }
    let Some(reverse) = node.receiver().and_then(|receiver| receiver.as_call_node()) else {
        return;
    };
    if !matches!(call_name(&reverse), b"reverse" | b"reverse_each")
        || argument_count(&reverse) != 0
        || reverse.receiver().is_none()
    {
        return;
    }
    let (Some(reverse_selector), Some(find_selector)) = (reverse.message_loc(), node.message_loc())
    else {
        return;
    };
    context.replace(
        "Use `rfind` instead.",
        reverse_selector.start_offset()..find_selector.end_offset(),
        reverse_selector.start_offset()..find_selector.end_offset(),
        "rfind",
    );
}

fn unpack_first(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let supported_tail = call_name(node) == b"first" && argument_count(node) == 0
        || matches!(call_name(node), b"[]" | b"slice" | b"at")
            && only_argument(node).is_some_and(|argument| {
                argument.as_integer_node().is_some_and(|integer| {
                    TryInto::<i32>::try_into(integer.value()).ok() == Some(0)
                })
            });
    if !supported_tail || !context.target_ruby_version().at_least(2, 4) {
        return;
    }
    let Some(unpack) = node.receiver().and_then(|receiver| receiver.as_call_node()) else {
        return;
    };
    if call_name(&unpack) != b"unpack" || argument_count(&unpack) != 1 {
        return;
    }
    let Some(selector) = unpack.message_loc() else {
        return;
    };
    let current = context
        .source_file()
        .slice(selector.start_offset()..node.location().end_offset())
        .unwrap_or_default();
    let first_format = first_argument(&unpack)
        .map(|argument| context.source_file().node(&argument))
        .unwrap_or_default();
    context.replace_many(
        format!("Use `unpack1({first_format})` instead of `{current}`."),
        selector.start_offset()..node.location().end_offset(),
        vec![
            (
                selector.start_offset()..selector.end_offset(),
                "unpack1".to_string(),
            ),
            (
                unpack.location().end_offset()..node.location().end_offset(),
                String::new(),
            ),
        ],
    );
}

fn send_with_mixin_argument(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !matches!(call_name(node), b"send" | b"public_send" | b"__send__")
        || node.receiver().as_ref().and_then(constant_path).is_none()
    {
        return;
    }
    let Some(arguments) = node.arguments() else {
        return;
    };
    let values = arguments.arguments().iter().collect::<Vec<_>>();
    let Some(method) = values.first().and_then(static_symbol_or_string) else {
        return;
    };
    if !matches!(method.as_slice(), b"include" | b"prepend" | b"extend")
        || values.len() < 2
        || values[1..]
            .iter()
            .any(|argument| constant_path(argument).is_none())
    {
        return;
    }
    let Some(selector) = node.message_loc() else {
        return;
    };
    let modules = values[1..]
        .iter()
        .map(|argument| context.source_file().node(argument))
        .collect::<Vec<_>>()
        .join(", ");
    let bad = context
        .source_file()
        .slice(selector.start_offset()..node.location().end_offset())
        .unwrap_or_default();
    let method = String::from_utf8_lossy(&method);
    let replacement = format!("{method} {modules}");
    context.replace(
        format!("Use `{replacement}` instead of `{bad}`."),
        selector.start_offset()..node.location().end_offset(),
        selector.start_offset()..node.location().end_offset(),
        replacement,
    );
}

fn exact_regexp_match(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !matches!(
        call_name(node),
        b"=~" | b"===" | b"!~" | b"match" | b"match?"
    ) {
        return;
    }
    let (Some(receiver), Some(regexp)) = (
        node.receiver(),
        only_argument(node).and_then(|argument| argument.as_regular_expression_node()),
    ) else {
        return;
    };
    if regexp.is_ignore_case()
        || regexp.is_extended()
        || regexp.is_multi_line()
        || regexp.is_once()
        || regexp.is_euc_jp()
        || regexp.is_ascii_8bit()
        || regexp.is_windows_31j()
        || regexp.is_utf_8()
    {
        return;
    }
    let pattern = regexp.unescaped();
    let Some(literal) = pattern
        .strip_prefix(br"\A")
        .and_then(|pattern| pattern.strip_suffix(br"\z"))
    else {
        return;
    };
    if literal.is_empty()
        || literal.iter().any(|byte| {
            matches!(
                byte,
                b'\\'
                    | b'.'
                    | b'['
                    | b']'
                    | b'('
                    | b')'
                    | b'{'
                    | b'}'
                    | b'*'
                    | b'+'
                    | b'?'
                    | b'|'
                    | b'^'
                    | b'$'
            )
        })
    {
        return;
    }
    let receiver = context.source_file().node(&receiver);
    let operator = if call_name(node) == b"!~" { "!=" } else { "==" };
    let literal = String::from_utf8_lossy(literal);
    let preferred = format!("{receiver} {operator} '{literal}'");
    context.replace_call(node, format!("Use `{preferred}`."), preferred);
}

fn ip_addresses(node: &ruby_prism::StringNode<'_>, context: &mut CopContext<'_, '_>) {
    let value = String::from_utf8_lossy(node.unescaped());
    if value.is_empty()
        || value.len() > 45
        || context
            .config_values("AllowedAddresses")
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(&value))
        || value.parse::<std::net::IpAddr>().is_err()
    {
        return;
    }
    context.report("Do not hardcode IP addresses.", node.location());
}

fn static_symbol_or_string(node: &Node<'_>) -> Option<Vec<u8>> {
    node.as_symbol_node()
        .map(|symbol| symbol.unescaped().to_vec())
        .or_else(|| {
            node.as_string_node()
                .map(|string| string.unescaped().to_vec())
        })
}
