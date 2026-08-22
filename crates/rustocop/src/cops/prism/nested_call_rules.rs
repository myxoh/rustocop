use super::*;

define_cops! {
    SafeNavigationChainLength => "Style/SafeNavigationChainLength" => call(safe_navigation_chain_length),
    NestedParenthesizedCalls => "Style/NestedParenthesizedCalls" => call(nested_parenthesized_calls),
}

fn safe_navigation_chain_length(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node
        .call_operator_loc()
        .is_none_or(|operator| operator.as_slice() != b"&.")
    {
        return;
    }
    if node.block().is_some_and(|block| block.as_block_node().is_some()) {
        return;
    }
    let maximum = context.config_usize("Max", 2);
    let mut chain = Vec::new();
    let node_location = node.location();
    let mut child = node_location.start_offset()..node_location.end_offset();
    for ancestor in context.ancestors().iter().rev() {
        let Some(call) = ancestor.as_call_node() else {
            break;
        };
        let receiver_contains_child = call.receiver().is_some_and(|receiver| {
            let location = receiver.location();
            location.start_offset() <= child.start && child.end <= location.end_offset()
        });
        if !receiver_contains_child {
            break;
        }
        if call
            .call_operator_loc()
            .is_none_or(|operator| operator.as_slice() != b"&.")
        {
            break;
        }
        let attached_block = call
            .block()
            .is_some_and(|block| block.as_block_node().is_some());
        let location = call.location();
        child = location.start_offset()..location.end_offset();
        chain.push(call);
        if attached_block {
            break;
        }
    }
    if chain.len() != maximum {
        return;
    }
    let outer = chain.last().expect("long safe-navigation chain");
    let location = outer.location();
    let offense = if outer
        .block()
        .is_some_and(|block| block.as_block_node().is_some())
    {
        location.start_offset()
            ..outer
                .closing_loc()
                .or_else(|| outer.message_loc())
                .map_or(location.end_offset(), |closing| closing.end_offset())
    } else {
        location.start_offset()..location.end_offset()
    };
    context.report(
        format!("Avoid safe navigation chains longer than {maximum} calls."),
        offense,
    );
}

fn nested_parenthesized_calls(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.opening_loc().is_none()
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
