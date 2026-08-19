use super::*;

define_cops! {
    ArrayIntersect => "Style/ArrayIntersect" => source(array_intersect),
    RedundantMinMaxBy => "Style/RedundantMinMaxBy" => source(redundant_min_max_by),
    RedundantSort => "Style/RedundantSort" => source(redundant_sort),
    TallyMethod => "Style/TallyMethod" => source(tally_method),
    ZeroLengthPredicate => "Style/ZeroLengthPredicate" => source(zero_length_predicate),
}

fn array_intersect(context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(3, 1) {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        let code = line.trim();
        let Some(prefix) = code
            .strip_suffix(".any?")
            .or_else(|| code.strip_suffix(".empty?"))
            .or_else(|| code.strip_suffix(".none?"))
        else {
            continue;
        };
        let negated = code.ends_with(".empty?") || code.ends_with(".none?");
        let inner = prefix.trim_matches(['(', ')']);
        let Some((left, right)) = inner.split_once(" & ") else {
            continue;
        };
        let replacement = format!(
            "{}{}.intersect?({})",
            if negated { "!" } else { "" },
            left.trim(),
            right.trim()
        );
        let start = offset + line.find(code).unwrap_or(0);
        context.replace(
            format!("Use `{replacement}` instead of `{code}`."),
            start..start + code.len(),
            start..start + code.len(),
            replacement,
        );
    }
}

fn redundant_min_max_by(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for method in ["max_by", "min_by"] {
        let needle = format!(".{method} {{ |");
        let mut search = 0;
        while let Some(relative) = source[search..].find(&needle) {
            let dot = search + relative;
            let Some(pipe_relative) = source[dot + needle.len()..].find('|') else {
                break;
            };
            let pipe = dot + needle.len() + pipe_relative;
            let parameter = source[dot + needle.len()..pipe].trim();
            let Some(close_relative) = source[pipe + 1..].find('}') else {
                break;
            };
            let end = pipe + 1 + close_relative + 1;
            if source[pipe + 1..end - 1].trim() != parameter {
                search = end;
                continue;
            }
            let preferred = method.trim_end_matches("_by");
            context.replace(
                format!("Use `{preferred}` instead of `{method} {{ |{parameter}| {parameter} }}`."),
                dot + 1..end,
                dot + 1..end,
                preferred,
            );
            search = end;
        }
    }
    for method in ["max_by", "min_by"] {
        let needle = format!(".{method} do |");
        let mut search = 0;
        while let Some(relative) = source[search..].find(&needle) {
            let dot = search + relative;
            let Some(pipe_relative) = source[dot + needle.len()..].find('|') else {
                break;
            };
            let pipe = dot + needle.len() + pipe_relative;
            let parameter = source[dot + needle.len()..pipe].trim();
            let Some(end_relative) = source[pipe + 1..].find("\nend") else {
                break;
            };
            let end = pipe + 1 + end_relative + "\nend".len();
            if source[pipe + 1..pipe + 1 + end_relative].trim() != parameter {
                search = end;
                continue;
            }
            let preferred = method.trim_end_matches("_by");
            context.replace(
                format!("Use `{preferred}` instead of `{}`.", &source[dot + 1..end]),
                dot + 1..end,
                dot + 1..end,
                preferred,
            );
            search = end;
        }
    }
}

fn redundant_sort(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for (suffix, replacement, message_name) in [
        (".sort.first", ".min", "min"),
        (".sort.last", ".max", "max"),
        (".sort_by.first", ".min_by", "min_by"),
        (".sort_by.last", ".max_by", "max_by"),
    ] {
        for start in source
            .match_indices(suffix)
            .map(|(at, _)| at)
            .collect::<Vec<_>>()
        {
            context.replace(
                format!(
                    "Use `{message_name}` instead of `{}...{}`.",
                    suffix.split('.').nth(1).unwrap_or("sort"),
                    suffix.rsplit('.').next().unwrap_or_default()
                ),
                start + 1..start + suffix.len(),
                start..start + suffix.len(),
                replacement,
            );
        }
        let safe_suffix = suffix.replace('.', "&.");
        for start in source
            .match_indices(&safe_suffix)
            .map(|(at, _)| at)
            .collect::<Vec<_>>()
        {
            context.replace(
                format!(
                    "Use `{message_name}` instead of `{}...{}`.",
                    suffix.split('.').nth(1).unwrap_or("sort"),
                    suffix.rsplit('.').next().unwrap_or_default()
                ),
                start + 2..start + safe_suffix.len(),
                start..start + safe_suffix.len(),
                replacement.replace('.', "&."),
            );
        }
    }
}

fn tally_method(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for standalone in [
        "group_by(&:itself).transform_values(&:count)",
        "group_by(&:itself).transform_values(&:size)",
        "group_by(&:itself).transform_values(&:length)",
    ] {
        if source.trim_end() == standalone {
            context.replace(
                "Use `tally` instead of `group_by` and `transform_values`.",
                0.."group_by".len(),
                0..standalone.len(),
                "tally",
            );
            return;
        }
    }
    for suffix in [
        ".group_by(&:itself).transform_values(&:count)",
        ".group_by(&:itself).transform_values(&:size)",
        ".group_by(&:itself).transform_values(&:length)",
        ".group_by { |item| item }.transform_values(&:count)",
    ] {
        if let Some(start) = source.find(suffix) {
            context.replace(
                "Use `tally` instead of `group_by` and `transform_values`.",
                start + 1..start + 1 + "group_by".len(),
                start..start + suffix.len(),
                ".tally",
            );
            return;
        }
    }
    let safe_suffix = "&.group_by(&:itself)&.transform_values(&:count)";
    if let Some(start) = source.find(safe_suffix) {
        context.replace(
            "Use `tally` instead of `group_by` and `transform_values`.",
            start + 2..start + 2 + "group_by".len(),
            start..start + safe_suffix.len(),
            "&.tally",
        );
        return;
    }
    if source.contains("each_with_object(::Hash.new(0))") {
        let rewritten = source.replace("::Hash.new(0)", "Hash.new(0)");
        let Some(start) = rewritten.find(".each_with_object(Hash.new(0))") else {
            return;
        };
        let end = source.trim_end().len();
        context.replace(
            "Use `tally` instead of `each_with_object`.",
            start + 1..start + 1 + "each_with_object".len(),
            start..end,
            ".tally",
        );
        return;
    }
    let needle = ".each_with_object(Hash.new(0))";
    let mut search = 0;
    while let Some(relative) = source[search..].find(needle) {
        let start = search + relative;
        let block_start = start + needle.len();
        let end = if source[block_start..].trim_start().starts_with('{') {
            let Some(end_relative) = source[start..].find('}') else {
                break;
            };
            start + end_relative + 1
        } else if source[block_start..].trim_start().starts_with("do") {
            let Some(end_relative) = source[block_start..].find("\nend") else {
                break;
            };
            block_start + end_relative + "\nend".len()
        } else {
            search = block_start;
            continue;
        };
        let block = &source[start + needle.len()..end];
        if !block.contains("+= 1") {
            search = end;
            continue;
        }
        if block.contains(';') {
            search = end;
            continue;
        }
        let same_key = if let Some(parameters) = block
            .split_once('|')
            .and_then(|(_, rest)| rest.split_once('|'))
        {
            let names = parameters.0.split(',').map(str::trim).collect::<Vec<_>>();
            names.len() == 2 && block.contains(&format!("{}[{}]", names[1], names[0]))
        } else {
            block.contains("_2[_1]")
        };
        if !same_key {
            search = end;
            continue;
        }
        let body_expression = block
            .split('|')
            .nth(2)
            .unwrap_or(block)
            .trim()
            .trim_start_matches('{')
            .trim_start_matches("do")
            .trim_end_matches('}')
            .trim_end_matches("end")
            .trim();
        if body_expression
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count()
            != 1
        {
            search = end;
            continue;
        }
        context.replace(
            "Use `tally` instead of `each_with_object`.",
            start + 1..start + 1 + "each_with_object".len(),
            start..end,
            ".tally",
        );
        search = end;
    }
    let standalone = "each_with_object(Hash.new(0))";
    if source.starts_with(standalone) && source.contains("+= 1") {
        let end = source.trim_end().len();
        context.replace(
            "Use `tally` instead of `each_with_object`.",
            0.."each_with_object".len(),
            0..end,
            "tally",
        );
    }
}

fn zero_length_predicate(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let code = line.trim();
        let patterns = [
            ".length == 0",
            ".size == 0",
            ".count == 0",
            ".length.zero?",
            ".size.zero?",
            ".count.zero?",
        ];
        let Some(pattern) = patterns.into_iter().find(|pattern| code.ends_with(pattern)) else {
            continue;
        };
        let receiver = &code[..code.len() - pattern.len()];
        let method = pattern
            .trim_start_matches('.')
            .trim_end_matches(" == 0")
            .trim_end_matches(".zero?");
        let replacement = format!("{receiver}.empty?");
        let start = offset + line.find(code).unwrap_or(0);
        let original = if pattern.ends_with(".zero?") {
            format!("{method}.zero?")
        } else {
            format!("{method} == 0")
        };
        context.replace(
            format!("Use `empty?` instead of `{original}`."),
            start..start + code.len(),
            start..start + code.len(),
            replacement,
        );
    }
}
