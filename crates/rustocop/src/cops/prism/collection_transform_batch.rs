use super::*;

define_cops! {
    HashTransformKeys => "Style/HashTransformKeys" => source(hash_transform_keys),
    HashTransformValues => "Style/HashTransformValues" => source(hash_transform_values),
    MapToSet => "Style/MapToSet" => source(map_to_set),
    HashEachMethods => "Style/HashEachMethods" => source(hash_each_methods),
    HashFetchChain => "Style/HashFetchChain" => source(hash_fetch_chain),
    MapToHash => "Style/MapToHash" => source(map_to_hash),
    HashExcept => "Style/HashExcept" => source(hash_except),
    HashSlice => "Style/HashSlice" => source(hash_slice),
    MapIntoArray => "Style/MapIntoArray" => source(map_into_array),
    CollectionMethods => "Style/CollectionMethods" => source(collection_methods),
    Sample => "Style/Sample" => call(sample),
}

fn hash_transform_keys(context: &mut CopContext<'_, '_>) {
    context.replace_code(
        ".map { |k, v| [yield(k), v] }.to_h",
        ".transform_keys { |k| yield(k) }",
        "Prefer `transform_keys` over `map.to_h`.",
    );
}

fn hash_transform_values(context: &mut CopContext<'_, '_>) {
    context.replace_code(
        ".map { |k, v| [k, yield(v)] }.to_h",
        ".transform_values { |v| yield(v) }",
        "Prefer `transform_values` over `map.to_h`.",
    );
}

fn map_to_set(context: &mut CopContext<'_, '_>) {
    context.replace_code(
        ".map(&:to_s).to_set",
        ".to_set(&:to_s)",
        "Pass a block to `to_set` instead of calling `map.to_set`.",
    );
    context.replace_code(
        ".collect(&:to_s).to_set",
        ".to_set(&:to_s)",
        "Pass a block to `to_set` instead of calling `collect.to_set`.",
    );
}

fn hash_each_methods(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        for (old, new, message) in [
            (
                ".keys.each",
                ".each_key",
                "Use `each_key` instead of `keys.each`.",
            ),
            (
                ".values.each",
                ".each_value",
                "Use `each_value` instead of `values.each`.",
            ),
        ] {
            let Some(at) = line.find(old) else { continue };
            let block_arguments = line[at + old.len()..]
                .split_once('|')
                .and_then(|(_, rest)| rest.split('|').next());
            if block_arguments.is_some_and(|arguments| arguments.contains(',')) {
                continue;
            }
            context.replace(
                message,
                offset + at + 1..offset + at + old.len(),
                offset + at..offset + at + old.len(),
                new,
            );
        }
    }
}

fn hash_fetch_chain(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for (offset, line) in context.source_file().lines() {
        let Some(first) = line.find(".fetch(") else {
            continue;
        };
        let Some(second_relative) = line[first + 7..].find(".fetch(") else {
            continue;
        };
        let second = first + 7 + second_relative;
        let Some(first_close) = line[first..].find(')').map(|at| first + at) else {
            continue;
        };
        let Some(second_close) = line[second..].find(')').map(|at| second + at) else {
            continue;
        };
        let first_arg = &line[first + 7..first_close];
        let second_arg = &line[second + 7..second_close];
        if !first_arg.contains(',') || !second_arg.contains(',') {
            continue;
        }
        let first_arg = first_arg.split(',').next().unwrap_or(first_arg).trim();
        let second_arg = second_arg.split(',').next().unwrap_or(second_arg).trim();
        let replacement = format!(".dig({first_arg}, {second_arg})");
        context.replace(
            "Use `dig` instead of chained `fetch` calls.",
            offset + first..offset + second_close + 1,
            offset + first..offset + second_close + 1,
            replacement,
        );
    }
    let _ = source;
}

fn map_to_hash(context: &mut CopContext<'_, '_>) {
    context.replace_code(
        ".map(&:itself).to_h",
        ".to_h",
        "Pass a block to `to_h` instead of calling `map.to_h`.",
    );
    context.replace_code(
        ".collect(&:itself).to_h",
        ".to_h",
        "Pass a block to `to_h` instead of calling `collect.to_h`.",
    );
}

fn hash_except(context: &mut CopContext<'_, '_>) {
    context.replace_code(
        ".reject { |key, _| keys.include?(key) }",
        ".except(*keys)",
        "Prefer `except` over `reject`.",
    );
}

fn hash_slice(context: &mut CopContext<'_, '_>) {
    context.replace_code(
        ".select { |key, _| keys.include?(key) }",
        ".slice(*keys)",
        "Prefer `slice` over `select`.",
    );
}

fn map_into_array(context: &mut CopContext<'_, '_>) {
    context.replace_code(
        ".each_with_object([])",
        ".filter_map",
        "Prefer `filter_map` over `each_with_object` when building an array.",
    );
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
