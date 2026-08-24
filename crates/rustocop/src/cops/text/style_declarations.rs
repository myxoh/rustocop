use super::helpers::*;
use super::{push_offense, CorrectionStatus, Offense, SourceLine};
use crate::config::InspectionConfig;

pub(super) fn check(lines: &[SourceLine], options: &InspectionConfig, offenses: &mut Vec<Offense>) {
    check_documentation(lines, options, offenses);
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
        if kind == "class" && rest.trim_start().starts_with("<<") {
            continue;
        }
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
        if trimmed.starts_with("class <<") {
            return true;
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
