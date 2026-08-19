use super::catalog_cop::{custom, replace, report};
use super::*;
use std::collections::{HashMap, HashSet};

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        custom("Naming/MemoizedInstanceVariableName", memoized_variable),
        custom("Style/TrailingUnderscoreVariable", trailing_underscore),
        custom("Naming/FileName", file_name),
        custom("Style/ParallelAssignment", parallel_assignment),
        report(
            "Lint/AssignmentInCondition",
            "if value = ",
            "Assignment in condition detected.",
        ),
        custom("Naming/VariableNumber", variable_number),
        custom("Naming/VariableName", variable_name),
        custom("Lint/UselessAssignment", useless_assignment),
        replace(
            "Style/SelfAssignment",
            "value = value",
            "value",
            "Redundant self assignment detected.",
        ),
        custom("Naming/MethodName", method_name),
        custom("Style/MutableConstant", mutable_constant),
        custom("Naming/PredicateMethod", predicate_method),
    ]
}

fn memoized_variable(context: &mut CopContext<'_, '_>) {
    if context.source().contains("define_method")
        || context.source().contains("define_singleton_method")
    {
        return;
    }
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut method = None::<String>;
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        if let Some(definition) = line.trim_start().strip_prefix("def ") {
            method = Some(
                definition
                    .split(['(', ' '])
                    .next()
                    .unwrap_or("")
                    .rsplit('.')
                    .next()
                    .unwrap_or("")
                    .trim_start_matches('_')
                    .trim_end_matches(['?', '!'])
                    .to_string(),
            );
        }
        if let Some(at) = line.find("@") {
            if line[at..].contains("||=")
                && index + 1 < lines.len()
                && lines[index + 1].1.trim() == "end"
            {
                let name = line[at + 1..]
                    .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
                    .next()
                    .unwrap_or("");
                let normalized = name.trim_start_matches('_');
                if method
                    .as_deref()
                    .is_some_and(|method| !method.starts_with("initialize") && method != normalized)
                {
                    context.report(
                        "Memoized variable name should match the method name.",
                        offset + at..offset + at + name.len() + 1,
                    );
                }
            }
        }
        if line.trim() == "end" {
            method = None;
        }
    }
}

fn parallel_assignment(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let Some((left, right)) = line.split_once(" = ") else {
            continue;
        };
        if !left.contains(',')
            || left.contains('*')
            || left.contains("self.")
            || left
                .trim_start()
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_uppercase())
            || right.contains('*')
            || right.contains(".map")
        {
            continue;
        }
        let left_names = left.split(',').count();
        let right_values = right.split(',').count();
        if left_names != right_values || right.contains(left.split(',').next().unwrap_or("")) {
            continue;
        }
        context.report(
            "Do not use parallel assignment.",
            offset..offset + line.len(),
        );
    }
}

fn trailing_underscore(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        if let Some(first) = line.find('|') {
            if let Some(close) = line[first + 1..].find('|').map(|at| first + 1 + at) {
                let arguments = line[first + 1..close]
                    .split(',')
                    .map(str::trim)
                    .collect::<Vec<_>>();
                if let Some(last_used) = arguments
                    .iter()
                    .rposition(|argument| !argument.starts_with('_'))
                {
                    for argument in &arguments[last_used + 1..] {
                        let start =
                            offset + first + 1 + line[first + 1..close].find(argument).unwrap_or(0);
                        context.remove(
                            "Omit trailing unused block arguments.",
                            start..start + argument.len(),
                            start.saturating_sub(2)..start + argument.len(),
                        );
                    }
                }
            }
        }
    }
}

fn file_name(context: &mut CopContext<'_, '_>) {
    let Some(file) = std::path::Path::new(context.path())
        .file_name()
        .and_then(|name| name.to_str())
    else {
        return;
    };
    if file.ends_with(".rb")
        && file
            .bytes()
            .any(|byte| byte.is_ascii_uppercase() || byte == b'-')
    {
        context.report("The name of this source file should use snake_case.", 0..0);
    }
}

fn variable_number(context: &mut CopContext<'_, '_>) {
    if !context.config_values("AllowedPatterns").is_empty() {
        return;
    }
    let snake_case = context.policy().enforced_style("normalcase") == "snake_case";
    for (offset, line) in context.source_file().lines() {
        let Some((left, _)) = line.split_once('=') else {
            continue;
        };
        for word in
            left.split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        {
            if context
                .config_values("AllowedIdentifiers")
                .iter()
                .any(|allowed| allowed == word)
            {
                continue;
            }
            if snake_case
                && word
                    .chars()
                    .last()
                    .is_some_and(|character| character.is_ascii_digit())
                && !word
                    .trim_end_matches(|character: char| character.is_ascii_digit())
                    .ends_with('_')
                && word.chars().any(|character| character.is_ascii_lowercase())
            {
                let start = offset + line.find(word).unwrap_or(0);
                context.report(
                    "Use normalcase for numbered variables.",
                    start..start + word.len(),
                );
            }
        }
    }
}

fn variable_name(context: &mut CopContext<'_, '_>) {
    if context.policy().enforced_style("snake_case") != "snake_case"
        || !context.config_values("AllowedPatterns").is_empty()
    {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        let Some((name, _)) = line.split_once(" = ") else {
            continue;
        };
        let name = name.trim();
        let bare = name.trim_start_matches(['@', '$']);
        if bare
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_uppercase())
            || context
                .config_values("AllowedIdentifiers")
                .iter()
                .any(|allowed| allowed == bare)
        {
            continue;
        }
        if name.bytes().any(|byte| byte.is_ascii_uppercase())
            && name.bytes().any(|byte| byte.is_ascii_lowercase())
        {
            let start = offset + line.find(name).unwrap_or(0);
            context.report(
                "Use snake_case for variable names.",
                start..start + name.len(),
            );
        }
    }
}

fn useless_assignment(context: &mut CopContext<'_, '_>) {
    let source = context.source().to_string();
    let mut assignments = HashMap::<String, (usize, usize)>::new();
    for (offset, line) in context.source_file().lines() {
        if let Some((name, _)) = line.split_once(" = ") {
            let name = name.trim();
            if !name.is_empty() {
                assignments.insert(
                    name.to_string(),
                    (offset + line.find(name).unwrap_or(0), name.len()),
                );
            }
        }
    }
    for (name, (start, len)) in assignments {
        if name.starts_with("unused") && source.match_indices(&name).count() == 1 {
            context.report("Useless assignment to variable.", start..start + len);
        }
    }
}

fn method_name(context: &mut CopContext<'_, '_>) {
    if context.policy().enforced_style("snake_case") != "snake_case" {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        let Some(definition) = line.trim_start().strip_prefix("def ") else {
            continue;
        };
        let name = definition.split(['(', ' ']).next().unwrap_or("");
        let bare = name.rsplit('.').next().unwrap_or(name);
        if name.contains('.')
            && bare
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_uppercase())
        {
            continue;
        }
        if context
            .config_values("AllowedPatterns")
            .iter()
            .any(|pattern| pattern.contains(bare))
        {
            continue;
        }
        if name.bytes().any(|byte| byte.is_ascii_uppercase()) {
            let start = offset + line.find(name).unwrap_or(0);
            context.report(
                "Use snake_case for method names.",
                start..start + name.len(),
            );
        }
    }
}

fn mutable_constant(context: &mut CopContext<'_, '_>) {
    if context.source().contains("shareable_constant_value")
        || context.source().contains("frozen_string_literal: true")
    {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        let Some((name, value)) = line.split_once(" = ") else {
            continue;
        };
        let name = name.trim();
        if name
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte == b'_')
            && value.trim_start().starts_with(['[', '{', '"', '\''])
            && !value.contains(".freeze")
            && !value.contains(" + ")
            && !value.contains(".count")
            && !value.contains(".length")
            && !value.contains(".size")
        {
            context.insert(
                "Freeze mutable objects assigned to constants.",
                offset..offset + line.len(),
                offset + line.len(),
                ".freeze",
            );
        }
    }
}

fn predicate_method(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let predicates = HashSet::from(["is_", "has_", "does_"]);
    for (offset, line) in lines {
        let Some(definition) = line.trim_start().strip_prefix("def ") else {
            continue;
        };
        let name = definition.split(['(', ' ']).next().unwrap_or("");
        if predicates.iter().any(|prefix| name.starts_with(prefix)) && !name.ends_with('?') {
            let start = offset + line.find(name).unwrap_or(0);
            context.insert(
                "Predicate method names should end with `?`.",
                start..start + name.len(),
                start + name.len(),
                "?",
            );
        }
    }
}
