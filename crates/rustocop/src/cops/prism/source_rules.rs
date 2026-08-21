use std::collections::HashSet;

use super::*;

mod project_files;
use project_files::*;

declare_source_cops! {
    SymbolLiteral => "Style/SymbolLiteral" => symbol_literals,
    AddRuntimeDependency => "Gemspec/AddRuntimeDependency" => add_runtime_dependency,
    ArrayIntersect => "Style/ArrayIntersectWithSingleElement" => array_intersect,
    WhenThen => "Style/WhenThen" => when_then,
    ClassAndModuleCamelCase => "Naming/ClassAndModuleCamelCase" => camel_case,
    ArrayCoercion => "Style/ArrayCoercion" => array_coercion,
    ClassMethods => "Style/ClassMethods" => class_methods,
    RedundantCapitalW => "Style/RedundantCapitalW" => redundant_capital_w,
    DuplicateElsifCondition => "Lint/DuplicateElsifCondition" => duplicate_elsif,
    EnsureReturn => "Lint/EnsureReturn" => ensure_return,
    ClassVars => "Style/ClassVars" => class_vars,
    DuplicatedGem => "Bundler/DuplicatedGem" => duplicated_gem,
    StringHashKeys => "Style/StringHashKeys" => string_hash_keys,
}

fn symbol_literals(source: &str, context: &mut Reporter<'_>) {
    for start in find_all(source, ":\"") {
        let Some(relative_end) = source[start + 2..].find('"') else {
            continue;
        };
        let end = start + 2 + relative_end + 1;
        let word = &source[start + 2..end - 1];
        if !word.is_empty()
            && word.as_bytes()[0].is_ascii_alphabetic()
            && word
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            context.replace(
                "Do not use strings for word-like symbol literals.",
                start..end,
                start..end,
                format!(":{word}"),
            );
        }
    }
}

fn add_runtime_dependency(source: &str, context: &mut Reporter<'_>) {
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

fn array_intersect(source: &str, context: &mut Reporter<'_>) {
    for start in find_all(source, ".intersect?(") {
        if start > 0 && source.as_bytes()[start - 1] == b'&' {
            continue;
        }
        let argument_start = start + ".intersect?(".len();
        let Some(relative_end) = source[argument_start..].find(')') else {
            continue;
        };
        let end = argument_start + relative_end + 1;
        let argument = &source[argument_start..end - 1];
        let element = if argument.starts_with('[')
            && argument.ends_with(']')
            && !argument[1..argument.len() - 1].contains(',')
        {
            Some(argument[1..argument.len() - 1].to_string())
        } else if argument.starts_with("%i[")
            && argument.ends_with(']')
            && !argument[3..argument.len() - 1].contains(' ')
        {
            Some(format!(":{}", &argument[3..argument.len() - 1]))
        } else {
            None
        };
        if let Some(element) = element {
            context.replace(
                "Use `include?(element)` instead of `intersect?([element])`.",
                start + 1..end,
                start..end,
                format!(".include?({element})"),
            );
        }
    }
}

fn when_then(source: &str, context: &mut Reporter<'_>) {
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("when ") {
            continue;
        }
        let Some(semi) = line.find(';') else { continue };
        if line[..semi].contains(" then ") {
            continue;
        }
        let prefix = line[..=semi].trim_start();
        context.replace(
            format!(
                "Do not use `{prefix}`. Use `{} then` instead.",
                prefix.trim_end_matches(';')
            ),
            offset + semi..offset + semi + 1,
            offset + semi..offset + semi + 1,
            " then",
        );
    }
}

fn camel_case(source: &str, context: &mut Reporter<'_>) {
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

fn array_coercion(source: &str, context: &mut Reporter<'_>) {
    for start in find_all(source, "[*") {
        let Some(end_rel) = source[start + 2..].find(']') else {
            continue;
        };
        let end = start + 2 + end_rel + 1;
        let value = &source[start + 2..end - 1];
        if !value.contains(',') {
            context.replace(
                format!("Use `Array({value})` instead of `[*{value}]`."),
                start..end,
                start..end,
                format!("Array({value})"),
            );
        }
    }
    for (offset, line) in source_lines(source) {
        let Some((left, rest)) = line.split_once(" = [") else {
            continue;
        };
        let value = left.trim();
        let pattern = format!("] unless {value}.is_a?(Array)");
        if rest == format!("{value}{pattern}") {
            context.replace(
                format!("Use `Array({value})` instead of explicit `Array` check."),
                offset..offset + line.len(),
                offset..offset + line.len(),
                format!("{value} = Array({value})"),
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
        let Some(relative_end) = source[start + 3..].find(close) else {
            continue;
        };
        let end = start + 3 + relative_end + 1;
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

fn duplicate_elsif(source: &str, context: &mut Reporter<'_>) {
    let mut seen = HashSet::new();
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim_start();
        if let Some(condition) = trimmed.strip_prefix("if ") {
            seen.clear();
            seen.insert(condition.to_string());
        } else if let Some(condition) = trimmed.strip_prefix("elsif ") {
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

fn class_vars(source: &str, context: &mut Reporter<'_>) {
    for start in find_all(source, "@@") {
        let end = start
            + source[start..]
                .bytes()
                .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'@' || *byte == b'_')
                .count();
        let prefix = &source[..start];
        let assignment = source[end..].trim_start().starts_with('=');
        let setter = prefix.ends_with(':') && prefix.rsplit_once("class_variable_set(").is_some();
        if assignment || setter {
            let offense_start = if setter { start - 1 } else { start };
            let name = &source[offense_start..end];
            context.report(
                format!("Replace class var {name} with a class instance var."),
                offense_start..end,
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
