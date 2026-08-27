use super::*;

define_cops! {
    CollectionQuerying => "Style/CollectionQuerying" => compatibility_prism_call(collection_querying),
}

fn collection_querying(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node
        .call_operator_loc()
        .is_some_and(|operator| operator.as_slice() == b"&.")
    {
        return;
    }
    let Some(count) = node.receiver().and_then(|receiver| receiver.as_call_node()) else {
        return;
    };
    if call_name(&count) != b"count"
        || count.receiver().is_none()
        || !queryable_count_arguments(&count)
    {
        return;
    }
    let Some(preferred) = preferred_query(node, context) else {
        return;
    };
    let (Some(selector), count_location) = (count.message_loc(), count.location()) else {
        return;
    };
    let offense = selector.start_offset()..node.location().end_offset();
    let suffix = context
        .source_file()
        .slice(selector.end_offset()..count_location.end_offset())
        .unwrap_or_default();
    context.replace(
        format!("Use `{preferred}` instead."),
        offense.clone(),
        offense,
        format!("{preferred}{suffix}"),
    );
}

fn queryable_count_arguments(node: &CallNode<'_>) -> bool {
    let arguments = arguments(node);
    arguments.is_empty() || arguments.len() == 1 && arguments[0].as_block_argument_node().is_some()
}

fn preferred_query(node: &CallNode<'_>, context: &CopContext<'_, '_>) -> Option<&'static str> {
    match call_name(node) {
        b"positive?" if argument_count(node) == 0 => Some("any?"),
        b"zero?" if argument_count(node) == 0 => Some("none?"),
        b">" if integer_argument(node) == Some(0) => Some("any?"),
        b"!=" if integer_argument(node) == Some(0) => Some("any?"),
        b"==" if integer_argument(node) == Some(0) => Some("none?"),
        b"==" if integer_argument(node) == Some(1) => Some("one?"),
        b">" if integer_argument(node) == Some(1)
            && context.related_config_value("AllCops", "ActiveSupportExtensionsEnabled")
                == Some("true") =>
        {
            Some("many?")
        }
        _ => None,
    }
}

fn integer_argument(node: &CallNode<'_>) -> Option<i32> {
    TryInto::<i32>::try_into(only_argument(node)?.as_integer_node()?.value()).ok()
}
