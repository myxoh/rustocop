use std::collections::HashSet;

use super::*;

mod project_files;
use project_files::*;

declare_source_cops! {
    AddRuntimeDependency => "Gemspec/AddRuntimeDependency" => add_runtime_dependency,
    ClassAndModuleCamelCase => "Naming/ClassAndModuleCamelCase" => camel_case,
    ClassMethods => "Style/ClassMethods" => class_methods,
    RedundantCapitalW => "Style/RedundantCapitalW" => redundant_capital_w,
    DuplicateElsifCondition => "Lint/DuplicateElsifCondition" => duplicate_elsif,
    EnsureReturn => "Lint/EnsureReturn" => ensure_return,
    DuplicatedGem => "Bundler/DuplicatedGem" => duplicated_gem,
}

fn add_runtime_dependency(source: &str, context: &mut Reporter<'_>) {
    if !context.path().ends_with("(string)") && !context.path().ends_with(".gemspec") {
        return;
    }
    const METHOD: &str = "add_runtime_dependency";
    for (offset, line) in source_lines(source) {
        let Some(dot) = line.find(&format!(".{METHOD}")) else {
            continue;
        };
        let start = dot + 1;
        let end = start + METHOD.len();
        let after = line[end..].trim_start();
        let has_argument = if let Some(arguments) = after.strip_prefix('(') {
            arguments
                .split_once(')')
                .is_some_and(|(arguments, _)| !arguments.trim().is_empty())
        } else {
            !after.is_empty() && !after.starts_with('#')
        };
        if has_argument {
            context.replace(
                "Use `add_dependency` instead of `add_runtime_dependency`.",
                offset + start..offset + end,
                offset + start..offset + end,
                "add_dependency",
            );
        }
    }
}

fn camel_case(source: &str, context: &mut Reporter<'_>) {
    let file = SourceFile::new(source);
    let class_offsets = file.code_offsets("class");
    let module_offsets = file.code_offsets("module");
    let heredocs = file.heredoc_ranges();
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim_start();
        let keyword = if trimmed.starts_with("class ") {
            "class "
        } else if trimmed.starts_with("module ") {
            "module "
        } else {
            continue;
        };
        let leading = line.len() - trimmed.len();
        let lexical = if keyword == "class " {
            &class_offsets
        } else {
            &module_offsets
        };
        let keyword_offset = offset + leading;
        if lexical.binary_search(&keyword_offset).is_err()
            || heredocs
                .iter()
                .any(|heredoc| heredoc.start <= keyword_offset && keyword_offset < heredoc.end)
        {
            continue;
        }
        let name_start = leading + keyword.len();
        let name = line[name_start..]
            .split_whitespace()
            .next()
            .unwrap_or_default();
        let invalid = name.split("::").any(|part| {
            part.contains('_') && !matches!(part, "module_parent" | "getter_class" | "setter_class")
        });
        if invalid {
            context.report(
                "Use CamelCase for classes and modules.",
                offset + name_start..offset + name_start + name.len(),
            );
        }
    }
}

fn class_methods(source: &str, context: &mut Reporter<'_>) {
    let mut owner: Option<String> = None;
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim();
        if let Some(name) = trimmed
            .strip_prefix("class ")
            .or_else(|| trimmed.strip_prefix("module "))
        {
            owner = Some(
                name.split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_string(),
            );
            continue;
        }
        let Some(owner_name) = owner.as_deref() else {
            continue;
        };
        let needle = format!("def {owner_name}.");
        if let Some(start) = line.find(&needle) {
            let receiver = offset + start + 4..offset + start + 4 + owner_name.len();
            let method = line[start + needle.len()..]
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .split('(')
                .next()
                .unwrap_or_default();
            context.replace(
                format!("Use `self.{method}` instead of `{owner_name}.{method}`."),
                receiver.clone(),
                receiver,
                "self",
            );
        }
    }
}

fn redundant_capital_w(source: &str, context: &mut Reporter<'_>) {
    for start in find_all(source, "%W") {
        let Some(open) = source.as_bytes().get(start + 2).copied() else {
            continue;
        };
        let close = match open {
            b'(' => ')',
            b'[' => ']',
            b'{' => '}',
            _ => continue,
        };
        let Some(end) = percent_literal_end(source, start + 2, open, close as u8) else {
            continue;
        };
        let body = &source[start + 3..end - 1];
        if !body.contains("#{") && !body.contains('\\') {
            context.replace(
                "Do not use `%W` unless interpolation is needed. If not, use `%w`.",
                start..end,
                start + 1..start + 2,
                "w",
            );
        }
    }
}

fn percent_literal_end(source: &str, open: usize, left: u8, right: u8) -> Option<usize> {
    let mut depth = 0_usize;
    let mut escaped = false;
    for (index, byte) in source.as_bytes().iter().copied().enumerate().skip(open) {
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == left {
            depth += 1;
        } else if byte == right {
            depth -= 1;
            if depth == 0 {
                return Some(index + 1);
            }
        }
    }
    None
}

fn duplicate_elsif(source: &str, context: &mut Reporter<'_>) {
    let mut seen = HashSet::new();
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim_start();
        if let Some(condition) = trimmed.strip_prefix("if ") {
            seen.clear();
            if !multiline_condition(condition) {
                seen.insert(condition.to_string());
            }
        } else if let Some(condition) = trimmed.strip_prefix("elsif ") {
            if multiline_condition(condition) {
                continue;
            }
            let start = offset + line.len() - trimmed.len() + 6;
            if !seen.insert(condition.to_string()) {
                context.report(
                    "Duplicate `elsif` condition detected.",
                    start..start + condition.len(),
                );
            }
        } else if trimmed == "end" {
            seen.clear();
        }
    }
}

fn multiline_condition(condition: &str) -> bool {
    condition.trim_end().ends_with("&&") || condition.trim_end().ends_with("||")
}

fn ensure_return(source: &str, context: &mut Reporter<'_>) {
    let mut in_ensure = false;
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim();
        if trimmed == "ensure" {
            in_ensure = true;
            continue;
        }
        if trimmed == "end" {
            in_ensure = false;
        }
        if in_ensure && (trimmed == "return" || trimmed.starts_with("return ")) {
            let start = offset + line.len() - line.trim_start().len();
            context.report(
                "Do not return from an `ensure` block.",
                start..offset + line.len(),
            );
        }
    }
}

fn find_all(source: &str, needle: &str) -> Vec<usize> {
    source
        .match_indices(needle)
        .map(|(offset, _)| offset)
        .collect()
}

fn source_lines(source: &str) -> impl Iterator<Item = (usize, &str)> {
    source.split_inclusive('\n').scan(0, |offset, line| {
        let start = *offset;
        *offset += line.len();
        Some((start, line.strip_suffix('\n').unwrap_or(line)))
    })
}
