use super::*;

define_cops! {
    ReduceToHash => "Style/ReduceToHash" => source(reduce_to_hash),
    HashTransformKeys => "Style/HashTransformKeys" => source(hash_transform_keys),
    HashTransformValues => "Style/HashTransformValues" => source(hash_transform_values),
    MapToSet => "Style/MapToSet" => source(map_to_set),
    HashEachMethods => "Style/HashEachMethods" => source(hash_each_methods),
    HashFetchChain => "Style/HashFetchChain" => source(hash_fetch_chain),
    PartitionInsteadOfDoubleSelect => "Style/PartitionInsteadOfDoubleSelect" => source(partition_double_select),
    RedundantFilterChain => "Style/RedundantFilterChain" => source(redundant_filter_chain),
    MapToHash => "Style/MapToHash" => source(map_to_hash),
    HashExcept => "Style/HashExcept" => source(hash_except),
    HashSlice => "Style/HashSlice" => source(hash_slice),
    MapIntoArray => "Style/MapIntoArray" => source(map_into_array),
    CollectionMethods => "Style/CollectionMethods" => source(collection_methods),
    Sample => "Style/Sample" => source(sample),
}

fn replace_token(context: &mut CopContext<'_, '_>, old: &str, new: &str, message: &str) {
    for start in context.source_file().code_offsets(old) {
        context.replace(
            message,
            start..start + old.len(),
            start..start + old.len(),
            new,
        );
    }
}

fn reduce_to_hash(context: &mut CopContext<'_, '_>) {
    replace_token(
        context,
        ".map { |key, value| [key, value] }.to_h",
        ".to_h",
        "Prefer `to_h` over `map.to_h` when the pairs are unchanged.",
    );
}

fn hash_transform_keys(context: &mut CopContext<'_, '_>) {
    replace_token(
        context,
        ".map { |k, v| [yield(k), v] }.to_h",
        ".transform_keys { |k| yield(k) }",
        "Prefer `transform_keys` over `map.to_h`.",
    );
}

fn hash_transform_values(context: &mut CopContext<'_, '_>) {
    replace_token(
        context,
        ".map { |k, v| [k, yield(v)] }.to_h",
        ".transform_values { |v| yield(v) }",
        "Prefer `transform_values` over `map.to_h`.",
    );
}

fn map_to_set(context: &mut CopContext<'_, '_>) {
    replace_token(
        context,
        ".map(&:to_s).to_set",
        ".to_set(&:to_s)",
        "Pass a block to `to_set` instead of calling `map.to_set`.",
    );
    replace_token(
        context,
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

fn partition_double_select(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        if line.matches(".select").count() >= 2
            || line.contains(".select") && line.contains(".reject")
        {
            let start = offset + line.find(".select").unwrap_or(0);
            context.report(
                "Use `partition` instead of multiple `select`/`reject` calls.",
                start..offset + line.len(),
            );
        }
    }
}

fn redundant_filter_chain(context: &mut CopContext<'_, '_>) {
    for chain in [".select.select", ".filter.filter", ".reject.reject"] {
        replace_token(
            context,
            chain,
            chain.split('.').nth(1).map_or(chain, |method| {
                if method == "select" {
                    ".select"
                } else if method == "filter" {
                    ".filter"
                } else {
                    ".reject"
                }
            }),
            "Combine chained filtering calls.",
        );
    }
}

fn map_to_hash(context: &mut CopContext<'_, '_>) {
    replace_token(
        context,
        ".map(&:itself).to_h",
        ".to_h",
        "Pass a block to `to_h` instead of calling `map.to_h`.",
    );
    replace_token(
        context,
        ".collect(&:itself).to_h",
        ".to_h",
        "Pass a block to `to_h` instead of calling `collect.to_h`.",
    );
}

fn hash_except(context: &mut CopContext<'_, '_>) {
    replace_token(
        context,
        ".reject { |key, _| keys.include?(key) }",
        ".except(*keys)",
        "Prefer `except` over `reject`.",
    );
}

fn hash_slice(context: &mut CopContext<'_, '_>) {
    replace_token(
        context,
        ".select { |key, _| keys.include?(key) }",
        ".slice(*keys)",
        "Prefer `slice` over `select`.",
    );
}

fn map_into_array(context: &mut CopContext<'_, '_>) {
    replace_token(
        context,
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
            replace_token(
                context,
                old,
                new,
                "Use the configured preferred collection method.",
            );
        }
    }
}

fn sample(context: &mut CopContext<'_, '_>) {
    replace_token(
        context,
        ".shuffle.first",
        ".sample",
        "Use `sample` instead of `shuffle.first`.",
    );
    replace_token(
        context,
        ".shuffle[0]",
        ".sample",
        "Use `sample` instead of `shuffle[0]`.",
    );
}
