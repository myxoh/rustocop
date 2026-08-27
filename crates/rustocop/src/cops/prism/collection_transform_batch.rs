use super::*;

define_cops! {
    CollectionMethods => "Style/CollectionMethods" => compatibility_prism_call(collection_methods),
    Sample => "Style/Sample" => compatibility_prism_call(sample),
}

fn collection_methods(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let method = String::from_utf8_lossy(call_name(node)).to_string();
    let lookup = method.strip_suffix('!').unwrap_or(&method);
    let Some(preferred) = context
        .config_map("PreferredMethods")
        .and_then(|methods| methods.get(lookup))
        .cloned()
    else {
        return;
    };
    let arguments = node
        .arguments()
        .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
        .unwrap_or_default();
    let implicit_block = arguments.last().is_some_and(|argument| {
        argument.as_block_argument_node().is_some()
            || argument.as_symbol_node().is_some()
                && context
                    .config_values("MethodsAcceptingSymbol")
                    .iter()
                    .any(|configured| configured == lookup)
    });
    if node.block().is_none() && !implicit_block {
        return;
    }
    let preferred = if method.ends_with('!') {
        format!("{preferred}!")
    } else {
        preferred
    };
    let Some(selector) = node.message_loc() else {
        return;
    };
    context.replace(
        format!("Prefer `{preferred}` over `{method}`."),
        &selector,
        &selector,
        preferred,
    );
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
        [first, second] if sample_integer(first) == Some(0) => Some(Some(sample_integer(second)?)),
        _ => None,
    }
}

fn sample_integer(node: &Node<'_>) -> Option<i32> {
    TryInto::<i32>::try_into(node.as_integer_node()?.value()).ok()
}
