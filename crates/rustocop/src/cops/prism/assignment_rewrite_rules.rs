use ruby_prism::{Location, MultiWriteNode, Node};

use super::*;

define_rule!(OrAssignmentRule);
define_rule!(ParallelAssignmentRule);

define_cops! {
    OrAssignment => "Style/OrAssignment" => node_rule_aliases(
        OrAssignmentRule,
        on_assignment_or_if => [
            as_local_variable_write_node,
            as_instance_variable_write_node,
            as_class_variable_write_node,
            as_global_variable_write_node,
            as_if_node,
            as_unless_node
        ]
    ),
    ParallelAssignment => "Style/ParallelAssignment" => node_rule(
        as_multi_write_node,
        ParallelAssignmentRule,
        on_masgn
    ),
}

impl OrAssignmentRule<'_, '_, '_> {
    fn on_assignment_or_if(&mut self, node: &Node<'_>) {
        if let Some((variable, default)) = self.ternary_assignment(node) {
            self.register_offense(node.location(), variable, default);
        } else if let Some((variable, default)) = self.unless_assignment(node) {
            self.register_offense(node.location(), variable, default);
        }
    }

    fn ternary_assignment<'pr>(&self, node: &Node<'pr>) -> Option<(String, Node<'pr>)> {
        let (variable, expression, _) = variable_assignment(node)?;
        let conditional = expression.as_if_node()?;
        let if_branch = only_statement(conditional.statements())?;
        let else_branch = conditional.subsequent()?.as_else_node()
            .and_then(|branch| only_statement(branch.statements()))?;
        if else_branch.as_if_node().is_some()
            || self.source_file().node(&conditional.predicate()) != variable
            || self.source_file().node(&if_branch) != variable
        {
            return None;
        }
        Some((variable, else_branch))
    }

    fn unless_assignment<'pr>(&self, node: &Node<'pr>) -> Option<(String, Node<'pr>)> {
        let (predicate, assignment) = if let Some(condition) = node.as_unless_node() {
            (condition.predicate(), only_statement(condition.statements())?)
        } else {
            let condition = node.as_if_node()?;
            if condition
                .statements()
                .is_some_and(|statements| statements.body().iter().next().is_some())
            {
                return None;
            }
            let assignment = condition.subsequent()?.as_else_node()
                .and_then(|branch| only_statement(branch.statements()))?;
            (condition.predicate(), assignment)
        };
        let (variable, default, _) = variable_assignment(&assignment)?;
        (self.source_file().node(&predicate) == variable).then_some((variable, default))
    }

    fn register_offense(&mut self, location: Location<'_>, variable: String, default: Node<'_>) {
        let replacement = format!("{variable} ||= {}", self.source_file().node(&default));
        let range = location.start_offset()..location.end_offset();
        add_offense!(self, range.clone(), message: "Use the double pipe equals operator `||=` instead.", |corrector| {
            corrector.replace(range, replacement);
        });
    }
}

fn variable_assignment<'pr>(node: &Node<'pr>) -> Option<(String, Node<'pr>, Location<'pr>)> {
    let (name, value, location) = if let Some(write) = node.as_local_variable_write_node() {
        (write.name_loc(), write.value(), write.location())
    } else if let Some(write) = node.as_instance_variable_write_node() {
        (write.name_loc(), write.value(), write.location())
    } else if let Some(write) = node.as_class_variable_write_node() {
        (write.name_loc(), write.value(), write.location())
    } else if let Some(write) = node.as_global_variable_write_node() {
        (write.name_loc(), write.value(), write.location())
    } else {
        return None;
    };
    Some((String::from_utf8_lossy(name.as_slice()).into_owned(), value, location))
}

impl ParallelAssignmentRule<'_, '_, '_> {
    fn on_masgn(&mut self, node: &MultiWriteNode<'_>) {
        return_if!(self.ancestors().iter().any(|ancestor| ancestor.as_multi_write_node().is_some()));
        let mut left = node.lefts().iter().collect::<Vec<_>>();
        return_if!(left.iter().any(|target| target.as_multi_target_node().is_some()));
        if node.rest().is_some_and(|rest| rest.as_implicit_rest_node().is_none()) { return; }
        left.extend(node.rights().iter());
        return_if!(left.len() <= 1);
        let value = node.value();
        let rescue = value.as_rescue_modifier_node();
        let rhs = rescue.as_ref().map_or(value, |rescue| rescue.expression());
        let Some(array) = rhs.as_array_node() else { return };
        let right = array.elements().iter().collect::<Vec<_>>();
        return_if!(left.len() != right.len() || right.iter().any(|node| node.as_splat_node().is_some()));
        let left_source = left.iter().map(|node| self.source_file().node(node).to_string()).collect::<Vec<_>>();
        let right_source = right.iter().map(|node| render_parallel_value(node, self.source_file())).collect::<Vec<_>>();
        let Some(order) = parallel_assignment_order(&left_source, &right_source) else { return };
        let node_start = node.location().start_offset();
        let line_start = self.source_file().line_start(node_start);
        let prefix = &self.source()[line_start..node_start];
        let indent = if prefix.trim().is_empty() {
            prefix.to_string()
        } else {
            " ".repeat(self.source_file().column(node_start))
        };
        let assignments = order.into_iter().map(|index| format!("{} = {}", left_source[index], right_source[index])).collect::<Vec<_>>();
        let replacement = assignments.join(&format!("\n{indent}"));
        let offense = node.location().start_offset()..array.location().end_offset();
        if let Some(modifier) = self.modifier_parent(node.location().start_offset()) {
            self.correct_modifier(offense, replacement, modifier);
        } else if let Some(rescue) = rescue.or_else(|| self.ancestors().iter().rev().find_map(Node::as_rescue_modifier_node)) {
            self.correct_rescue(offense, replacement, rescue, node.location().start_offset()..node.location().end_offset());
        } else {
            add_offense!(self, offense.clone(), message: "Do not use parallel assignment.", |corrector| {
                corrector.replace(offense, replacement);
            });
        }
    }

    fn modifier_parent(&self, assignment_start: usize) -> Option<(std::ops::Range<usize>, String, String)> {
        for ancestor in self.ancestors().iter().rev() {
            let (location, keyword, predicate) = if let Some(node) = ancestor.as_if_node() {
                (node.location(), node.if_keyword_loc()?, node.predicate())
            } else if let Some(node) = ancestor.as_unless_node() {
                (node.location(), node.keyword_loc(), node.predicate())
            } else if let Some(node) = ancestor.as_while_node() {
                (node.location(), node.keyword_loc(), node.predicate())
            } else if let Some(node) = ancestor.as_until_node() {
                (node.location(), node.keyword_loc(), node.predicate())
            } else { continue };
            if keyword.start_offset() > assignment_start {
                return Some((location.start_offset()..location.end_offset(), String::from_utf8_lossy(keyword.as_slice()).into_owned(), self.source_file().node(&predicate).to_string()));
            }
        }
        None
    }

    fn correct_modifier(&mut self, offense: std::ops::Range<usize>, assignments: String, modifier: (std::ops::Range<usize>, String, String)) {
        let width = self.related_config_value("Layout/IndentationWidth", "Width").and_then(|value| value.parse().ok()).unwrap_or(2);
        let base = self.source_file().indentation_text(modifier.0.start);
        let body_indent = format!("{base}{}", " ".repeat(width));
        let body = assignments.lines().map(|line| format!("{body_indent}{}", line.trim_start())).collect::<Vec<_>>().join("\n");
        let replacement = format!("{} {}\n{body}\n{base}end", modifier.1, modifier.2);
        add_offense!(self, offense, message: "Do not use parallel assignment.", |corrector| {
            corrector.replace(modifier.0, replacement);
        });
    }

    fn correct_rescue(&mut self, offense: std::ops::Range<usize>, assignments: String, rescue: ruby_prism::RescueModifierNode<'_>, edit: std::ops::Range<usize>) {
        let width = self.related_config_value("Layout/IndentationWidth", "Width").and_then(|value| value.parse().ok()).unwrap_or(2);
        let base = self.source_file().indentation_text(edit.start);
        let body_indent = format!("{base}{}", " ".repeat(width));
        let fallback = self.source_file().node(&rescue.rescue_expression());
        let implicit_method_rescue = self.ancestors().iter().any(|ancestor| ancestor.as_def_node().is_some())
            && self.source().lines().filter(|line| !line.trim().is_empty()).count() == 3;
        let replacement = if implicit_method_rescue {
            let definition_indent = &base[..base.len().saturating_sub(width)];
            format!("{assignments}\n{definition_indent}rescue\n{base}{fallback}")
        } else {
            let body = assignments.lines().map(|line| format!("{body_indent}{}", line.trim_start())).collect::<Vec<_>>().join("\n");
            format!("begin\n{body}\n{base}rescue\n{body_indent}{fallback}\n{base}end")
        };
        add_offense!(self, offense, message: "Do not use parallel assignment.", |corrector| {
            corrector.replace(edit, replacement);
        });
    }
}

fn render_parallel_value(node: &Node<'_>, file: SourceFile<'_>) -> String {
    if let Some(string) = node.as_string_node() {
        if string.opening_loc().is_none() {
            return format!("'{}'", String::from_utf8_lossy(string.unescaped()).replace('\\', "\\\\").replace('\'', "\\'"));
        }
    }
    if let Some(symbol) = node.as_symbol_node() {
        if symbol.opening_loc().is_none() {
            return format!(":{}", String::from_utf8_lossy(symbol.unescaped()));
        }
    }
    file.node(node).to_string()
}

fn parallel_assignment_order(left: &[String], right: &[String]) -> Option<Vec<usize>> {
    let mut remaining = (0..left.len()).collect::<Vec<_>>();
    let mut result = Vec::new();
    while !remaining.is_empty() {
        let position = remaining.iter().position(|index| {
            !remaining.iter().any(|other| other != index && lhs_accessed_by(&left[*index], &right[*other]))
        })?;
        result.push(remaining.remove(position));
    }
    Some(result)
}

fn lhs_accessed_by(lhs: &str, rhs: &str) -> bool {
    if rhs.trim_start().starts_with("->") { return false; }
    if let Some(method) = lhs.strip_prefix("self.") {
        return rhs.split(|character: char| !character.is_ascii_alphanumeric() && character != '_').any(|part| part == method);
    }
    if lhs.contains('.') || lhs.contains('[') {
        return rhs.contains(lhs);
    }
    let name = lhs.trim_start_matches(['@', '$']);
    rhs.match_indices(name).any(|(start, _)| {
        let before = rhs[..start].chars().next_back();
        let after = rhs[start + name.len()..].chars().next();
        before.is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_')
            && after.is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_')
            && after != Some(':')
    })
}
