use super::*;

define_cops! {
    SingleArgumentDig => "Style/SingleArgumentDig" => call(single_argument_dig),
}

fn single_argument_dig(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !match_call(node)
        .named(b"dig")
        .with_receiver()
        .with_argument_count(1)
        .matches()
        || node
            .call_operator_loc()
            .is_some_and(|operator| operator.as_slice() == b"&.")
    {
        return;
    }
    let Some(argument) = only_argument(node) else {
        return;
    };
    if argument.as_splat_node().is_some()
        || argument.as_forwarding_arguments_node().is_some()
        || argument.as_hash_node().is_some()
        || argument.as_keyword_hash_node().is_some()
    {
        return;
    }
    let receiver_is_dig = node
        .receiver()
        .and_then(|receiver| receiver.as_call_node())
        .is_some_and(|receiver| call_name(&receiver) == b"dig");
    let inside_dig = context
        .ancestors()
        .iter()
        .rev()
        .filter_map(Node::as_call_node)
        .any(|ancestor| call_name(&ancestor) == b"dig");
    if context.related_config_value("Style/DigChain", "Enabled") == Some("true")
        && (receiver_is_dig || inside_dig)
    {
        return;
    }
    let receiver = node.receiver().expect("receiver checked above");
    let receiver_source = context.source_file().node(&receiver);
    let argument_source = context.source_file().node(&argument);
    let send_range = node.location().start_offset()
        ..node.closing_loc().map_or_else(
            || {
                node.arguments().map_or_else(
                    || {
                        node.message_loc()
                            .map_or(node.location().end_offset(), |loc| loc.end_offset())
                    },
                    |arguments| arguments.location().end_offset(),
                )
            },
            |closing| closing.end_offset(),
        );
    let original = &context.source()[send_range.clone()];
    let message = format!("Use `{receiver_source}[{argument_source}]` instead of `{original}`.");
    if inside_dig && context.autocorrect_enabled() {
        return;
    }
    if inside_dig {
        context.report(message, send_range);
    } else {
        context.replace(
            message,
            send_range.clone(),
            send_range,
            format!("{receiver_source}[{argument_source}]"),
        );
    }
}
