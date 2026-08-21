use super::*;
use std::collections::{HashMap, HashSet};

define_cops! {
    UnderscorePrefixedVariableName => "Lint/UnderscorePrefixedVariableName" => any_node(underscore_variable),
    HeredocDelimiterNaming => "Naming/HeredocDelimiterNaming" => source(heredoc_naming),
    DeprecatedConstants => "Lint/DeprecatedConstants" => source(deprecated_constants),
    RedundantCopEnableDirective => "Lint/RedundantCopEnableDirective" => source(redundant_enable),
    UnreachablePatternBranch => "Lint/UnreachablePatternBranch" => source(unreachable_pattern),
    MethodParameterName => "Naming/MethodParameterName" => source(method_parameter_name),
}

fn underscore_variable(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if node.as_program_node().is_none()
        || context.config_bool("AllowKeywordBlockArguments", false)
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
        context.report(
            "Do not use prefix `_` for a variable that is used.",
            range,
        );
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

    fn declare(
        &mut self,
        name: &[u8],
        depth: u32,
        location: ruby_prism::Location<'_>,
    ) {
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
        self.variables.entry((scope, name.to_vec())).or_default().used = true;
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
            let modifier = usize::from(matches!(
                line.as_bytes().get(at + 2),
                Some(b'-' | b'~')
            ));
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
            } else if let Some(start) = source[line_offset..]
                .find(&format!("\n{delimiter}\n"))
            {
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
    for (old, new) in [("NIL", "nil"), ("TRUE", "true"), ("FALSE", "false")] {
        for start in context.source_file().code_offsets(old) {
            context.replace(
                format!("Use `{new}` instead of `{old}`, deprecated since Ruby 2.4."),
                start..start + old.len(),
                start..start + old.len(),
                new,
            );
        }
    }
}

fn redundant_enable(context: &mut CopContext<'_, '_>) {
    let mut disabled = HashSet::new();
    for (offset, line) in context.source_file().lines() {
        if let Some(list) = line.split("rubocop:disable ").nth(1) {
            disabled.extend(list.split(',').map(|cop| cop.trim().to_string()));
        }
        let Some(list) = line.split("rubocop:enable ").nth(1) else {
            continue;
        };
        for cop in list.split(',').map(str::trim) {
            if disabled.remove(cop) {
                continue;
            }
            let start = offset + line.find(cop).unwrap_or(0);
            context.remove(
                format!("Unnecessary enabling of {cop}."),
                start..start + cop.len(),
                start..start + cop.len(),
            );
        }
    }
}

fn unreachable_pattern(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut catch_all = false;
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        let trimmed = line.trim_start();
        if catch_all && trimmed == "else" {
            context.report(
                "Unreachable `else` branch detected.",
                offset..offset + line.len(),
            );
            continue;
        }
        let Some(pattern) = trimmed.strip_prefix("in ") else {
            continue;
        };
        if catch_all {
            let end = lines[index + 1..]
                .iter()
                .find(|(_, next)| {
                    next.trim_start().starts_with("in ")
                        || matches!(next.trim(), "else" | "end")
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
            .split(|character: char| character.is_ascii_whitespace() || "()|=>,".contains(character))
            .any(|part| part == "_");
        catch_all = !guarded
            && (has_wildcard || first.is_some_and(|byte| byte.is_ascii_lowercase()));
    }
}

fn method_parameter_name(context: &mut CopContext<'_, '_>) {
    let minimum = context.config_usize("MinNameLength", 3);
    let allow_numbers = context.config_bool("AllowNamesEndingInNumbers", false);
    let allowed = context.config_values("AllowedNames").to_vec();
    let forbidden = context.config_values("ForbiddenNames").to_vec();
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("def ") {
            continue;
        }
        let Some(open) = line.find('(') else { continue };
        let Some(close) = line[open..].find(')').map(|at| open + at) else {
            continue;
        };
        let mut search_from = open + 1;
        for raw in line[open + 1..close].split(',').map(str::trim) {
            let token = raw
                .split(['=', ':'])
                .next()
                .unwrap_or("")
                .trim();
            let name = token.trim_start_matches(['*', '&']);
            let normalized = name.trim_start_matches('_');
            if normalized.is_empty() || normalized == "..." {
                continue;
            }
            let relative = line[search_from..].find(token).unwrap_or(0) + search_from;
            search_from = relative + token.len();
            let range = offset + relative..offset + relative + token.len();
            if allowed.iter().any(|allowed| allowed == normalized) {
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
}
