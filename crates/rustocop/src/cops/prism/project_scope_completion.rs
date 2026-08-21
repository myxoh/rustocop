use std::collections::{HashMap, HashSet};

use ruby_prism::{CaseMatchNode, Node};

use super::*;

mod scope_rules;

define_cops! {
    DuplicatedGroup => "Bundler/DuplicatedGroup" => source(duplicated_group),
    DevelopmentDependencies => "Gemspec/DevelopmentDependencies" => source(development_dependencies),
    DeprecatedAttributeAssignment => "Gemspec/DeprecatedAttributeAssignment" => source(deprecated_gemspec_attribute),
    DuplicateMatchPattern => "Lint/DuplicateMatchPattern" => rubocop_callbacks(DuplicateMatchPatternRule, [on_case_match]),
    ConstantName => "Naming/ConstantName" => source(constant_name),
    ConstantVisibility => "Style/ConstantVisibility" => source(constant_visibility),
    RedundantSelfAssignment => "Style/RedundantSelfAssignment" => source(scope_rules::redundant_self_assignment),
    TopLevelMethodDefinition => "Style/TopLevelMethodDefinition" => any_node(top_level_method_definition),
}

fn top_level_method_definition(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let method_definition = node.as_def_node().is_some();
    let dynamic_definition = node
        .as_call_node()
        .is_some_and(|call| call.name().as_slice() == b"define_method");
    if !method_definition && !dynamic_definition {
        return;
    }
    if context.ancestors().iter().any(|ancestor| {
        ancestor.as_program_node().is_none() && ancestor.as_statements_node().is_none()
    }) {
        return;
    }
    context.report(
        "Do not define methods at the top-level.",
        &node.location(),
    );
}

fn duplicated_group(context: &mut CopContext<'_, '_>) {
    if !context.path().ends_with("Gemfile") {
        return;
    }
    let mut seen = HashMap::<String, usize>::new();
    for (line_number, (offset, line)) in context.source_file().lines().enumerate() {
        let trimmed = line.trim_start();
        let Some(arguments) = trimmed
            .strip_prefix("group ")
            .and_then(|line| line.split_once(" do"))
            .map(|p| p.0)
        else {
            continue;
        };
        let parts = arguments.split(',').map(str::trim).collect::<Vec<_>>();
        let option_start = parts
            .iter()
            .position(|part| !part.starts_with(':') && !part.starts_with(['\'', '"']))
            .unwrap_or(parts.len());
        let options = parts[option_start..].join(",");
        for group in &parts[..option_start] {
            let display = group.trim();
            let name = display.trim_start_matches(':').trim_matches(['\'', '"']);
            let identity = format!("{name}|{options}");
            if let Some(first) = seen.get(&identity) {
                let indent = line.len() - trimmed.len();
                context.report(
                    format!(
                        "Gem group `{display}` already defined on line {first} of the Gemfile."
                    ),
                    offset + indent..offset + line.find(" do").unwrap_or(line.len()),
                );
            } else {
                seen.insert(identity, line_number + 1);
            }
        }
    }
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
        let Some(left) = [" += ", " = "]
            .into_iter()
            .find_map(|operator| trimmed.split_once(operator).map(|parts| parts.0))
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

impl DuplicateMatchPatternRule<'_, '_, '_> {
    fn on_case_match(&mut self, node: &CaseMatchNode<'_>) {
        let mut seen = HashSet::new();
        for branch in node
            .conditions()
            .iter()
            .filter_map(|condition| condition.as_in_node())
        {
            let (pattern, identity) = match_pattern_identity(branch.pattern(), self.source_file());
            if !seen.insert(identity) {
                self.report("Duplicate `in` pattern detected.", pattern.location());
            }
        }
    }
}

fn match_pattern_identity<'pr>(pattern: Node<'pr>, file: SourceFile<'_>) -> (Node<'pr>, String) {
    if let Some(condition) = pattern.as_if_node() {
        if let Some(body) = only_statement(condition.statements()) {
            let identity = format!(
                "{}if{}",
                canonical_pattern(file.node(&body)),
                file.node(&condition.predicate())
            );
            return (body, identity);
        }
    }
    if let Some(condition) = pattern.as_unless_node() {
        if let Some(body) = only_statement(condition.statements()) {
            let identity = format!(
                "{}unless{}",
                canonical_pattern(file.node(&body)),
                file.node(&condition.predicate())
            );
            return (body, identity);
        }
    }
    let identity = canonical_pattern(file.node(&pattern));
    (pattern, identity)
}

fn canonical_pattern(pattern: &str) -> String {
    let pattern = pattern.trim();
    if pattern.contains('|') {
        let mut alternatives = pattern.split('|').map(str::trim).collect::<Vec<_>>();
        alternatives.sort_unstable();
        alternatives.join(" | ")
    } else if pattern.contains(',') && !pattern.contains(['[', '(']) {
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
