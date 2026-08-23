use super::*;
use std::collections::{HashMap, HashSet};

define_cops! {
    UnderscorePrefixedVariableName => "Lint/UnderscorePrefixedVariableName" => any_node(underscore_variable),
    HeredocDelimiterNaming => "Naming/HeredocDelimiterNaming" => source(heredoc_naming),
    DeprecatedConstants => "Lint/DeprecatedConstants" => source(deprecated_constants),
    RedundantCopEnableDirective => "Lint/RedundantCopEnableDirective" => source(redundant_enable),
    UnreachablePatternBranch => "Lint/UnreachablePatternBranch" => source(unreachable_pattern),
    MethodParameterName => "Naming/MethodParameterName" => node(as_def_node, method_parameter_name),
    AccessorMethodName => "Naming/AccessorMethodName" => node(as_def_node, accessor_method_name),
}

fn underscore_variable(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if node.as_program_node().is_none() || context.config_bool("AllowKeywordBlockArguments", false)
    {
        return;
    }
    let mut visitor = UnderscoreVariableVisitor::default();
    ruby_prism::Visit::visit(&mut visitor, node);
    let mut offenses = visitor
        .variables
        .into_values()
        .filter_map(|variable| variable.used.then_some(variable.declaration).flatten())
        .collect::<Vec<_>>();
    offenses.sort_by_key(|range| range.start);
    for range in offenses {
        let range = if context.source().starts_with('/') && context.source().contains("(?<_") {
            context.source()[1..]
                .find('/')
                .map_or(range.clone(), |end| 0..end + 2)
        } else {
            context.source()[..range.start]
                .rfind("/(?<")
                .and_then(|start| {
                    context.source()[range.end..]
                        .find('/')
                        .map(|end| start..range.end + end + 1)
                })
                .unwrap_or(range)
        };
        context.report("Do not use prefix `_` for a variable that is used.", range);
    }
}

#[derive(Default)]
struct UnderscoreVariableVisitor {
    variables: HashMap<(usize, Vec<u8>), UnderscoreVariable>,
    scopes: Vec<usize>,
    branch_scopes: Vec<bool>,
    next_scope: usize,
}

#[derive(Default)]
struct UnderscoreVariable {
    declaration: Option<std::ops::Range<usize>>,
    used: bool,
}

impl UnderscoreVariableVisitor {
    fn scope_for_depth(&self, depth: u32) -> Option<usize> {
        self.scopes
            .len()
            .checked_sub(depth as usize + 1)
            .and_then(|index| self.scopes.get(index))
            .copied()
    }

    fn declare(&mut self, name: &[u8], depth: u32, location: ruby_prism::Location<'_>) {
        if !underscore_prefixed_name(name) {
            return;
        }
        let Some(scope) = self.scope_for_depth(depth) else {
            return;
        };
        self.variables
            .entry((scope, name.to_vec()))
            .or_default()
            .declaration
            .get_or_insert(location.start_offset()..location.start_offset() + name.len());
    }

    fn use_variable(&mut self, name: &[u8], depth: u32) {
        if !underscore_prefixed_name(name) {
            return;
        }
        let Some(scope) = self.scope_for_depth(depth) else {
            return;
        };
        self.variables
            .entry((scope, name.to_vec()))
            .or_default()
            .used = true;
    }

    fn observe(&mut self, node: &Node<'_>) {
        if let Some(read) = node.as_local_variable_read_node() {
            self.use_variable(read.name().as_slice(), read.depth());
        } else if let Some(write) = node.as_local_variable_write_node() {
            self.declare(write.name().as_slice(), write.depth(), write.name_loc());
        } else if let Some(target) = node.as_local_variable_target_node() {
            self.declare(target.name().as_slice(), target.depth(), target.location());
        } else if let Some(write) = node.as_local_variable_and_write_node() {
            self.declare(write.name().as_slice(), write.depth(), write.name_loc());
            self.use_variable(write.name().as_slice(), write.depth());
        } else if let Some(write) = node.as_local_variable_or_write_node() {
            self.declare(write.name().as_slice(), write.depth(), write.name_loc());
            self.use_variable(write.name().as_slice(), write.depth());
        } else if let Some(write) = node.as_local_variable_operator_write_node() {
            self.declare(write.name().as_slice(), write.depth(), write.name_loc());
            self.use_variable(write.name().as_slice(), write.depth());
        } else if let Some(parameter) = node.as_required_parameter_node() {
            self.declare(parameter.name().as_slice(), 0, parameter.location());
        } else if let Some(parameter) = node.as_optional_parameter_node() {
            self.declare(parameter.name().as_slice(), 0, parameter.name_loc());
        } else if let Some(parameter) = node.as_rest_parameter_node() {
            if let (Some(name), Some(location)) = (parameter.name(), parameter.name_loc()) {
                self.declare(name.as_slice(), 0, location);
            }
        } else if let Some(parameter) = node.as_required_keyword_parameter_node() {
            self.declare(parameter.name().as_slice(), 0, parameter.name_loc());
        } else if let Some(parameter) = node.as_optional_keyword_parameter_node() {
            self.declare(parameter.name().as_slice(), 0, parameter.name_loc());
        } else if let Some(parameter) = node.as_keyword_rest_parameter_node() {
            if let (Some(name), Some(location)) = (parameter.name(), parameter.name_loc()) {
                self.declare(name.as_slice(), 0, location);
            }
        } else if let Some(parameter) = node.as_block_parameter_node() {
            if let (Some(name), Some(location)) = (parameter.name(), parameter.name_loc()) {
                self.declare(name.as_slice(), 0, location);
            }
        } else if let Some(parameter) = node.as_block_local_variable_node() {
            self.declare(parameter.name().as_slice(), 0, parameter.location());
        }
    }
}

impl<'pr> ruby_prism::Visit<'pr> for UnderscoreVariableVisitor {
    fn visit_branch_node_enter(&mut self, node: Node<'pr>) {
        let opens_scope = prism_local_scope(&node);
        self.branch_scopes.push(opens_scope);
        if opens_scope {
            let scope = self.next_scope;
            self.next_scope += 1;
            self.scopes.push(scope);
        }
        self.observe(&node);
    }

    fn visit_branch_node_leave(&mut self) {
        if self.branch_scopes.pop() == Some(true) {
            self.scopes.pop();
        }
    }

    fn visit_leaf_node_enter(&mut self, node: Node<'pr>) {
        self.observe(&node);
    }
}

fn prism_local_scope(node: &Node<'_>) -> bool {
    node.as_program_node().is_some()
        || node.as_def_node().is_some()
        || node.as_block_node().is_some()
        || node.as_lambda_node().is_some()
        || node.as_class_node().is_some()
        || node.as_module_node().is_some()
        || node.as_singleton_class_node().is_some()
}

fn underscore_prefixed_name(name: &[u8]) -> bool {
    name.starts_with(b"_")
}

fn heredoc_naming(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for (line_offset, line) in context.source_file().lines() {
        for (at, _) in line.match_indices("<<") {
            let modifier = usize::from(matches!(line.as_bytes().get(at + 2), Some(b'-' | b'~')));
            let tail = &line[at + 2 + modifier..];
            let (delimiter, token_length) = match tail.as_bytes().first().copied() {
                Some(quote @ (b'\'' | b'"' | b'`')) => {
                    let value = &tail[1..];
                    let Some(end) = value.bytes().position(|byte| byte == quote) else {
                        continue;
                    };
                    (&value[..end], 2 + modifier + end + 2)
                }
                _ => {
                    let end = tail
                        .find(|character: char| {
                            !(character.is_ascii_alphanumeric() || character == '_')
                        })
                        .unwrap_or(tail.len());
                    (&tail[..end], 2 + modifier + end)
                }
            };
            let meaningful = !delimiter.is_empty()
                && delimiter
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                && delimiter
                    .bytes()
                    .any(|byte| !matches!(byte, b'E' | b'N' | b'D' | b'O' | b'H' | b'S' | b'L'));
            if meaningful {
                continue;
            }
            if delimiter.is_empty() {
                context.report(
                    "Use meaningful heredoc delimiters.",
                    line_offset + at..line_offset + at + token_length,
                );
            } else if let Some(start) = source[line_offset..].find(&format!("\n{delimiter}\n")) {
                let absolute = line_offset + start + 1;
                context.report(
                    "Use meaningful heredoc delimiters.",
                    absolute..absolute + delimiter.len(),
                );
            }
        }
    }
}

fn deprecated_constants(context: &mut CopContext<'_, '_>) {
    let configured = context
        .config_map("DeprecatedConstants")
        .cloned()
        .unwrap_or_default();
    for (old, details) in configured {
        let mut alternative = None;
        let mut deprecated_version = None;
        for field in details.lines() {
            let Some((key, value)) = field.split_once('=') else {
                continue;
            };
            match key {
                "Alternative" => alternative = Some(value),
                "DeprecatedVersion" => deprecated_version = Some(value),
                _ => {}
            }
        }
        if deprecated_version.is_some_and(|version| {
            let mut parts = version
                .split('.')
                .filter_map(|part| part.parse::<u16>().ok());
            let (Some(major), Some(minor)) = (parts.next(), parts.next()) else {
                return false;
            };
            !context.target_ruby_version().at_least(major, minor)
        }) {
            continue;
        }
        for start in context.source_file().code_offsets(&old) {
            let root_start = start.checked_sub(2).filter(|root| {
                &context.source()[*root..start] == "::"
                    && (*root == 0
                        || !context.source().as_bytes()[*root - 1].is_ascii_alphanumeric()
                            && context.source().as_bytes()[*root - 1] != b'_')
            });
            if start > 0 && root_start.is_none() {
                let previous = context.source().as_bytes()[start - 1];
                if previous.is_ascii_alphanumeric() || matches!(previous, b'_' | b':') {
                    continue;
                }
            }
            let end = start + old.len();
            if context
                .source()
                .as_bytes()
                .get(end)
                .is_some_and(|next| next.is_ascii_alphanumeric() || matches!(next, b'_' | b':'))
            {
                continue;
            }
            let offense = root_start.unwrap_or(start)..end;
            let used = &context.source()[offense.clone()];
            let suffix = deprecated_version
                .map(|version| format!(", deprecated since Ruby {version}"))
                .unwrap_or_default();
            if let Some(alternative) = alternative {
                context.replace(
                    format!("Use `{alternative}` instead of `{used}`{suffix}."),
                    offense.clone(),
                    offense,
                    alternative,
                );
            } else {
                context.report(format!("Do not use `{used}`{suffix}."), offense);
            }
        }
    }
}

fn redundant_enable(context: &mut CopContext<'_, '_>) {
    let mut disabled = HashSet::new();
    let mut configured_disable_consumed = HashSet::new();
    for (offset, line) in context.source_file().lines() {
        if let Some(list) = line.split("rubocop:disable ").nth(1) {
            disabled.extend(list.split(',').map(|cop| cop.trim().to_string()));
        }
        let Some(marker) = line.find("rubocop:enable ") else {
            continue;
        };
        let list_start = marker + "rubocop:enable ".len();
        let list = &line[list_start..];
        let mut redundant = Vec::new();
        let mut necessary = Vec::new();
        let mut preserve_department_line = false;
        for cop in list.split(',').map(str::trim) {
            if cop == "all" && !disabled.is_empty() {
                disabled.clear();
                necessary.push(cop);
                continue;
            }
            if disabled.remove(cop) {
                necessary.push(cop);
                continue;
            }
            if context.related_config_value(cop, "Enabled") == Some("false")
                && configured_disable_consumed.insert(cop.to_string())
            {
                necessary.push(cop);
                continue;
            }
            if !cop.contains('/')
                && disabled
                    .iter()
                    .any(|disabled| disabled.starts_with(&format!("{cop}/")))
            {
                preserve_department_line = true;
            }
            redundant.push(cop);
        }
        if redundant.is_empty() {
            continue;
        }
        let separator = list
            .find(',')
            .map(|comma| {
                let mut end = comma + 1;
                while list.as_bytes().get(end) == Some(&b' ') {
                    end += 1;
                }
                &list[comma..end]
            })
            .unwrap_or(", ");
        let replacement = necessary.join(separator);
        for (index, cop) in redundant.iter().enumerate() {
            let start = offset + line.find(cop).unwrap_or(list_start);
            let label = if *cop == "all" { "all cops" } else { cop };
            let message = format!("Unnecessary enabling of {label}.");
            if index == 0 {
                if necessary.is_empty() {
                    let mut edit_end = offset + line.len();
                    if context.source().as_bytes().get(edit_end) == Some(&b'\n') {
                        edit_end += 1;
                    }
                    let replacement = if preserve_department_line { "\n" } else { "" };
                    context.replace(
                        message,
                        start..start + cop.len(),
                        offset..edit_end,
                        replacement,
                    );
                } else {
                    context.replace(
                        message,
                        start..start + cop.len(),
                        offset + list_start..offset + line.len(),
                        replacement.clone(),
                    );
                }
            } else {
                context.replace(message, start..start + cop.len(), start..start, "");
            }
        }
    }
}

fn unreachable_pattern(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let literal_ranges = context.source_file().literal_ranges();
    let mut cases = Vec::<(usize, Option<usize>)>::new();
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        let trimmed = line.trim_start();
        let indentation = line.len() - trimmed.len();
        let code_start = offset + indentation;
        if literal_ranges
            .iter()
            .any(|range| range.start <= code_start && code_start < range.end)
            || trimmed.starts_with('#')
        {
            continue;
        }
        if trimmed == "end" {
            if cases.last().is_some_and(|(case_indent, _)| *case_indent == indentation) {
                cases.pop();
            }
            continue;
        }
        if trimmed == "case" || trimmed.starts_with("case ") {
            cases.push((indentation, None));
            continue;
        }
        let Some((case_indent, catch_all_indent)) = cases.last_mut() else {
            continue;
        };
        if *case_indent > indentation {
            continue;
        }
        if *catch_all_indent == Some(indentation) && trimmed == "else" {
            context.report(
                "Unreachable `else` branch detected.",
                offset..offset + line.len(),
            );
            continue;
        }
        let Some(pattern) = trimmed.strip_prefix("in ") else {
            continue;
        };
        if catch_all_indent.is_some() {
            let end = lines[index + 1..]
                .iter()
                .find(|(_, next)| {
                    next.trim_start().starts_with("in ") || matches!(next.trim(), "else" | "end")
                })
                .map_or(offset + line.len(), |(at, _)| *at);
            context.report(
                "Unreachable `in` pattern branch detected.",
                offset..end.saturating_sub(1),
            );
            continue;
        }
        let pattern = pattern.trim();
        let guarded = pattern.contains(" if ") || pattern.contains(" unless ");
        let first = pattern.trim_start_matches('(').as_bytes().first().copied();
        let has_wildcard = pattern
            .split(|character: char| {
                character.is_ascii_whitespace() || "()|=>,".contains(character)
            })
            .any(|part| part == "_");
        if !guarded && (has_wildcard || first.is_some_and(|byte| byte.is_ascii_lowercase())) {
            *catch_all_indent = Some(indentation);
        }
    }
}

fn method_parameter_name(definition: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let minimum = context.config_usize("MinNameLength", 3);
    let allow_numbers = context.config_bool("AllowNamesEndingInNumbers", false);
    let allowed = context.config_values("AllowedNames").to_vec();
    let forbidden = context.config_values("ForbiddenNames").to_vec();
    let Some(parameters) = definition.parameters() else {
        return;
    };
    for (name, range) in named_method_parameters(&parameters) {
        let normalized = name.trim_start_matches('_');
        if normalized.is_empty() || allowed.iter().any(|allowed| allowed == normalized) {
            continue;
        }
        let message = if forbidden.iter().any(|forbidden| forbidden == normalized) {
            Some(format!(
                "Do not use {normalized} as a name for a method parameter."
            ))
        } else if normalized.len() < minimum {
            Some(format!(
                "Method parameter must be at least {minimum} characters long."
            ))
        } else if normalized.bytes().any(|byte| byte.is_ascii_uppercase()) {
            Some("Only use lowercase characters for method parameter.".to_string())
        } else if !allow_numbers
            && normalized
                .bytes()
                .last()
                .is_some_and(|byte| byte.is_ascii_digit())
        {
            Some("Do not end method parameter with a number.".to_string())
        } else {
            None
        };
        if let Some(message) = message {
            context.report(message, range);
        }
    }
}

fn named_method_parameters(
    parameters: &ruby_prism::ParametersNode<'_>,
) -> Vec<(String, std::ops::Range<usize>)> {
    let mut result = Vec::new();
    for parameter in parameters
        .requireds()
        .iter()
        .chain(parameters.posts().iter())
    {
        if let Some(parameter) = parameter.as_required_parameter_node() {
            result.push((
                String::from_utf8_lossy(parameter.name().as_slice()).into_owned(),
                parameter.location().start_offset()..parameter.location().end_offset(),
            ));
        }
    }
    for parameter in parameters.optionals().iter() {
        if let Some(parameter) = parameter.as_optional_parameter_node() {
            let location = parameter.name_loc();
            result.push((
                String::from_utf8_lossy(parameter.name().as_slice()).into_owned(),
                location.start_offset()..location.end_offset(),
            ));
        }
    }
    for parameter in parameters.keywords().iter() {
        if let Some(parameter) = parameter.as_required_keyword_parameter_node() {
            let location = parameter.name_loc();
            result.push((
                String::from_utf8_lossy(parameter.name().as_slice()).into_owned(),
                location.start_offset()..location.end_offset().saturating_sub(1),
            ));
        } else if let Some(parameter) = parameter.as_optional_keyword_parameter_node() {
            let location = parameter.name_loc();
            result.push((
                String::from_utf8_lossy(parameter.name().as_slice()).into_owned(),
                location.start_offset()..location.end_offset().saturating_sub(1),
            ));
        }
    }
    if let Some(parameter) = parameters
        .rest()
        .and_then(|node| node.as_rest_parameter_node())
    {
        if let (Some(name), Some(_)) = (parameter.name(), parameter.name_loc()) {
            result.push((
                String::from_utf8_lossy(name.as_slice()).into_owned(),
                parameter.location().start_offset()..parameter.location().end_offset(),
            ));
        }
    }
    if let Some(parameter) = parameters
        .keyword_rest()
        .and_then(|node| node.as_keyword_rest_parameter_node())
    {
        if let (Some(name), Some(_)) = (parameter.name(), parameter.name_loc()) {
            result.push((
                String::from_utf8_lossy(name.as_slice()).into_owned(),
                parameter.location().start_offset()..parameter.location().end_offset(),
            ));
        }
    }
    if let Some(parameter) = parameters.block() {
        if let (Some(name), Some(_)) = (parameter.name(), parameter.name_loc()) {
            let start = parameter.location().start_offset();
            result.push((
                String::from_utf8_lossy(name.as_slice()).into_owned(),
                start..start + name.as_slice().len(),
            ));
        }
    }
    result
}

fn accessor_method_name(definition: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let name = String::from_utf8_lossy(definition.name().as_slice());
    if name.ends_with(['!', '?', '=']) {
        return;
    }
    let parameter_count = definition.parameters().map_or(0, |parameters| {
        parameters.requireds().len()
            + parameters.optionals().len()
            + usize::from(parameters.rest().is_some())
            + parameters.posts().len()
            + parameters.keywords().len()
            + usize::from(parameters.keyword_rest().is_some())
            + usize::from(parameters.block().is_some())
    });
    let single_required = definition.parameters().is_some_and(|parameters| {
        parameters.requireds().len() == 1
            && parameters
                .requireds()
                .first()
                .is_some_and(|node| node.as_required_parameter_node().is_some())
            && parameter_count == 1
    });
    let message = if name.starts_with("get_") && parameter_count == 0 {
        Some("Do not prefix reader method names with `get_`.")
    } else if name.starts_with("set_") && single_required {
        Some("Do not prefix writer method names with `set_`.")
    } else {
        None
    };
    if let Some(message) = message {
        let location = definition.name_loc();
        context.report(message, location.start_offset()..location.end_offset());
    }
}
