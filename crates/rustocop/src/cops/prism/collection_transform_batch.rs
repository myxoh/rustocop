use super::*;

define_cops! {
    CollectionMethods => "Style/CollectionMethods" => source(collection_methods),
    Sample => "Style/Sample" => call(sample),
}

fn collection_methods(context: &mut CopContext<'_, '_>) {
    if context.config_value("PreferredMethods").is_none() {
        return;
    }
    for (old, new) in [
        (".map", ".collect"),
        (".find", ".detect"),
        (".select", ".find_all"),
    ] {
        if context.policy().enforced_style("preferred") == "preferred" {
            context.replace_code(
                old,
                new,
                "Use the configured preferred collection method.",
            );
        }
    }
}

fn sample(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let method = call_name(node);
    if !matches!(method, b"first" | b"last" | b"[]" | b"at" | b"slice") {
        return;
    }
    let Some(shuffle) = node
        .receiver()
        .and_then(|receiver| receiver.as_call_node())
        .filter(|call| call_name(call) == b"shuffle" && call.block().is_none())
    else {
        return;
    };
    let method_arguments = node
        .arguments()
        .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
        .unwrap_or_default();
    let sample_size = match method {
        b"first" | b"last" => method_arguments
            .first()
            .map(|argument| context.source_file().node(argument).to_string()),
        b"[]" | b"at" | b"slice" => {
            let Some(size) = sample_collection_size(&method_arguments) else {
                return;
            };
            size.map(|size| size.to_string())
        }
        _ => return,
    };
    let shuffle_argument = shuffle
        .arguments()
        .and_then(|arguments| arguments.arguments().iter().next())
        .map(|argument| context.source_file().node(&argument).to_string());
    let arguments = [sample_size, shuffle_argument]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(", ");
    let correction = if arguments.is_empty() {
        "sample".to_string()
    } else {
        format!("sample({arguments})")
    };
    let Some(shuffle_selector) = shuffle.message_loc() else {
        return;
    };
    let end = node
        .closing_loc()
        .or_else(|| node.message_loc())
        .map(|location| location.end_offset())
        .unwrap_or_else(|| node.location().end_offset());
    let range = shuffle_selector.start_offset()..end;
    let incorrect = &context.source()[range.clone()];
    context.replace(
        format!("Use `{correction}` instead of `{incorrect}`."),
        range.clone(),
        range,
        correction,
    );
}

/// `None` means the access cannot be converted; `Some(None)` means `sample`
/// without a size, and `Some(Some(n))` means `sample(n)`.
fn sample_collection_size(arguments: &[Node<'_>]) -> Option<Option<i32>> {
    match arguments {
        [argument] => {
            if let Some(value) = sample_integer(argument) {
                return matches!(value, 0 | -1).then_some(None);
            }
            let range = argument.as_range_node()?;
            let low = match range.left() {
                Some(left) => sample_integer(&left)?,
                None => 0,
            };
            let high = match range.right() {
                Some(right) => sample_integer(&right)?,
                None => 0,
            };
            if low != 0 || high < 0 {
                return None;
            }
            let size = if range.operator_loc().as_slice() == b"..." {
                high
            } else {
                high.checked_add(1)?
            };
            Some(Some(size))
        }
        [first, second] if sample_integer(first) == Some(0) => {
            Some(Some(sample_integer(second)?))
        }
        _ => None,
    }
}

fn sample_integer(node: &Node<'_>) -> Option<i32> {
    TryInto::<i32>::try_into(node.as_integer_node()?.value()).ok()
}
