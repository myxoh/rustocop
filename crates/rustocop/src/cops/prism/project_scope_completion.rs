use std::collections::HashMap;

use super::*;

mod scope_rules;

define_cops! {
    DuplicatedGroup => "Bundler/DuplicatedGroup" => source(duplicated_group),
    DevelopmentDependencies => "Gemspec/DevelopmentDependencies" => source(development_dependencies),
    DeprecatedAttributeAssignment => "Gemspec/DeprecatedAttributeAssignment" => source(deprecated_gemspec_attribute),
    DuplicateMatchPattern => "Lint/DuplicateMatchPattern" => source(duplicate_match_pattern),
    ConstantName => "Naming/ConstantName" => source(constant_name),
    ConstantVisibility => "Style/ConstantVisibility" => source(constant_visibility),
    RedundantSelfAssignment => "Style/RedundantSelfAssignment" => source(scope_rules::redundant_self_assignment),
    TopLevelMethodDefinition => "Style/TopLevelMethodDefinition" => source(scope_rules::top_level_method_definition),
}

fn duplicated_group(context: &mut CopContext<'_, '_>) {
    if !context.path().ends_with("Gemfile") {
        return;
    }
    let mut seen = HashMap::<String, usize>::new();
    let mut scopes = vec![String::new()];
    for (line_number, (offset, line)) in context.source_file().lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.trim() == "end" {
            if scopes.len() > 1 {
                scopes.pop();
            }
            continue;
        }
        let current_scope = scopes.last().cloned().unwrap_or_default();
        let opens_block = trimmed.contains(" do") || trimmed.ends_with("do");
        let source_scope = ["source", "git", "platforms", "path"]
            .into_iter()
            .find(|name| {
                trimmed.starts_with(&format!("{name} "))
                    || trimmed.starts_with(&format!("{name}("))
            })
            .map(|name| {
                let argument = trimmed
                    .strip_prefix(name)
                    .unwrap_or_default()
                    .trim_start_matches('(')
                    .split(" do")
                    .next()
                    .unwrap_or_default()
                    .trim_end_matches(')')
                    .trim();
                format!("{name}{argument}")
            });
        let Some(arguments) = trimmed
            .strip_prefix("group ")
            .or_else(|| trimmed.strip_prefix("group("))
            .and_then(|line| line.split_once(" do"))
            .map(|p| p.0)
        else {
            if opens_block {
                scopes.push(source_scope.unwrap_or(current_scope));
            }
            continue;
        };
        let arguments = arguments.trim_end_matches(')').trim();
        let parts = split_group_arguments(arguments);
        let option_start = parts
            .iter()
            .position(|part| {
                !part.starts_with(':') && !part.starts_with(['\'', '"', '*'])
            })
            .unwrap_or(parts.len());
        let mut attributes = parts[..option_start]
            .iter()
            .map(|part| {
                part.trim()
                    .trim_start_matches(':')
                    .trim_matches(['\'', '"'])
                    .to_string()
            })
            .collect::<Vec<_>>();
        if option_start < parts.len() {
            let mut options = parts[option_start..]
                .iter()
                .map(|part| part.trim().to_string())
                .collect::<Vec<_>>();
            options.sort();
            attributes.push(options.join(", "));
        }
        attributes.sort();
        let identity = format!("{current_scope}|{}", attributes.join(""));
        let display = parts.join(", ");
        if let Some(first) = seen.get(&identity) {
            let indent = line.len() - trimmed.len();
            context.report(
                format!("Gem group `{display}` already defined on line {first} of the Gemfile."),
                offset + indent..offset + line.find(" do").unwrap_or(line.len()),
            );
        } else {
            seen.insert(identity, line_number + 1);
        }
        if opens_block {
            scopes.push(current_scope);
        }
    }
}

fn split_group_arguments(source: &str) -> Vec<&str> {
    let mut arguments = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut quote = None;
    for (index, byte) in source.bytes().enumerate() {
        if let Some(delimiter) = quote {
            if byte == delimiter && source.as_bytes().get(index.wrapping_sub(1)) != Some(&b'\\') {
                quote = None;
            }
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'[' | b'{' | b'(' => depth += 1,
            b']' | b'}' | b')' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                arguments.push(source[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    arguments.push(source[start..].trim());
    arguments
}

fn development_dependencies(context: &mut CopContext<'_, '_>) {
    let style = context.policy().enforced_style("Gemfile").to_string();
    let gemspec = context.path().ends_with(".gemspec");
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        if gemspec && style != "gemspec" && trimmed.contains(".add_development_dependency") {
            let gem = trimmed
                .split("add_development_dependency")
                .nth(1)
                .unwrap_or_default()
                .trim()
                .trim_start_matches('(')
                .split([',', ')'])
                .next()
                .unwrap_or_default()
                .trim()
                .trim_matches(['\'', '"']);
            if context
                .config_values("AllowedGems")
                .iter()
                .any(|allowed| allowed == gem)
            {
                continue;
            }
            let indent = line.len() - trimmed.len();
            context.report(
                format!("Specify development dependencies in {style}."),
                offset + indent..offset + line.len(),
            );
        } else if !gemspec && style == "gemspec" && trimmed.starts_with("gem ") {
            let gem = trimmed
                .strip_prefix("gem ")
                .unwrap_or_default()
                .split([',', ' '])
                .next()
                .unwrap_or_default()
                .trim_matches(['\'', '"']);
            if context
                .config_values("AllowedGems")
                .iter()
                .any(|allowed| allowed == gem)
            {
                continue;
            }
            let indent = line.len() - trimmed.len();
            context.report(
                "Specify development dependencies in gemspec.",
                offset + indent..offset + line.len(),
            );
        }
    }
}

fn deprecated_gemspec_attribute(context: &mut CopContext<'_, '_>) {
    let mut in_specification = false;
    let mut block_variable = String::new();
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        if trimmed.contains("Gem::Specification.new") {
            in_specification = true;
            block_variable = trimmed
                .split('|')
                .nth(1)
                .unwrap_or_default()
                .trim()
                .to_string();
            continue;
        }
        if in_specification && trimmed.trim() == "end" {
            in_specification = false;
            continue;
        }
        if !in_specification {
            continue;
        }
        let Some((left, operator)) = [" += ", " = "].into_iter().find_map(|operator| {
            trimmed
                .split_once(operator)
                .map(|parts| (parts.0, operator))
        })
        else {
            continue;
        };
        if !left.starts_with(&format!("{block_variable}.")) {
            continue;
        }
        let attribute = left.rsplit('.').next().unwrap_or_default();
        if !matches!(
            attribute,
            "date" | "rubygems_version" | "specification_version" | "test_files"
        ) {
            continue;
        }
        // Under RuboCop's Prism parser, only `test_files` is represented in
        // the op-assignment shape accepted by this cop. The other deprecated
        // attributes are checked for ordinary writer assignment only.
        if operator == " += " && attribute != "test_files" {
            continue;
        }
        let indent = line.len() - trimmed.len();
        let line_end = offset
            + line.len()
            + usize::from(context.source().as_bytes().get(offset + line.len()) == Some(&b'\n'));
        context.remove(
            format!("Do not set `{attribute}` in gemspec."),
            offset + indent..offset + line.len(),
            offset..line_end,
        );
    }
}

fn duplicate_match_pattern(context: &mut CopContext<'_, '_>) {
    let mut seen = HashMap::<String, usize>::new();
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        let Some(pattern) = trimmed.strip_prefix("in ") else {
            continue;
        };
        let raw_pattern = pattern.trim();
        let (pattern, guard) = raw_pattern
            .split_once(" if ")
            .map(|(pattern, guard)| (pattern, format!("if {guard}")))
            .or_else(|| {
                raw_pattern
                    .split_once(" unless ")
                    .map(|(pattern, guard)| (pattern, format!("unless {guard}")))
            })
            .unwrap_or((raw_pattern, String::new()));
        let mut alternatives = pattern
            .split('|')
            .map(canonical_pattern)
            .collect::<Vec<_>>();
        alternatives.sort_unstable();
        let identity = format!("{}|{guard}", alternatives.join(" | "));
        if let std::collections::hash_map::Entry::Vacant(entry) = seen.entry(identity) {
            entry.insert(offset);
        } else {
            let start = offset + line.find(pattern).unwrap_or(0);
            context.report(
                "Duplicate `in` pattern detected.",
                start..start + pattern.len(),
            );
        }
    }
}

fn canonical_pattern(pattern: &str) -> String {
    let pattern = pattern.trim();
    if pattern.contains(',') && !pattern.contains(['[', '(']) {
        let mut elements = pattern.split(',').map(str::trim).collect::<Vec<_>>();
        elements.sort_unstable();
        elements.join(", ")
    } else {
        pattern.to_string()
    }
}

fn constant_name(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        if trimmed.contains("||=") {
            continue;
        }
        let Some((left, _)) = trimmed.rsplit_once('=') else {
            continue;
        };
        let rhs = trimmed.rsplit_once('=').map_or("", |(_, rhs)| rhs.trim());
        let static_rhs = rhs.chars().next().is_some_and(|first| {
            first.is_ascii_digit() || matches!(first, '\'' | '"' | '[' | '{' | '%')
        }) || rhs.contains(".freeze")
            || (rhs.starts_with("if ") && rhs.matches(['\'', '"']).count() >= 4);
        if !static_rhs {
            continue;
        }
        let assigned = left.rsplit('=').next().unwrap_or(left);
        for qualified_name in assigned.split(',').map(str::trim) {
            let name = qualified_name.rsplit("::").next().unwrap_or(qualified_name);
            if name.is_empty()
                || !name.chars().next().is_some_and(char::is_uppercase)
                || name.chars().all(|character| {
                    character.is_uppercase() || character.is_numeric() || character == '_'
                })
            {
                continue;
            }
            let start = offset + line.find(name).unwrap_or(0);
            context.report(
                "Use SCREAMING_SNAKE_CASE for constants.",
                start..start + name.len(),
            );
        }
    }
}

fn constant_visibility(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let mut scope_depth = 0usize;
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("class ") || trimmed.starts_with("module ") {
            scope_depth += 1;
            continue;
        }
        if trimmed == "end" {
            scope_depth = scope_depth.saturating_sub(1);
            continue;
        }
        if scope_depth == 0 {
            continue;
        }
        let Some((name, value)) = trimmed.split_once(" = ") else {
            continue;
        };
        if context.config_bool("IgnoreModules", false)
            && ["Class.new", "Module.new", "Struct.new"]
                .iter()
                .any(|constructor| value.starts_with(constructor))
        {
            continue;
        }
        if !name
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_uppercase())
            || ["private_constant", "public_constant"]
                .into_iter()
                .any(|visibility| {
                    source.lines().any(|line| {
                        let line = line.trim();
                        line.starts_with(visibility)
                            && line[visibility.len()..].split(',').any(|argument| {
                                argument
                                    .trim()
                                    .trim_start_matches(':')
                                    .trim_matches(['\'', '"'])
                                    == name
                            })
                    }) || source.contains(&format!("{visibility} :{name}"))
                        || source.contains(&format!("{visibility} '{name}'"))
                        || source.contains(&format!("{visibility} \"{name}\""))
                })
            || source.lines().any(|visibility| {
                let visibility = visibility.trim();
                visibility.starts_with("private_constant *")
                    || visibility.starts_with("public_constant *")
            })
        {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        context.report(
            format!("Explicitly make `{name}` public or private using either `#public_constant` or `#private_constant`."),
            offset + indent..offset + line.len(),
        );
    }
}
