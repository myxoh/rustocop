use super::helpers::*;
use super::{push_offense, CorrectionStatus, Offense, SourceLine};
use crate::config::InspectionConfig;

pub(super) fn check(lines: &[SourceLine], options: &InspectionConfig, offenses: &mut Vec<Offense>) {
    check_endless_method(lines, options, offenses);
    check_documentation(lines, options, offenses);
    check_numbered_parameters(lines, options, offenses);
}

fn check_endless_method(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/EndlessMethod";
    if !options.cop_enabled(cop) {
        return;
    }
    let style = options
        .cop_config
        .value(cop, "EnforcedStyle")
        .unwrap_or("allow_single_line");
    if style == "allow_always" {
        return;
    }
    for (index, line) in lines.iter().enumerate() {
        let Some(def_column) = line.body.find("def ") else {
            continue;
        };
        let definition = &line.body[def_column..];
        let endless = definition.contains(" = ");
        if endless {
            if !matches!(
                style,
                "disallow" | "allow_single_line" | "require_single_line"
            ) {
                continue;
            }
            let last = endless_expression_end(lines, index);
            let multiline = last > index;
            if !multiline && style != "disallow" {
                continue;
            }
            let message = if multiline {
                "Avoid endless method definitions with multiple lines."
            } else {
                "Avoid endless method definitions."
            };
            push_multiline_offense(offenses, cop, message, index, def_column, last, lines);
            continue;
        }
        if !matches!(style, "require_single_line" | "require_always") {
            continue;
        }
        let Some(end) = regular_method_end(lines, index, def_column) else {
            continue;
        };
        let body = &lines[index + 1..end];
        let meaningful = body
            .iter()
            .filter(|line| !line.body.trim().is_empty() && !line.body.trim_start().starts_with('#'))
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
                .any(|line| line.body.contains("<<") || line.body.trim() == "begin")
        {
            continue;
        }
        let expressions = meaningful
            .iter()
            .filter(|line| !line.body.trim_start().starts_with('.'))
            .count();
        if expressions != 1 || style == "require_single_line" && meaningful.len() != 1 {
            continue;
        }
        let signature = definition.trim();
        let body_source = meaningful
            .iter()
            .map(|line| line.body.trim())
            .collect::<Vec<_>>()
            .join(" ");
        let proposed = format!("{signature} = {body_source}");
        let line_length_enabled =
            options.cop_config.value("Layout/LineLength", "Enabled") != Some("false");
        let max = options
            .cop_config
            .value("Layout/LineLength", "Max")
            .and_then(|max| max.parse::<usize>().ok())
            .unwrap_or(120);
        if line_length_enabled && def_column + proposed.chars().count() > max {
            continue;
        }
        let message = if style == "require_always" {
            "Use endless method definitions."
        } else {
            "Use endless method definitions for single line methods."
        };
        push_multiline_offense(offenses, cop, message, index, def_column, end, lines);
    }
}

fn endless_expression_end(lines: &[SourceLine], index: usize) -> usize {
    if lines[index].body.contains(" = begin") {
        return lines[index + 1..]
            .iter()
            .position(|line| line.body.trim() == "end")
            .map_or(index, |at| index + 1 + at);
    }
    let mut last = index;
    for (offset, line) in lines[index + 1..].iter().enumerate() {
        if !line.body.trim_start().starts_with('.') {
            break;
        }
        last = index + 1 + offset;
    }
    last
}

fn regular_method_end(lines: &[SourceLine], index: usize, def_column: usize) -> Option<usize> {
    lines[index + 1..]
        .iter()
        .position(|line| line.body.trim() == "end" && leading_spaces(&line.body) <= def_column)
        .map(|at| index + 1 + at)
}

fn push_multiline_offense(
    offenses: &mut Vec<Offense>,
    cop: &'static str,
    message: &str,
    first: usize,
    column: usize,
    last: usize,
    lines: &[SourceLine],
) {
    offenses.push(Offense {
        cop_name: cop.to_string(),
        message: message.to_string(),
        corrected: false,
        correctable: true,
        line: first + 1,
        column: column + 1,
        last_line: last + 1,
        last_column: lines[last].body.len().max(1),
        length: lines[first..=last]
            .iter()
            .map(|line| line.body.len() + line.ending.len())
            .sum::<usize>()
            .max(1),
    });
}

fn check_documentation(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/Documentation";
    if !options.cop_enabled(cop) {
        return;
    }
    let allowed = options.cop_config.values(cop, "AllowedConstants");
    let mut namespaces = Vec::<(usize, String, bool)>::new();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.body.trim();
        if !(trimmed.starts_with("class ") || trimmed.starts_with("module ")) {
            continue;
        }
        let indent = leading_spaces(&line.body);
        namespaces.retain(|(namespace_indent, _, _)| *namespace_indent < indent);
        let (kind, rest) = trimmed.split_once(' ').expect("declaration has keyword");
        let name = rest
            .split(|character: char| {
                character.is_whitespace() || matches!(character, '<' | ';' | '#')
            })
            .next()
            .unwrap_or_default();
        let inherited_nodoc = namespaces.iter().any(|(_, _, nodoc)| *nodoc);
        let nodoc = trimmed.contains(":nodoc:");
        let nodoc_all = inherited_nodoc || trimmed.contains(":nodoc: all");
        let full_name = if name.starts_with("::") || namespaces.is_empty() {
            name.to_string()
        } else {
            format!(
                "{}::{name}",
                namespaces
                    .last()
                    .map(|(_, name, _)| name.as_str())
                    .unwrap_or_default()
            )
        };
        namespaces.push((indent, full_name.clone(), nodoc_all));
        let documented = contiguous_documentation(lines, index)
            || rest
                .find('#')
                .is_some_and(|comment| !rest[comment..].contains(":nodoc:"));
        let private = lines[index + 1..].iter().any(|candidate| {
            candidate.body.trim().starts_with("private_constant")
                && candidate
                    .body
                    .split(':')
                    .nth(1)
                    .is_some_and(|constant| constant.trim() == name)
        });
        if !documented
            && !nodoc
            && !inherited_nodoc
            && !private
            && !allowed.iter().any(|allowed| allowed == name)
            && (declaration_substantial(lines, index)
                || kind == "module" && declaration_empty(lines, index))
        {
            push_offense(
                offenses,
                cop,
                &format!("Missing top-level documentation comment for `{kind} {full_name}`."),
                index + 1,
                indent + 1,
                kind.len() + 1 + name.len(),
                CorrectionStatus::Unavailable,
            );
        }
    }
}

fn contiguous_documentation(lines: &[SourceLine], index: usize) -> bool {
    if index == 0
        || lines[index - 1].body.trim().is_empty()
        || !lines[index - 1].body.trim_start().starts_with('#')
    {
        return false;
    }
    lines[..index]
        .iter()
        .rev()
        .take_while(|line| line.body.trim_start().starts_with('#'))
        .any(|line| documentation_comment(&line.body))
}

fn documentation_comment(line: &str) -> bool {
    let comment = line
        .trim_start()
        .strip_prefix('#')
        .map(str::trim)
        .unwrap_or_default();
    !comment.is_empty()
        && ![
            "TODO",
            "FIXME",
            "OPTIMIZE",
            "HACK",
            "rubocop:",
            "frozen_string_literal:",
            "encoding:",
        ]
        .iter()
        .any(|marker| comment.starts_with(marker))
}

fn declaration_substantial(lines: &[SourceLine], index: usize) -> bool {
    let declaration = lines[index].body.trim();
    if let Some((_, tail)) = declaration.split_once(';') {
        return tail.trim() != "end";
    }
    let base_indent = leading_spaces(&lines[index].body);
    let mut child_indent = None;
    for candidate in &lines[index + 1..] {
        let trimmed = candidate.body.trim();
        let indent = leading_spaces(&candidate.body);
        if trimmed.starts_with("end") && indent <= base_indent {
            break;
        }
        if indent <= base_indent || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let direct_indent = *child_indent.get_or_insert(indent);
        if indent > direct_indent || trimmed.starts_with("end") {
            continue;
        }
        if trimmed.starts_with("class ")
            || trimmed.starts_with("module ")
            || trimmed.starts_with("private_constant")
            || trimmed.starts_with("include ")
            || trimmed.starts_with("extend ")
            || trimmed.starts_with("prepend ")
            || trimmed.split_once('=').is_some_and(|(left, _)| {
                left.trim()
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
            })
        {
            continue;
        }
        return true;
    }
    false
}

fn declaration_empty(lines: &[SourceLine], index: usize) -> bool {
    lines[index + 1..]
        .iter()
        .find(|line| !line.body.trim().is_empty())
        .is_some_and(|line| line.body.trim().starts_with("end"))
}

fn check_numbered_parameters(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/NumberedParameters";
    if !options.cop_enabled(cop) {
        return;
    }
    for (index, line) in lines.iter().enumerate() {
        if let Some(column) = find_numbered_parameter(&line.body) {
            push_offense(
                offenses,
                cop,
                "Avoid using numbered parameters.",
                index + 1,
                column,
                2,
                CorrectionStatus::Unavailable,
            );
        }
    }
}
