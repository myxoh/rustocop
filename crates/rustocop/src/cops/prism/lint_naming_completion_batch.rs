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
    if node.as_program_node().is_none() {
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
        let Some(at) = line.find("<<") else { continue };
        let delimiter = line[at + 2..]
            .trim_start_matches(['-', '~', '\'', '"', '`'])
            .trim_end_matches(['\'', '"', '`'])
            .trim();
        if !matches!(delimiter, "END" | "EOH" | "EOS" | "EOL") {
            continue;
        }
        if let Some(start) = source[line_offset..].rfind(&format!("\n{delimiter}")) {
            let absolute = line_offset + start + 1;
            context.report(
                "Use meaningful heredoc delimiters.",
                absolute..absolute + delimiter.len(),
            );
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
        let Some(pattern) = line.trim_start().strip_prefix("in ") else {
            continue;
        };
        if catch_all {
            let end = lines[index + 1..]
                .iter()
                .find(|(_, next)| next.trim_start().starts_with("in ") || next.trim() == "end")
                .map_or(offset + line.len(), |(at, _)| *at);
            context.report(
                "Unreachable `in` pattern branch detected.",
                offset..end.saturating_sub(1),
            );
        }
        catch_all = pattern.trim() == "_"
            || pattern
                .trim()
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b == b'_');
    }
}

fn method_parameter_name(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("def ") {
            continue;
        }
        let Some(open) = line.find('(') else { continue };
        let Some(close) = line[open..].find(')').map(|at| open + at) else {
            continue;
        };
        for parameter in line[open + 1..close]
            .split(',')
            .map(|p| p.trim().trim_start_matches(['*', '&']))
        {
            if parameter.chars().last().is_some_and(|c| c.is_ascii_digit()) {
                let start = offset + line.find(parameter).unwrap_or(0);
                context.report(
                    "Do not end method parameter with a number.",
                    start..start + parameter.len(),
                );
            }
        }
    }
}
