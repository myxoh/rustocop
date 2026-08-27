use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> { Vec::new() }

fn file_empty(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(2, 4) {
        return;
    }
    if call_name(node) == b"zero?" {
        if let Some((constant, argument)) = direct_file_call(node, &[b"zero?"]) {
            report_file_predicate(node, &constant, &argument, false, context);
            return;
        }
        if let Some(size) = node.receiver().and_then(|receiver| receiver.as_call_node()) {
            if let Some((constant, argument)) = direct_file_call(&size, &[b"size"]) {
                report_file_predicate(node, &constant, &argument, false, context);
            }
        }
        return;
    }
    if call_name(node) == b"empty?" {
        let Some(read) = node.receiver().and_then(|receiver| receiver.as_call_node()) else {
            return;
        };
        if let Some((constant, argument)) = direct_file_call(&read, &[b"read", b"binread"]) {
            report_file_predicate(node, &constant, &argument, false, context);
        }
        return;
    }
    if !matches!(call_name(node), b"==" | b"!=" | b">=") {
        return;
    }
    let Some(expected) = only_argument(node) else {
        return;
    };
    let (receiver, initially_negated) = unwrap_not(node.receiver());
    let Some(operation) = receiver.and_then(|receiver| receiver.as_call_node()) else {
        return;
    };
    let comparison = call_name(node);
    let (constant, argument, negate) = if call_name(&operation) == b"size"
        && matches!(comparison, b"==" | b">=")
        && is_zero(&expected)
    {
        let Some((constant, argument)) = direct_file_call(&operation, &[b"size"]) else {
            return;
        };
        (constant, argument, comparison == b">=")
    } else if matches!(call_name(&operation), b"read" | b"binread")
        && static_string(&expected).is_some_and(|string| string.is_empty())
    {
        let Some((constant, argument)) = direct_file_call(&operation, &[b"read", b"binread"])
        else {
            return;
        };
        (constant, argument, comparison == b"!=")
    } else {
        return;
    };
    report_file_predicate(
        node,
        &constant,
        &argument,
        negate ^ initially_negated,
        context,
    );
}

fn direct_file_call<'pr>(node: &CallNode<'pr>, methods: &[&[u8]]) -> Option<(String, Node<'pr>)> {
    if !methods.contains(&call_name(node)) {
        return None;
    }
    let receiver = node.receiver()?;
    let constant = constant_path(&receiver)?;
    if constant.len() != 1 || !matches!(constant[0], b"File" | b"FileTest") {
        return None;
    }
    let argument = only_argument(node)?;
    Some((String::from_utf8_lossy(constant[0]).into_owned(), argument))
}

fn unwrap_not(receiver: Option<Node<'_>>) -> (Option<Node<'_>>, bool) {
    let Some(receiver) = receiver else {
        return (None, false);
    };
    if let Some(not) = receiver
        .as_call_node()
        .filter(|call| call_name(call) == b"!")
    {
        (not.receiver(), true)
    } else {
        (Some(receiver), false)
    }
}

fn is_zero(node: &Node<'_>) -> bool {
    node.as_integer_node()
        .is_some_and(|integer| TryInto::<i32>::try_into(integer.value()).ok() == Some(0))
}

fn report_file_predicate(
    node: &CallNode<'_>,
    constant: &str,
    argument: &Node<'_>,
    negate: bool,
    context: &mut CopContext<'_, '_>,
) {
    let preferred = format!(
        "{constant}.empty?({})",
        context.source_file().node(argument)
    );
    context.replace_call(
        node,
        format!("Use `{preferred}` instead."),
        format!("{}{preferred}", if negate { "!" } else { "" }),
    );
}
