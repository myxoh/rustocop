use super::*;
use crate::rubocop::ast::node::core::NodeRef as RubocopNodeRef;
use crate::rubocop::ast::prism::convert as convert_rubocop_ast;

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
        .or_else(|| {
            group.iter().rposition(|(_, end, _, _)| {
                let line_end = context.source_file().line_end(*end);
                context.source()[*end..line_end].contains('#')
            })
        })
        .unwrap_or(0);
    let first_start = group[anchor].0;
    let first_end = context.source_file().line_end(group[anchor].0);
    let trailing_comment = group.iter().rev().find_map(|(_, end, _, _)| {
        let line_end = context.source_file().line_end(*end);
        let suffix = &context.source()[*end..line_end];
        suffix.contains('#').then_some(suffix)
    });
    let mut edits = vec![(
        first_start..first_end,
        format!(
            "{accessor} {}{}",
            attributes.join(", "),
            trailing_comment.unwrap_or_default()
        ),
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
        let line_end = context.source_file().line_end(*end);
        let removal_end = line_end
            + usize::from(context.source().as_bytes().get(line_end) == Some(&b'\n'));
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
    let parsed = ruby_prism::parse(context.source().as_bytes());
    let (ast, root) = convert_rubocop_ast(context.source(), &parsed.node());
    let Some(root) = root.map(|root| ast.node(root)) else { return };
    for scope in root.each_node(&["class", "module", "sclass"]) {
        let Some(body) = scope.body() else { continue };
        let sends = if body.kind() == "begin" {
            body.each_child_node(&["send"])
        } else if body.kind() == "send" {
            vec![body]
        } else {
            Vec::new()
        };
        for accessor in ["attr_reader", "attr_writer", "attr_accessor", "attr"] {
            for visibility in 0..=2 {
                let group = sends
                    .iter()
                    .copied()
                    .filter(|node| {
                        node.attribute_accessor()
                            && node.method_name() == Some(accessor)
                            && accessor_visibility(*node) == visibility
                            && accessor_groupable(*node, context.source())
                            && !accessor_previous_line_comment(*node, lines)
                    })
                    .collect::<Vec<_>>();
                if group.len() < 2 {
                    continue;
                }
                let mut entries = group
                    .into_iter()
                    .filter_map(|node| {
                        let (offset, line) = *lines.get(node.first_line().checked_sub(1)?)?;
                        Some(accessor_line(
                            offset,
                            line,
                            line.trim_start(),
                            accessor,
                            true,
                        ))
                    })
                    .collect::<Vec<AccessorLine>>();
                flush_accessor_group(&mut entries, context);
            }
        }
    }
}

fn accessor_previous_line_comment(node: RubocopNodeRef<'_>, lines: &[(usize, &str)]) -> bool {
    node.first_line() >= 2
        && lines
            .get(node.first_line() - 2)
            .is_some_and(|(_, line)| line.trim_start().starts_with('#'))
}

fn accessor_visibility(node: RubocopNodeRef<'_>) -> u8 {
    node.left_siblings()
        .into_iter()
        .rev()
        .find_map(|sibling| {
            (sibling.kind() == "send"
                && sibling.receiver().is_none()
                && sibling.arguments().is_empty())
            .then(|| match sibling.method_name() {
                Some("protected") => Some(1),
                Some("private") => Some(2),
                Some("public") => Some(0),
                _ => None,
            })
            .flatten()
        })
        .unwrap_or(0)
}

fn accessor_groupable(node: RubocopNodeRef<'_>, source: &str) -> bool {
    let Some(mut previous) = node.left_siblings().into_iter().last() else {
        return true;
    };
    if matches!(previous.kind(), "block" | "numblock" | "itblock") {
        if let Some(send) = previous
            .child_nodes()
            .into_iter()
            .find(|child| child.kind() == "send")
        {
            previous = send;
        }
    }
    if previous.kind() != "send" {
        return true;
    }
    if previous.source_range().is_some_and(|range| {
        let line_end = source[range.end..]
            .find('\n')
            .map_or(source.len(), |end| range.end + end);
        source[range.end..line_end]
            .trim_start()
            .starts_with("#:")
    }) {
        return false;
    }
    previous.attribute_accessor()
        || (previous.receiver().is_none()
            && previous.arguments().is_empty()
            && matches!(
                previous.method_name(),
                Some("public" | "protected" | "private" | "module_function")
            ))
        || node.first_line().saturating_sub(previous.last_line()) > 1
}

#[allow(dead_code)]
fn grouped_accessors_legacy(context: &mut CopContext<'_, '_>, lines: &[(usize, &str)]) {
    let mut groups = std::collections::BTreeMap::<
        (usize, usize, usize, &'static str),
        Vec<AccessorLine>,
    >::new();
    let mut visibility = 0usize;
    let mut eigenclass = 0usize;
    let mut scope_sequence = 0usize;
    let mut scopes = Vec::<(usize, usize)>::new();
    for (index, (offset, line)) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        let indentation = line.len() - trimmed.len();
        if (trimmed.starts_with("class ") || trimmed.starts_with("module "))
            && !trimmed.contains("; end")
        {
            while scopes.last().is_some_and(|(scope_indent, _)| *scope_indent >= indentation) {
                scopes.pop();
            }
            scope_sequence += 1;
            scopes.push((indentation, scope_sequence));
            visibility = 0;
            eigenclass = usize::from(trimmed.trim() == "class << self");
            continue;
        }
        if trimmed.trim() == "end"
            && scopes.last().is_some_and(|(scope_indent, _)| *scope_indent == indentation)
        {
            scopes.pop();
            visibility = 0;
            eigenclass = 0;
            continue;
        }
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
            let enclosing_line = lines[..index].iter().rev().find(|(_, previous)| {
                !previous.trim().is_empty()
                    && previous.len() - previous.trim_start().len() < indentation
            });
            if enclosing_line.is_some_and(|(_, previous)| {
                let previous = previous.trim_start();
                !previous.starts_with("class ")
                    && !previous.starts_with("module ")
                    && previous.trim() != "class << self"
                    && previous.trim() != "end"
                    && !matches!(previous.trim(), "private" | "protected" | "public")
            }) {
                continue;
            }
            let group = groups
                .entry((scopes.last().map_or(0, |(_, id)| *id), eigenclass, visibility, accessor))
                .or_default();
            let immediately_commented =
                index > 0 && lines[index - 1].1.trim_start().starts_with('#');
            let annotated = lines[..index]
                .iter()
                .rev()
                .find(|(_, previous)| !previous.trim().is_empty())
                .is_some_and(|(_, previous)| {
                    let previous = previous.trim_start();
                    previous.starts_with("sig ") || previous.starts_with("annotation_method ")
                });
            let typed_or_commented = immediately_commented || annotated;
            if typed_or_commented {
                continue;
            }
            if index > 0 {
                let previous = lines[index - 1].1.trim_start();
                let previous_is_accessor = ["attr_reader", "attr_writer", "attr_accessor"]
                    .iter()
                    .any(|name| previous.starts_with(name));
                if !lines[index - 1].1.trim().is_empty()
                    && !previous.starts_with('#')
                    && !previous_is_accessor
                    && !previous.starts_with("class ")
                    && !previous.starts_with("module ")
                    && previous.trim() != "class << self"
                    && previous.trim() != "end"
                    && !matches!(previous.trim(), "private" | "protected" | "public")
                {
                    continue;
                }
            }
            if trimmed
                .split_once('#')
                .is_some_and(|(_, comment)| comment.starts_with(':'))
            {
                continue;
            }
            group.push(accessor_line(*offset, line, trimmed, accessor, true));
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
    for group in groups.values_mut() {
        flush_accessor_group(group, context);
    }
}
