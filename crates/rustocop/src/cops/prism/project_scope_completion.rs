use std::collections::{HashMap, HashSet};

use ruby_prism::{CaseMatchNode, Node};

use super::*;

mod scope_rules;

define_cops! {
    DuplicatedGroup => "Bundler/DuplicatedGroup" => source(duplicated_group),
    DevelopmentDependencies => "Gemspec/DevelopmentDependencies" => source(development_dependencies),
    DeprecatedAttributeAssignment => "Gemspec/DeprecatedAttributeAssignment" => source(deprecated_gemspec_attribute),
    DuplicateMatchPattern => "Lint/DuplicateMatchPattern" => rubocop_callbacks(DuplicateMatchPatternRule, [on_case_match]),
    ConstantName => "Naming/ConstantName" => any_node(constant_name),
    ConstantVisibility => "Style/ConstantVisibility" => source(constant_visibility),
    RedundantSelfAssignment => "Style/RedundantSelfAssignment" => any_node(scope_rules::redundant_self_assignment),
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
    if context.path() != "(string)" && !context.path().ends_with("Gemfile") {
        return;
    }
    let mut seen = HashMap::<String, usize>::new();
    let mut scopes = Vec::<String>::new();
    for (line_number, (offset, line)) in context.source_file().lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.trim() == "end" {
            scopes.pop();
            continue;
        }
        let Some((call, _)) = trimmed.split_once(" do") else { continue };
        let (method, raw_arguments) = if let Some(arguments) = call.strip_prefix("group ") {
            ("group", arguments)
        } else if let Some(arguments) = call
            .strip_prefix("group(")
            .and_then(|arguments| arguments.strip_suffix(')'))
        {
            ("group", arguments)
        } else {
            let Some((method, arguments)) = call.split_once(' ') else { continue };
            if matches!(method, "source" | "git" | "platforms" | "path") {
                scopes.push(format!("{method}:{}", arguments.trim()));
            }
            continue;
        };
        debug_assert_eq!(method, "group");
        let arguments = raw_arguments.trim();
        let parts = arguments.split(',').map(str::trim).collect::<Vec<_>>();
        let option_start = parts
            .iter()
            .position(|part| !part.starts_with(':') && !part.starts_with(['\'', '"']))
            .unwrap_or(parts.len());
        let mut groups = parts[..option_start]
            .iter()
            .map(|group| group.trim_start_matches(':').trim_matches(['\'', '"']))
            .collect::<Vec<_>>();
        groups.sort_unstable();
        let mut options = parts[option_start..].to_vec();
        options.sort_unstable();
        let identity = format!(
            "{}|groups:{}|options:{}",
            scopes.join("/"),
            groups.join(","),
            options.join(",")
        );
        if let Some(first) = seen.get(&identity) {
            let indent = line.len() - trimmed.len();
            context.report(
                format!(
                    "Gem group `{arguments}` already defined on line {first} of the Gemfile."
                ),
                offset + indent..offset + line.find(" do").unwrap_or(line.len()),
            );
        } else {
            seen.insert(identity, line_number + 1);
        }
        scopes.push(format!("group:{arguments}"));
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
    if !context.path().ends_with("(string)") && !context.path().ends_with(".gemspec") {
        return;
    }
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

fn constant_name(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (location, value) = if let Some(write) = node.as_constant_write_node() {
        (write.name_loc(), write.value())
    } else if let Some(write) = node.as_constant_path_write_node() {
        let target = write.target();
        (target.name_loc(), write.value())
    } else if let Some(target) = node.as_constant_target_node() {
        let Some(value) = context.parent().and_then(Node::as_multi_write_node).map(|write| write.value()) else {
            return;
        };
        (target.location(), value)
    } else {
        return;
    };
    if constant_name_allowed_assignment(&value) {
        return;
    }
    let name = context.source_file().at(&location);
    if name.chars().all(|character| {
        character.is_uppercase() || character.is_numeric() || character == '_'
    }) {
        return;
    }
    context.report("Use SCREAMING_SNAKE_CASE for constants.", location);
}

fn constant_name_allowed_assignment(value: &Node<'_>) -> bool {
    if value.as_constant_read_node().is_some()
        || value.as_constant_path_node().is_some()
        || value.as_constant_write_node().is_some()
        || value.as_constant_path_write_node().is_some()
        || value.as_block_node().is_some()
        || value.as_lambda_node().is_some()
    {
        return true;
    }
    if let Some(call) = value.as_call_node() {
        if call.block().is_some() {
            return true;
        }
        if call_name(&call) == b"new"
            && (root_constant(call.receiver(), b"Class") || root_constant(call.receiver(), b"Struct"))
        {
            return true;
        }
        return call.receiver().is_none_or(|receiver| !literal_node(&receiver));
    }
    value.as_if_node().is_some_and(|conditional| {
        let branch_has_constant = |statements: Option<ruby_prism::StatementsNode<'_>>| {
            statements.is_some_and(|statements| {
                statements.body().iter().any(|branch| {
                    branch.as_constant_read_node().is_some()
                        || branch.as_constant_path_node().is_some()
                })
            })
        };
        branch_has_constant(conditional.statements())
            || conditional.subsequent().is_some_and(|subsequent| {
                subsequent
                    .as_else_node()
                    .is_some_and(|branch| branch_has_constant(branch.statements()))
            })
    })
}

fn literal_node(node: &Node<'_>) -> bool {
    if let Some(parentheses) = node.as_parentheses_node() {
        return parentheses.body().is_some_and(|body| {
            body.as_statements_node()
                .is_some_and(|statements| only_statement(Some(statements)).is_some_and(|inner| literal_node(&inner)))
        });
    }
    node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_rational_node().is_some()
        || node.as_imaginary_node().is_some()
        || node.as_string_node().is_some()
        || node.as_interpolated_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_interpolated_symbol_node().is_some()
        || node.as_array_node().is_some()
        || node.as_hash_node().is_some()
        || node.as_range_node().is_some()
        || node.as_regular_expression_node().is_some()
        || node.as_interpolated_regular_expression_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_nil_node().is_some()
}

fn constant_visibility(context: &mut CopContext<'_, '_>) {
    #[derive(Default)]
    struct ClassOrModuleBodies<'pr>(Vec<ruby_prism::StatementsNode<'pr>>);

    impl<'pr> Visit<'pr> for ClassOrModuleBodies<'pr> {
        fn visit_class_node(&mut self, node: &ruby_prism::ClassNode<'pr>) {
            if let Some(body) = node.body().and_then(|body| body.as_statements_node()) {
                self.0.push(body);
            }
            ruby_prism::visit_class_node(self, node);
        }

        fn visit_module_node(&mut self, node: &ruby_prism::ModuleNode<'pr>) {
            if let Some(body) = node.body().and_then(|body| body.as_statements_node()) {
                self.0.push(body);
            }
            ruby_prism::visit_module_node(self, node);
        }
    }

    let parsed = parse(context.source().as_bytes());
    let mut bodies = ClassOrModuleBodies::default();
    bodies.visit(&parsed.node());
    for statements in bodies.0 {
        inspect_constant_visibility_scope(&statements, context);
    }
}

fn inspect_constant_visibility_scope(
    statements: &ruby_prism::StatementsNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let mut declared = HashSet::<String>::new();
    for statement in statements.body().iter() {
        let Some(call) = statement.as_call_node() else {
            continue;
        };
        if !matches!(call_name(&call), b"private_constant" | b"public_constant")
            || call.receiver().is_some()
        {
            continue;
        }
        for argument in call
            .arguments()
            .into_iter()
            .flat_map(|arguments| arguments.arguments().iter())
        {
            collect_constant_visibility_names(&argument, &mut declared);
        }
    }

    for statement in statements.body().iter() {
        let assignment = if let Some(write) = statement.as_constant_write_node() {
            Some((
                String::from_utf8_lossy(write.name().as_slice()).into_owned(),
                write.value(),
            ))
        } else if let Some(write) = statement.as_constant_path_write_node() {
            let target_location = write.target().location();
            let target = context.source_file().at(&target_location);
            Some((
                target.rsplit("::").next().unwrap_or(target).to_string(),
                write.value(),
            ))
        } else {
            None
        };
        let Some((name, value)) = assignment else {
            continue;
        };
        if declared.contains(&name)
            || context.config_bool("IgnoreModules", false)
                && constant_class_constructor(&value, context.source_file())
        {
            continue;
        }
        context.report(
            format!("Explicitly make `{name}` public or private using either `#public_constant` or `#private_constant`."),
            &statement.location(),
        );
    }
}

fn collect_constant_visibility_names(node: &Node<'_>, names: &mut HashSet<String>) {
    if let Some(symbol) = node.as_symbol_node() {
        names.insert(String::from_utf8_lossy(symbol.unescaped()).into_owned());
    } else if let Some(string) = node.as_string_node() {
        names.insert(String::from_utf8_lossy(string.unescaped()).into_owned());
    } else if let Some(splat) = node.as_splat_node() {
        if let Some(expression) = splat.expression() {
            if let Some(array) = expression.as_array_node() {
                for element in array.elements().iter() {
                    collect_constant_visibility_names(&element, names);
                }
            }
        }
    }
}

fn constant_class_constructor(node: &Node<'_>, source_file: SourceFile<'_>) -> bool {
    let Some(call) = node.as_call_node() else {
        return false;
    };
    if call_name(&call) != b"new" {
        return false;
    }
    call.receiver().is_some_and(|receiver| {
        matches!(
            source_file.node(&receiver),
            "Class" | "Module" | "Struct" | "Data"
        )
    })
}
