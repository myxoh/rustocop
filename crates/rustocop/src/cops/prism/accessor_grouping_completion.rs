use super::*;

define_cops! {
    AccessorGrouping => "Style/AccessorGrouping" => source(accessor_grouping),
}

fn accessor_grouping(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    if context.policy().enforced_style("grouped") == "separated" {
        separated_accessors(context, &lines);
    } else {
        grouped_accessors(context, &lines);
    }
}

fn separated_accessors(context: &mut CopContext<'_, '_>, lines: &[(usize, &str)]) {
    separate_parenthesized_accessors(context);
    separate_multiline_accessors(context, lines);
    separate_inline_accessors(context, lines);
}

fn separate_parenthesized_accessors(context: &mut CopContext<'_, '_>) {
    for accessor in ["attr_reader", "attr_writer", "attr_accessor"] {
        let needle = format!("{accessor}(");
        let mut search = 0usize;
        while let Some(relative) = context.source()[search..].find(&needle) {
            let start = search + relative;
            let Some(close_relative) = context.source()[start..].find(')') else {
                break;
            };
            let end = start + close_relative + 1;
            let call = &context.source()[start..end];
            if call.contains('\n') && call.contains(',') {
                let line_start = context.source_file().line_start(start);
                let indent = &context.source()[line_start..start];
                let replacement = separated_multiline_replacement(call, accessor, indent);
                context.replace(
                    format!("Use one attribute per `{accessor}`."),
                    start..end,
                    start..end,
                    replacement,
                );
            }
            search = end;
        }
    }
}

fn separate_multiline_accessors(context: &mut CopContext<'_, '_>, lines: &[(usize, &str)]) {
    for (index, (offset, line)) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        let Some(accessor) = ["attr_reader", "attr_writer", "attr_accessor"]
            .into_iter()
            .find(|accessor| trimmed.starts_with(&format!("{accessor} ")))
        else {
            continue;
        };
        if !line
            .split('#')
            .next()
            .unwrap_or(line)
            .trim_end()
            .ends_with(',')
        {
            continue;
        }
        let mut end = offset + line.len();
        let mut offense_end = offset + line.split('#').next().unwrap_or(line).trim_end().len();
        for (next_offset, next) in lines.iter().skip(index + 1) {
            if !next.trim_start().starts_with(':') {
                break;
            }
            end = next_offset + next.len();
            offense_end = next_offset + next.split('#').next().unwrap_or(next).trim_end().len();
            if !next
                .split('#')
                .next()
                .unwrap_or(next)
                .trim_end()
                .ends_with(',')
            {
                break;
            }
        }
        let start = offset + line.len() - trimmed.len();
        let indent = &line[..line.len() - trimmed.len()];
        let replacement =
            separated_multiline_replacement(&context.source()[start..end], accessor, indent);
        context.replace(
            format!("Use one attribute per `{accessor}`."),
            start..offense_end,
            start..end,
            replacement,
        );
    }
}

fn separated_multiline_replacement(source: &str, accessor: &str, indent: &str) -> String {
    let mut pending_comments = Vec::<String>::new();
    let mut output = Vec::<(String, bool)>::new();
    let mut first_attribute = true;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            pending_comments.push(trimmed.to_string());
            continue;
        }
        let values = trimmed
            .strip_prefix(accessor)
            .unwrap_or(trimmed)
            .trim_start_matches('(')
            .trim_end_matches(')')
            .trim();
        if values.is_empty() {
            continue;
        }
        let (attribute, inline_comment) = values
            .split_once('#')
            .map_or((values, None), |(attribute, comment)| {
                (attribute, Some(format!("#{}", comment.trim_end())))
            });
        let attribute = attribute.trim().trim_end_matches(',').trim();
        if !attribute.starts_with(':') {
            continue;
        }
        if let Some(comment) = inline_comment {
            pending_comments.push(comment);
        }
        for comment in pending_comments.drain(..) {
            output.push((comment, !output.is_empty()));
        }
        output.push((format!("{accessor} {attribute}"), !first_attribute));
        first_attribute = false;
    }
    output
        .into_iter()
        .map(|(line, indented)| {
            if indented {
                format!("{indent}{line}")
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn separate_inline_accessors(context: &mut CopContext<'_, '_>, lines: &[(usize, &str)]) {
    let mut preceding_comment = false;
    for (offset, line) in lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            preceding_comment = true;
            continue;
        }
        if trimmed.trim().is_empty() {
            continue;
        }
        let Some(accessor) = ["attr_reader", "attr_writer", "attr_accessor"]
            .into_iter()
            .find(|accessor| {
                trimmed.starts_with(*accessor)
                    && trimmed
                        .as_bytes()
                        .get(accessor.len())
                        .is_some_and(|byte| matches!(byte, b' ' | b'('))
            })
        else {
            preceding_comment = false;
            continue;
        };
        if preceding_comment {
            preceding_comment = false;
            continue;
        }
        if line
            .split('#')
            .next()
            .unwrap_or(line)
            .trim_end()
            .ends_with(',')
        {
            continue;
        }
        let attributes = trimmed[accessor.len()..]
            .split(',')
            .map(str::trim)
            .collect::<Vec<_>>();
        if attributes.len() < 2 {
            continue;
        }
        let indent = &line[..line.len() - trimmed.len()];
        let mut seen = Vec::new();
        let replacement = attributes
            .iter()
            .map(|attribute| {
                let duplicate = seen.contains(attribute);
                seen.push(*attribute);
                let line_indent = if duplicate { "" } else { indent };
                format!("{line_indent}{accessor} {attribute}")
            })
            .collect::<Vec<_>>()
            .join("\n");
        context.replace(
            format!("Use one attribute per `{accessor}`."),
            *offset + indent.len()..*offset + line.len(),
            *offset..*offset + line.len(),
            replacement,
        );
    }
}

type AccessorLine = (usize, usize, &'static str, String);

fn accessor_line(
    offset: usize,
    line: &str,
    trimmed: &str,
    accessor: &'static str,
    trim_parentheses: bool,
) -> AccessorLine {
    let start = offset + line.len() - trimmed.len();
    let end = offset + line.split('#').next().unwrap_or(line).trim_end().len();
    let values = trimmed[accessor.len()..]
        .split('#')
        .next()
        .unwrap_or_default()
        .trim();
    let values = if trim_parentheses {
        values.trim_matches(['(', ')'])
    } else {
        values
    };
    (start, end, accessor, values.to_string())
}

fn flush_accessor_group(group: &mut Vec<AccessorLine>, context: &mut CopContext<'_, '_>) {
    if group.len() < 2 {
        group.clear();
        return;
    }
    let accessor = group[0].2;
    let mut attributes = Vec::new();
    for (_, _, _, values) in group.iter() {
        for attribute in values.split(',').map(str::trim) {
            if !attributes.contains(&attribute) {
                attributes.push(attribute);
            }
        }
    }
    let anchor = group
        .iter()
        .position(|entry| entry.3.contains('*'))
        .unwrap_or(0);
    let first_start = group[anchor].0;
    let first_end = group[anchor].1;
    let mut edits = vec![(
        first_start..first_end,
        format!("{accessor} {}", attributes.join(", ")),
    )];
    for (index, (start, end, _, _)) in group.iter().enumerate() {
        if index == anchor {
            continue;
        }
        let line_start = context.source_file().line_start(*start);
        let preceded_by_blank = line_start >= 2
            && context.source().as_bytes().get(line_start - 1) == Some(&b'\n')
            && context.source().as_bytes().get(line_start - 2) == Some(&b'\n');
        let removal_start = line_start - usize::from(preceded_by_blank);
        let removal_end = *end + usize::from(context.source().as_bytes().get(*end) == Some(&b'\n'));
        edits.push((removal_start..removal_end, String::new()));
    }
    for (start, end, _, _) in group.iter() {
        context.replace_many(
            format!("Group together all `{accessor}` attributes."),
            *start..*end,
            edits.clone(),
        );
    }
    group.clear();
}

fn grouped_accessors(context: &mut CopContext<'_, '_>, lines: &[(usize, &str)]) {
    let mut groups: [Vec<AccessorLine>; 18] = Default::default();
    let mut visibility = 0usize;
    let mut eigenclass = 0usize;
    for (index, (offset, line)) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        let accessor = ["attr_reader", "attr_writer", "attr_accessor"]
            .into_iter()
            .find(|accessor| {
                trimmed.starts_with(*accessor)
                    && trimmed
                        .as_bytes()
                        .get(accessor.len())
                        .is_some_and(|byte| matches!(byte, b' ' | b'('))
            });
        if let Some(accessor) = accessor {
            let group_index = eigenclass * 9
                + visibility * 3
                + match accessor {
                    "attr_reader" => 0,
                    "attr_writer" => 1,
                    _ => 2,
                };
            let typed_or_commented = lines[..index]
                .iter()
                .rev()
                .find(|(_, previous)| !previous.trim().is_empty())
                .is_some_and(|(_, previous)| {
                    let previous = previous.trim_start();
                    previous.starts_with('#')
                        || previous.starts_with("sig ")
                        || previous.starts_with("annotation_method ")
                });
            if typed_or_commented {
                flush_accessor_group(&mut groups[group_index], context);
                groups[group_index].push(accessor_line(*offset, line, trimmed, accessor, false));
                flush_accessor_group(&mut groups[group_index], context);
                continue;
            }
            if trimmed
                .split_once('#')
                .is_some_and(|(_, comment)| comment.trim_start().starts_with(':'))
            {
                groups[group_index].push(accessor_line(*offset, line, trimmed, accessor, false));
                flush_accessor_group(&mut groups[group_index], context);
                continue;
            }
            groups[group_index].push(accessor_line(*offset, line, trimmed, accessor, true));
        } else if matches!(trimmed.trim(), "private" | "protected" | "public") {
            visibility = match trimmed.trim() {
                "protected" => 1,
                "private" => 2,
                _ => 0,
            };
        } else if trimmed.trim() == "class << self" {
            eigenclass = 1;
            visibility = 0;
        } else if trimmed.trim() == "end" && eigenclass == 1 {
            eigenclass = 0;
            visibility = 0;
        }
    }
    for group in &mut groups {
        flush_accessor_group(group, context);
    }
}
