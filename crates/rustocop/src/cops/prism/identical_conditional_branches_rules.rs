use ruby_prism::{CaseMatchNode, CaseNode, IfNode, Node, StatementsNode};

use super::*;

define_cops! {
    IdenticalConditionalBranches => "Style/IdenticalConditionalBranches" => compatibility_prism_callbacks(
        IdenticalConditionalBranchesRule,
        [on_if, on_case, on_case_match]
    ),
}

impl IdenticalConditionalBranchesRule<'_, '_, '_> {
    fn on_if(&mut self, node: &IfNode<'_>) {
        return_if!(node.if_keyword_loc().is_some_and(|keyword| keyword.as_slice() == b"elsif"));
        let Some(subsequent) = node.subsequent() else { return };
        let mut branches = Vec::new();
        branches.push(statement_nodes(node.statements()));
        expand_subsequent(Some(subsequent), &mut branches);
        let noncorrectable_if_form = node.if_keyword_loc().is_none()
            || node
                .then_keyword_loc()
                .is_some_and(|keyword| keyword.as_slice() == b"then");
        self.check(
            node.location(),
            branches,
            noncorrectable_if_form,
            Some(self.source_file().node(&node.predicate()).to_string()),
        );
    }

    fn on_case(&mut self, node: &CaseNode<'_>) {
        let Some(else_clause) = node.else_clause() else { return };
        let mut branches = node.conditions().iter().filter_map(|condition| condition.as_when_node()).map(|branch| statement_nodes(branch.statements())).collect::<Vec<_>>();
        branches.push(statement_nodes(else_clause.statements()));
        self.check(node.location(), branches, false, None);
    }

    fn on_case_match(&mut self, node: &CaseMatchNode<'_>) {
        let Some(else_clause) = node.else_clause() else { return };
        let mut branches = node.conditions().iter().filter_map(|condition| condition.as_in_node()).map(|branch| statement_nodes(branch.statements())).collect::<Vec<_>>();
        branches.push(statement_nodes(else_clause.statements()));
        self.check(node.location(), branches, false, None);
    }

    fn check(&mut self, conditional: ruby_prism::Location<'_>, branches: Vec<Vec<Node<'_>>>, then_form: bool, condition: Option<String>) {
        let conditional = (conditional.start_offset(), conditional.end_offset());
        return_if!(branches.len() < 2 || branches.iter().any(Vec::is_empty));
        let tails = branches.iter().map(|branch| branch.last()).collect::<Option<Vec<_>>>();
        if let Some(expressions) = tails.filter(|expressions| {
            same_sources(expressions, self.source_file())
                && !assignment_conflict(
                    expressions[0],
                    condition.as_deref(),
                    self.source_file(),
                    false,
                )
                && !defined_assignment_conflict(
                    &branches,
                    expressions[0],
                    condition.as_deref(),
                    self.source_file(),
                )
                && !uninitialized_local_conflict(
                    &branches,
                    expressions[0],
                    condition.as_deref(),
                    conditional.0,
                    self.source_file(),
                )
        }) {
            self.register(expressions, conditional, false, then_form);
        }
        return_if!(branches.iter().any(|branch| branch.len() == 1) && self.last_child(conditional));
        let heads = branches.iter().map(|branch| branch.first()).collect::<Option<Vec<_>>>();
        if let Some(expressions) = heads.filter(|expressions| same_sources(expressions, self.source_file()) && !assignment_conflict(expressions[0], condition.as_deref(), self.source_file(), true)) {
            self.register(expressions, conditional, true, then_form);
        }
    }

    fn register(&mut self, expressions: Vec<&Node<'_>>, conditional: (usize, usize), before: bool, then_form: bool) {
        let expression_source = self.source_file().node(expressions[0]).to_string();
        let correctable = !then_form;
        let parent_prefix = self.parent().and_then(|parent| {
            (parent.location().start_offset() < conditional.0 && parent.location().end_offset() == conditional.1)
                .then(|| (parent.location().start_offset()..conditional.0, self.source_file().slice(parent.location().start_offset()..conditional.0).unwrap_or_default().to_string()))
        });
        for (index, expression) in expressions.into_iter().enumerate() {
            let message = format!(
                "Move `{}` out of the conditional.",
                self.source_file().node(expression)
            );
            if !correctable {
                self.report(message, expression.location());
                continue;
            }
            let remove = whole_line(expression.location(), self.source_file());
            let insert = if before { parent_prefix.as_ref().map_or(conditional.0, |(range, _)| range.start) } else { conditional.1 };
            let addition = if before {
                format!("{expression_source}\n")
            } else {
                format!("\n{}{expression_source}", parent_prefix.as_ref().map_or("", |(_, prefix)| prefix.as_str()))
            };
            let remove_prefix = (!before && index == 0).then(|| parent_prefix.as_ref().map(|(range, _)| range.clone())).flatten();
            add_offense!(self, expression.location(), message: message, |corrector| {
                corrector.replace(remove, "");
                if index == 0 {
                    if let Some(prefix) = remove_prefix { corrector.replace(prefix, ""); }
                    corrector.replace(insert..insert, addition);
                }
            });
        }
    }

    fn last_child(&self, conditional: (usize, usize)) -> bool {
        self.parent().is_none_or(|parent| parent.location().end_offset() == conditional.1)
    }

}

fn assignment_conflict(node: &Node<'_>, condition: Option<&str>, file: SourceFile<'_>, head: bool) -> bool {
    let Some(condition) = condition else { return false };
    let source = file.node(node);
    let Some((operator, left, right)) = [" ||= ", " &&= ", " += ", " -= ", " = "].iter().find_map(|operator| source.split_once(operator).map(|(left, right)| (*operator, left, right))) else { return false };
    let condition_mentions = |name: &str| {
        condition.split(|character: char| !character.is_ascii_alphanumeric() && character != '_' && character != '@').any(|word| word == name)
    };
    if operator == " = " {
        let first_child = right.trim();
        let variable_write = node.as_local_variable_write_node().is_some()
            || node.as_instance_variable_write_node().is_some()
            || node.as_class_variable_write_node().is_some()
            || node.as_global_variable_write_node().is_some();
        if variable_write
            && first_child
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'@'))
            && condition_mentions(first_child)
        {
            return true;
        }
        if !head { return false; }
    }
    let name = left.trim().split(['[', '.']).next().unwrap_or(left.trim());
    condition_mentions(name)
}

fn statement_nodes(statements: Option<StatementsNode<'_>>) -> Vec<Node<'_>> {
    statements.map(|statements| statements.body().iter().collect()).unwrap_or_default()
}

fn expand_subsequent<'pr>(subsequent: Option<Node<'pr>>, branches: &mut Vec<Vec<Node<'pr>>>) {
    let Some(subsequent) = subsequent else {
        branches.push(Vec::new());
        return;
    };
    if let Some(elsif) = subsequent.as_if_node() {
        branches.push(statement_nodes(elsif.statements()));
        expand_subsequent(elsif.subsequent(), branches);
        return;
    }
    let Some(else_node) = subsequent.as_else_node() else {
        branches.push(Vec::new());
        return;
    };
    let nodes = statement_nodes(else_node.statements());
    if nodes.len() != 1 {
        branches.push(nodes);
        return;
    }
    if let Some(nested) = nodes[0].as_if_node() {
        branches.push(statement_nodes(nested.statements()));
        expand_subsequent(nested.subsequent(), branches);
    } else if let Some(nested) = nodes[0].as_unless_node() {
        branches.push(
            nested
                .else_clause()
                .map_or_else(Vec::new, |clause| statement_nodes(clause.statements())),
        );
        branches.push(statement_nodes(nested.statements()));
    } else {
        branches.push(nodes);
    }
}

fn same_sources(nodes: &[&Node<'_>], file: SourceFile<'_>) -> bool {
    nodes.first().is_some_and(|first| {
        let semantic_source = |node: &Node<'_>| {
            let source = file.node(node).to_string();
            let mut heredoc_content = None;
            if source.contains("<<") {
                if let Some(range) = file
                    .heredoc_ranges()
                    .into_iter()
                    .find(|range| range.start >= node.location().start_offset())
                {
                    heredoc_content = file.slice(range);
                }
            }
            let mut normalized = String::with_capacity(source.len());
            let mut quote = None;
            let mut escaped = false;
            for character in source.chars() {
                if let Some(delimiter) = quote {
                    normalized.push(character);
                    if escaped {
                        escaped = false;
                    } else if character == '\\' {
                        escaped = true;
                    } else if character == delimiter {
                        quote = None;
                    }
                } else if matches!(character, '\'' | '"') {
                    quote = Some(character);
                    normalized.push(character);
                } else if !character.is_whitespace() {
                    normalized.push(character);
                }
            }
            if let Some(open) = normalized.find('(') {
                let callable = &normalized[..open];
                if normalized.ends_with(')')
                    && !callable.is_empty()
                    && callable.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric()
                            || matches!(byte, b'_' | b':' | b'.' | b'?' | b'!')
                    })
                {
                    normalized.remove(open);
                    normalized.pop();
                }
            }
            if normalized.contains("__LINE__") {
                normalized.push('@');
                normalized.push_str(
                    &(file.as_str()[..node.location().start_offset()]
                        .bytes()
                        .filter(|byte| *byte == b'\n')
                        .count()
                        + 1)
                        .to_string(),
                );
            }
            if let Some(content) = heredoc_content {
                normalized.push('\0');
                normalized.push_str(content);
            }
            normalized
        };
        let source = semantic_source(first);
        source != "()"
            && nodes.iter().skip(1).all(|node| {
                semantic_source(node) == source
            })
    })
}

fn defined_assignment_conflict(
    branches: &[Vec<Node<'_>>],
    expression: &Node<'_>,
    condition: Option<&str>,
    file: SourceFile<'_>,
) -> bool {
    let Some(condition) = condition else { return false };
    let Some(start) = condition.find("defined?(") else { return false };
    let remainder = &condition[start + "defined?(".len()..];
    let Some(end) = remainder.find(')') else { return false };
    let name = remainder[..end].trim();
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return false;
    }
    let expression_source = file.node(expression);
    expression_source
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .any(|word| word == name)
        && branches
        .iter()
        .flat_map(|branch| branch.iter().take(branch.len().saturating_sub(1)))
        .flat_map(|node| file.node(node).lines())
        .any(|line| {
            line.trim_start()
                .starts_with(&format!("{name} = "))
        })
}

fn uninitialized_local_conflict(
    branches: &[Vec<Node<'_>>],
    expression: &Node<'_>,
    condition: Option<&str>,
    conditional_start: usize,
    file: SourceFile<'_>,
) -> bool {
    let name = file.node(expression).trim();
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return false;
    }
    let assignment = format!("{name} = ");
    let assigned_branches = branches
        .iter()
        .filter(|branch| {
            branch
                .iter()
                .take(branch.len().saturating_sub(1))
                .flat_map(|node| file.node(node).lines())
                .any(|line| line.trim_start().starts_with(&assignment))
        })
        .count();
    let mentioned_before = file.as_str()[..conditional_start]
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .any(|word| word == name);
    assigned_branches > 0
        && assigned_branches < branches.len()
        && !condition.is_some_and(|condition| condition.contains(&assignment))
        && !mentioned_before
}

fn whole_line(location: ruby_prism::Location<'_>, file: SourceFile<'_>) -> std::ops::Range<usize> {
    let start = file.line_start(location.start_offset());
    let end = file.line_end(location.end_offset());
    start..if file.slice(end..end + 1) == Some("\n") { end + 1 } else { end }
}
