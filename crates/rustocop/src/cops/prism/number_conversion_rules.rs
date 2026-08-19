use super::*;

define_cops! {
    NumberConversion => "Lint/NumberConversion" => call(number_conversion),
}

fn number_conversion(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if direct_conversion(node, context) || block_conversion(node, context) {
        return;
    }
    symbol_argument_conversion(node, context);
}

fn direct_conversion(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) -> bool {
    let Some(parser) = parser_for(call_name(node)) else {
        return false;
    };
    if argument_count(node) != 0 {
        return false;
    }
    let Some(receiver) = node.receiver() else {
        return false;
    };
    if receiver.as_integer_node().is_some()
        || receiver.as_float_node().is_some()
        || numeric_constructor(&receiver)
        || receiver
            .as_call_node()
            .is_some_and(|call| parser_for(call_name(&call)).is_some())
        || ignored_receiver(&receiver, context)
        || allowed_receiver_method(&receiver, context)
    {
        return false;
    }
    let receiver_source = context.source_file().node(&receiver);
    let preferred = parser.render(receiver_source);
    let current = format!(
        "{receiver_source}.{}",
        String::from_utf8_lossy(call_name(node))
    );
    let message = format!(
        "Replace unsafe number conversion with number class parsing, instead of using `{current}`, use stricter `{preferred}`."
    );
    if inside_reported_conversion(node, context) {
        if !context.autocorrect_enabled() {
            context.report_call(node, message);
        }
    } else {
        context.replace_call(node, message, preferred);
    }
    true
}

fn inside_reported_conversion(node: &CallNode<'_>, context: &CopContext<'_, '_>) -> bool {
    context.ancestors().iter().rev().any(|ancestor| {
        let Some(call) = ancestor.as_call_node() else {
            return false;
        };
        if parser_for(call_name(&call)).is_none() || argument_count(&call) != 0 {
            return false;
        }
        let Some(receiver) = call.receiver() else {
            return false;
        };
        receiver
            .as_call_node()
            .is_none_or(|receiver| parser_for(call_name(&receiver)).is_none())
            && receiver.location().start_offset() <= node.location().start_offset()
            && receiver.location().end_offset() >= node.location().end_offset()
    })
}

fn block_conversion(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) -> bool {
    let Some(argument) = node
        .block()
        .and_then(|block| block.as_block_argument_node())
    else {
        return false;
    };
    let Some(symbol) = argument
        .expression()
        .and_then(|expression| expression.as_symbol_node())
    else {
        return false;
    };
    let Some(parser) = parser_for(symbol.unescaped()) else {
        return false;
    };
    rewrite_symbol_conversion(
        node,
        context,
        parser,
        format!("&:{}", String::from_utf8_lossy(symbol.unescaped())),
    );
    true
}

fn symbol_argument_conversion(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) -> bool {
    if !matches!(call_name(node), b"try" | b"send") || node.receiver().is_none() {
        return false;
    }
    let Some(symbol) = only_argument(node).and_then(|argument| argument.as_symbol_node()) else {
        return false;
    };
    let Some(parser) = parser_for(symbol.unescaped()) else {
        return false;
    };
    rewrite_symbol_conversion(
        node,
        context,
        parser,
        format!(":{}", String::from_utf8_lossy(symbol.unescaped())),
    );
    true
}

fn rewrite_symbol_conversion(
    node: &CallNode<'_>,
    context: &mut CopContext<'_, '_>,
    parser: NumberParser,
    original: String,
) {
    let selector = node.message_loc().expect("conversion host has selector");
    let callee = context
        .source_file()
        .slice(node.location().start_offset()..selector.end_offset())
        .unwrap_or_default();
    let block = parser.render("i");
    let preferred = format!("{{ |i| {block} }}");
    context.replace_call(
        node,
        format!(
            "Replace unsafe number conversion with number class parsing, instead of using `{original}`, use stricter `{preferred}`."
        ),
        format!("{callee} {preferred}"),
    );
}

#[derive(Clone, Copy)]
enum NumberParser {
    Integer,
    Float,
    Complex,
    Rational,
}

impl NumberParser {
    fn render(self, value: &str) -> String {
        match self {
            Self::Integer => format!("Integer({value}, 10)"),
            Self::Float => format!("Float({value})"),
            Self::Complex => format!("Complex({value})"),
            Self::Rational => format!("Rational({value})"),
        }
    }
}

fn parser_for(method: &[u8]) -> Option<NumberParser> {
    match method {
        b"to_i" => Some(NumberParser::Integer),
        b"to_f" => Some(NumberParser::Float),
        b"to_c" => Some(NumberParser::Complex),
        b"to_r" => Some(NumberParser::Rational),
        _ => None,
    }
}

fn numeric_constructor(node: &Node<'_>) -> bool {
    node.as_call_node().is_some_and(|call| {
        call.receiver().is_none()
            && matches!(
                call_name(&call),
                b"Integer" | b"Float" | b"Complex" | b"Rational"
            )
    })
}

fn ignored_receiver(node: &Node<'_>, context: &CopContext<'_, '_>) -> bool {
    let ignored = context.config_values("IgnoredClasses");
    receiver_root_constant(node).is_some_and(|name| {
        ignored
            .iter()
            .any(|ignored| ignored.as_bytes() == name.as_slice())
    })
}

fn receiver_root_constant(node: &Node<'_>) -> Option<Vec<u8>> {
    if let Some(path) = constant_path(node) {
        return path.first().map(|name| name.to_vec());
    }
    node.as_call_node()
        .and_then(|call| call.receiver())
        .as_ref()
        .and_then(receiver_root_constant)
}

fn allowed_receiver_method(node: &Node<'_>, context: &CopContext<'_, '_>) -> bool {
    let Some(call) = node.as_call_node() else {
        return false;
    };
    context.policy().allows_method(call_name(&call))
        || call
            .receiver()
            .as_ref()
            .is_some_and(|receiver| allowed_receiver_method(receiver, context))
}
