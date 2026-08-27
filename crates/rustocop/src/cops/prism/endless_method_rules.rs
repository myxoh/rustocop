use super::*;

define_cops! {
    EndlessMethod => "Style/EndlessMethod" => compatibility_source(endless_method),
}

#[allow(clippy::too_many_lines)]
fn endless_method(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    let style = context
        .policy()
        .enforced_style("allow_single_line")
        .to_string();
    if style == "allow_always" {
        return;
    }
    let source_file = context.source_file();
    let lines = source_file.lines().collect::<Vec<_>>();
    let literal_ranges = source_file.literal_ranges();
    let comment_ranges = source_file.comment_ranges();
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        let Some(def_column) = line.find("def ") else {
            continue;
        };
        let definition_start = offset + def_column;
        if line[..def_column].trim_start().starts_with('#')
            || literal_ranges
                .iter()
                .chain(&comment_ranges)
                .any(|range| range.contains(&definition_start))
        {
            continue;
        }
        let definition = &line[def_column..];
        if let Some(equal) = endless_method_equal(definition) {
            if !matches!(
                style.as_str(),
                "disallow" | "allow_single_line" | "require_single_line"
            ) {
                continue;
            }
            let last = endless_method_expression_end(&lines, index, def_column + equal);
            let multiline = last > index;
            if !multiline && style != "disallow" {
                continue;
            }
            let start = definition_start;
            let end = lines[last].0 + lines[last].1.len();
            let message = if multiline {
                "Avoid endless method definitions with multiple lines."
            } else {
                "Avoid endless method definitions."
            };
            let header = definition[..equal].trim_end().trim_end_matches("()");
            let first_expression = definition[equal + 1..].trim();
            let mut replacement = format!("{header}\n");
            let continuation_start = if first_expression.is_empty() {
                if let Some((_, continuation)) = lines.get(index + 1) {
                    replacement.push_str("  ");
                    replacement.push_str(continuation.trim_start());
                }
                index + 2
            } else {
                replacement.push_str("  ");
                replacement.push_str(first_expression);
                index + 1
            };
            if continuation_start <= last {
                for (_, continuation) in &lines[continuation_start..=last] {
                    replacement.push('\n');
                    replacement.push_str(continuation);
                }
            }
            replacement.push('\n');
            replacement.push_str("end");
            context.replace(message, start..end, start..end, replacement);
            continue;
        }
        if !matches!(style.as_str(), "require_single_line" | "require_always") {
            continue;
        }
        let Some(end_index) = regular_method_end(&lines, index, def_column) else {
            continue;
        };
        let meaningful = lines[index + 1..end_index]
            .iter()
            .filter(|(_, line)| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
            .collect::<Vec<_>>();
        let method_name = definition
            .strip_prefix("def ")
            .unwrap_or(definition)
            .split(['(', ' '])
            .next()
            .unwrap_or_default();
        if meaningful.is_empty()
            || method_name.ends_with('=')
            || meaningful
                .iter()
                .any(|(_, line)| line.contains("<<") || line.trim() == "begin")
        {
            continue;
        }
        let expressions = meaningful
            .iter()
            .filter(|(_, line)| !line.trim_start().starts_with('.'))
            .count();
        if expressions != 1 || style == "require_single_line" && meaningful.len() != 1 {
            continue;
        }
        let header = definition.trim().to_string();
        let first_body_index = lines[index + 1..end_index]
            .iter()
            .position(|(_, line)| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
            .map(|at| index + 1 + at)
            .unwrap_or(index + 1);
        let first_body = lines[first_body_index].1.trim();
        let proposed = format!("{header} = {first_body}");
        let line_length_enabled =
            context.related_config_value("Layout/LineLength", "Enabled") != Some("false");
        let max = context
            .related_config_value("Layout/LineLength", "Max")
            .and_then(|max| max.parse::<usize>().ok())
            .unwrap_or(120);
        if line_length_enabled && def_column + proposed.chars().count() > max {
            continue;
        }
        let start = definition_start;
        let end = lines[end_index].0 + lines[end_index].1.len();
        let mut replacement = proposed;
        for (_, continuation) in &lines[first_body_index + 1..end_index] {
            replacement.push('\n');
            replacement.push_str(continuation);
        }
        let message = if style == "require_always" {
            "Use endless method definitions."
        } else {
            "Use endless method definitions for single line methods."
        };
        context.replace(message, start..end, start..end, replacement);
    }
}

fn endless_method_equal(definition: &str) -> Option<usize> {
    definition.match_indices(" =").find_map(|(at, _)| {
        if definition
            .as_bytes()
            .get(at + 2)
            .is_some_and(|byte| !byte.is_ascii_whitespace())
        {
            return None;
        }
        let signature = &definition[..at];
        let mut parentheses = 0_isize;
        for byte in signature.bytes() {
            match byte {
                b'(' => parentheses += 1,
                b')' => parentheses -= 1,
                _ => {}
            }
        }
        if parentheses != 0 || signature.trim_end().ends_with(',') {
            return None;
        }
        if !signature.contains('(') && signature.split_whitespace().count() != 2 {
            return None;
        }
        Some(at + 1)
    })
}

fn endless_method_expression_end(
    lines: &[(usize, &str)],
    index: usize,
    equal: usize,
) -> usize {
    if lines[index].1.contains(" = begin") {
        return lines[index + 1..]
            .iter()
            .position(|(_, line)| line.trim() == "end")
            .map_or(index, |at| index + 1 + at);
    }
    let delimiter_delta = |source: &str| {
        source.bytes().fold(0_isize, |depth, byte| match byte {
            b'{' | b'[' | b'(' => depth + 1,
            b'}' | b']' | b')' => depth - 1,
            _ => depth,
        })
    };
    if lines[index].1[equal + 1..].trim().is_empty() {
        let mut depth = 0_isize;
        for (relative, (_, line)) in lines[index + 1..].iter().enumerate() {
            depth += delimiter_delta(line);
            if relative == 0 && depth == 0 {
                return index + 1;
            }
            if depth <= 0 {
                return index + 1 + relative;
            }
        }
    }
    let mut depth = delimiter_delta(lines[index].1);
    if depth > 0 {
        for (relative, (_, line)) in lines[index + 1..].iter().enumerate() {
            depth += delimiter_delta(line);
            if depth <= 0 {
                return index + 1 + relative;
            }
        }
    }
    let mut last = index;
    for (relative, (_, line)) in lines[index + 1..].iter().enumerate() {
        if !line.trim_start().starts_with('.') {
            break;
        }
        last = index + 1 + relative;
    }
    last
}

fn regular_method_end(lines: &[(usize, &str)], index: usize, def_column: usize) -> Option<usize> {
    lines[index + 1..]
        .iter()
        .position(|(_, line)| {
            line.trim() == "end" && line.len().saturating_sub(line.trim_start().len()) <= def_column
        })
        .map(|at| index + 1 + at)
}
