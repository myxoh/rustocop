use ruby_prism::{CallNode, Location, Node};

pub(super) fn call_name<'pr>(node: &CallNode<'pr>) -> &'pr [u8] {
    node.name().as_slice()
}

pub(super) fn first_argument<'pr>(node: &CallNode<'pr>) -> Option<Node<'pr>> {
    node.arguments()?.arguments().first()
}

pub(super) fn eval_receiver(receiver: Option<Node<'_>>) -> bool {
    let Some(receiver) = receiver else {
        return true;
    };
    if node_is_root_constant(&receiver, b"Kernel") {
        return true;
    }
    receiver.as_call_node().is_some_and(|call| {
        call.receiver().is_none() && call.arguments().is_none() && call_name(&call) == b"binding"
    })
}

pub(super) fn root_constant(receiver: Option<Node<'_>>, expected: &[u8]) -> bool {
    receiver
        .as_ref()
        .is_some_and(|receiver| node_is_root_constant(receiver, expected))
}

pub(super) fn node_is_root_constant(receiver: &Node<'_>, expected: &[u8]) -> bool {
    if let Some(constant) = receiver.as_constant_read_node() {
        return constant.name().as_slice() == expected;
    }
    receiver.as_constant_path_node().is_some_and(|constant| {
        constant.parent().is_none()
            && constant
                .name()
                .is_some_and(|name| name.as_slice() == expected)
    })
}

pub(super) fn constant_read(receiver: Option<Node<'_>>, expected: &[u8]) -> bool {
    receiver
        .and_then(|node| node.as_constant_read_node())
        .is_some_and(|constant| constant.name().as_slice() == expected)
}

pub(super) fn marshal_dump(node: &Node<'_>) -> bool {
    node.as_call_node().is_some_and(|call| {
        call_name(&call) == b"dump" && root_constant(call.receiver(), b"Marshal")
    })
}

pub(super) fn has_keyword(node: &CallNode<'_>, expected: &[u8]) -> bool {
    node.arguments().is_some_and(|arguments| {
        arguments.arguments().iter().any(|argument| {
            argument
                .as_keyword_hash_node()
                .is_some_and(|hash| keyword_hash_contains(&hash, expected))
        })
    })
}

fn keyword_hash_contains(hash: &ruby_prism::KeywordHashNode<'_>, expected: &[u8]) -> bool {
    hash.elements().iter().any(|element| {
        element
            .as_assoc_node()
            .and_then(|association| association.key().as_symbol_node())
            .is_some_and(|symbol| symbol.unescaped() == expected)
    })
}

pub(super) fn recursive_literal_string(node: &Node<'_>) -> bool {
    node.as_interpolated_string_node().is_some_and(|string| {
        string.parts().iter().all(|part| {
            part.as_string_node().is_some()
                || part.as_embedded_statements_node().is_some_and(|embedded| {
                    embedded
                        .statements()
                        .is_some_and(|statements| statements.body().iter().all(literal_node))
                })
        })
    })
}

fn literal_node(node: Node<'_>) -> bool {
    node.as_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_nil_node().is_some()
}

pub(super) fn safe_open_argument(node: &Node<'_>) -> bool {
    if let Some(string) = node.as_string_node() {
        return safe_open_text(string.unescaped());
    }
    if let Some(string) = node.as_interpolated_string_node() {
        return string
            .parts()
            .first()
            .is_some_and(|part| safe_open_argument(&part));
    }
    node.as_call_node().is_some_and(|call| {
        call_name(&call) == b"+"
            && call
                .receiver()
                .is_some_and(|receiver| safe_open_argument(&receiver))
    })
}

fn safe_open_text(text: &[u8]) -> bool {
    !text.is_empty() && !text.starts_with(b"|")
}

pub(super) fn string_starts_with_pipe(node: &Node<'_>) -> bool {
    node.as_string_node()
        .is_some_and(|string| trim_ascii_whitespace(string.unescaped()).starts_with(b"|"))
}

fn trim_ascii_whitespace(mut value: &[u8]) -> &[u8] {
    while value.first().is_some_and(u8::is_ascii_whitespace) {
        value = &value[1..];
    }
    while value.last().is_some_and(u8::is_ascii_whitespace) {
        value = &value[..value.len() - 1];
    }
    value
}

pub(super) fn source_at<'source>(source: &'source str, location: &Location<'_>) -> &'source str {
    &source[location.start_offset()..location.end_offset()]
}

pub(super) fn literal_zero(node: Option<&Node<'_>>) -> bool {
    let Some(node) = node else {
        return false;
    };
    if let Some(integer) = node.as_integer_node() {
        return integer
            .value()
            .to_u32_digits()
            .1
            .iter()
            .all(|digit| *digit == 0);
    }
    node.as_float_node()
        .is_some_and(|float| float.value() == 0.0)
}

pub(super) fn float_expression(node: Option<&Node<'_>>) -> bool {
    let Some(node) = node else {
        return false;
    };
    if node.as_float_node().is_some() {
        return true;
    }
    node.as_call_node().is_some_and(|call| {
        matches!(call_name(&call), b"to_f" | b"fdiv" | b"Float")
            || matches!(call_name(&call), b"+" | b"-" | b"*" | b"**" | b"/" | b"%")
                && (float_expression(call.receiver().as_ref())
                    || first_argument(&call)
                        .as_ref()
                        .is_some_and(|argument| float_expression(Some(argument))))
    })
}

pub(super) fn immutable_literal(node: &Node<'_>) -> bool {
    node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_nil_node().is_some()
        || node.as_regular_expression_node().is_some()
        || node.as_range_node().is_some()
}
