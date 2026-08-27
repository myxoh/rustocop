use ruby_prism::{BeginNode, Node};

use super::*;

define_rule!(RedundantBeginRule);

define_cops! {
    RedundantBegin => "Style/RedundantBegin" => compatibility_prism_node_rule(
        as_begin_node,
        RedundantBeginRule,
        on_kwbegin
    ),
}

impl RedundantBeginRule<'_, '_, '_> {
    fn on_kwbegin(&mut self, node: &BeginNode<'_>) {
        let Some(begin_keyword) = node.begin_keyword_loc() else { return };
        let Some(end_keyword) = node.end_keyword_loc() else { return };
        let statements = node.statements().map(|statements| statements.body().iter().collect::<Vec<_>>()).unwrap_or_default();
        return_if!(statements.is_empty());
        let parent = semantic_begin_parent(self.ancestors());
        let direct_body = parent.is_some_and(|parent| parent_has_only_begin(parent, node));
        let assignment = parent.filter(|parent| parent.as_def_node().is_none())
            .and_then(|parent| assignment_around_begin(parent, begin_keyword.start_offset(), self.source_file()));
        return_if!(assignment.is_some() && statements.len() != 1);
        return_if!(parent.is_some_and(|parent| {
            parent.as_call_node().is_some()
                || parent.as_and_node().is_some()
                || parent.as_or_node().is_some()
        }));
        return_if!(direct_block_is_lambda(self.ancestors()));

        let endless_definition = parent.and_then(Node::as_def_node).is_some_and(|definition| definition.equal_loc().is_some());
        let direct_definition = direct_body && parent.is_some_and(|parent| parent.as_def_node().is_some()) && !endless_definition;
        let direct_block = parent.and_then(Node::as_block_node).is_some_and(|block| {
            direct_body && block.opening_loc().as_slice() == b"do" && self.target_ruby_version().at_least(2, 5)
        });
        let direct_branch = direct_body && parent.is_some_and(branch_or_loop_parent);
        let modifier_parent = parent.and_then(modifier_parent_parts)
            .filter(|(_, keyword, _)| keyword.start_offset() > begin_keyword.start_offset());
        return_if!(modifier_parent.is_some() && statements.len() > 1);
        return_if!(parent.and_then(Node::as_while_node).is_some_and(|loop_node| loop_node.keyword_loc().start_offset() > begin_keyword.start_offset())
            || parent.and_then(Node::as_until_node).is_some_and(|loop_node| loop_node.keyword_loc().start_offset() > begin_keyword.start_offset()));
        let top_level = parent.is_some_and(|parent| parent.as_program_node().is_some());
        let contains_rescue_or_ensure = node.rescue_clause().is_some()
            || node.ensure_clause().is_some()
            || statements.first().is_some_and(|statement| statement.as_rescue_modifier_node().is_some());
        if contains_rescue_or_ensure
            && !(direct_definition || direct_block)
        {
            return;
        }
        return_if!(!(direct_definition || direct_block || direct_branch || top_level || assignment.is_some() || statements.len() == 1));

        if assignment.is_some() && statements.first().is_some_and(|statement| self.source_file().node(statement).contains("begin")) {
            return;
        }
        if let Some(outer) = enclosing_assignment_begin(self.ancestors(), self.source_file()) {
            let Some(outer_body) = outer.statements() else { return };
            let outer_statements = outer_body.body().iter().collect::<Vec<_>>();
            let Some(first) = outer_statements.first() else { return };
            let replacement = format!("{}\n{}", collapse_nested_assignment_begin(self.source_file().node(first)), self.source_file().indentation_text(first.location().start_offset()));
            let Some(outer_begin) = outer.begin_keyword_loc() else { return };
            let Some(closing) = outer.end_keyword_loc() else { return };
            let replace = outer_begin.start_offset()..first.location().end_offset();
            add_offense!(self, begin_keyword, message: "Redundant `begin` block detected.", |corrector| {
                corrector.replace(replace, replacement);
                corrector.remove(closing);
            });
        } else if let Some(assignment_range) = assignment {
            let first = &statements[0];
            let mut replacement = self.source_file().node(first).to_string();
            if (first.as_if_node().is_some() || first.as_unless_node().is_some())
                && self.source_file().same_line(first.location().start_offset(), first.location().end_offset().saturating_sub(1))
            {
                replacement = format!("({replacement})");
            }
            replacement = collapse_nested_assignment_begin(&replacement);
            let comments = self.source()[begin_keyword.end_offset()..first.location().start_offset()].to_string();
            let replace = begin_keyword.start_offset()..first.location().end_offset();
            add_offense!(self, begin_keyword, message: "Redundant `begin` block detected.", |corrector| {
                if !comments.trim().is_empty() {
                    corrector.replace(assignment_range.start..assignment_range.start, comments);
                }
                corrector.replace(replace, replacement);
                corrector.remove(end_keyword);
            });
        } else if endless_definition {
            let first = &statements[0];
            let replace = begin_keyword.start_offset()..first.location().start_offset();
            let closing = end_keyword.start_offset()..end_keyword.end_offset();
            add_offense!(self, begin_keyword, message: "Redundant `begin` block detected.", |corrector| {
                corrector.replace(replace, " ");
                corrector.remove(closing);
            });
        } else if let Some((parent_location, keyword, predicate)) = modifier_parent.filter(|_| !self.source_file().same_line(begin_keyword.start_offset(), end_keyword.end_offset())) {
            let first = &statements[0];
            let condition = format!(" {} {}", String::from_utf8_lossy(keyword.as_slice()), self.source_file().node(&predicate));
            let condition_line = self.source_file().full_line_range(keyword.start_offset()..predicate.location().end_offset());
            let offense = begin_keyword.start_offset()..begin_keyword.end_offset();
            add_offense!(self, offense.clone(), message: "Redundant `begin` block detected.", |corrector| {
                corrector.remove(offense);
                corrector.replace(first.location().end_offset()..first.location().end_offset(), condition);
                corrector.remove(condition_line);
                let _ = parent_location;
            });
        } else {
            let offense = begin_keyword.start_offset()..begin_keyword.end_offset();
            let closing = end_keyword.start_offset()..end_keyword.end_offset();
            add_offense!(self, offense.clone(), message: "Redundant `begin` block detected.", |corrector| {
                corrector.remove(offense);
                corrector.remove(closing);
            });
        }
    }
}

fn semantic_begin_parent<'a, 'pr>(ancestors: &'a [Node<'pr>]) -> Option<&'a Node<'pr>> {
    ancestors.iter().rev().find(|node| node.as_statements_node().is_none())
}

fn assignment_around_begin(parent: &Node<'_>, begin: usize, file: SourceFile<'_>) -> Option<std::ops::Range<usize>> {
    let location = parent.location();
    if location.start_offset() >= begin || begin > location.end_offset() { return None; }
    let before = file.slice(location.start_offset()..begin)?.trim_end();
    (before.ends_with('=') || before.ends_with("||=") || before.ends_with("&&=")).then_some(location.start_offset()..location.end_offset())
}

fn branch_or_loop_parent(node: &Node<'_>) -> bool {
    node.as_if_node().is_some() || node.as_unless_node().is_some()
        || node.as_while_node().is_some() || node.as_until_node().is_some()
        || node.as_when_node().is_some() || node.as_in_node().is_some()
        || node.as_else_node().is_some()
}

fn parent_has_only_begin(parent: &Node<'_>, begin: &BeginNode<'_>) -> bool {
    let statements = if let Some(program) = parent.as_program_node() {
        Some(program.statements())
    } else if let Some(definition) = parent.as_def_node() {
        definition.body().and_then(|body| body.as_statements_node())
    } else if let Some(block) = parent.as_block_node() {
        block.body().and_then(|body| body.as_statements_node())
    } else if let Some(condition) = parent.as_if_node() {
        condition.statements()
    } else if let Some(condition) = parent.as_unless_node() {
        condition.statements()
    } else if let Some(branch) = parent.as_when_node() {
        branch.statements()
    } else if let Some(branch) = parent.as_in_node() {
        branch.statements()
    } else if let Some(branch) = parent.as_else_node() {
        branch.statements()
    } else if let Some(loop_node) = parent.as_while_node() {
        loop_node.statements()
    } else if let Some(loop_node) = parent.as_until_node() {
        loop_node.statements()
    } else {
        None
    };
    statements.is_some_and(|statements| {
        let body = statements.body();
        body.len() == 1
            && body.first().is_some_and(|child| {
                child.location().start_offset() == begin.location().start_offset()
                    && child.location().end_offset() == begin.location().end_offset()
            })
    })
}

fn direct_block_is_lambda(ancestors: &[Node<'_>]) -> bool {
    let mut semantic = ancestors
        .iter()
        .rev()
        .filter(|node| node.as_statements_node().is_none());
    if semantic.next().is_none_or(|node| node.as_block_node().is_none()) {
        return false;
    }
    semantic.next().is_some_and(|node| node.as_lambda_node().is_some())
}

fn modifier_parent_parts<'pr>(node: &Node<'pr>) -> Option<(std::ops::Range<usize>, ruby_prism::Location<'pr>, Node<'pr>)> {
    if let Some(condition) = node.as_if_node() {
        let keyword = condition.if_keyword_loc()?;
        return Some((condition.location().start_offset()..condition.location().end_offset(), keyword, condition.predicate()));
    }
    if let Some(condition) = node.as_unless_node() {
        return Some((condition.location().start_offset()..condition.location().end_offset(), condition.keyword_loc(), condition.predicate()));
    }
    None
}

fn enclosing_assignment_begin<'pr>(ancestors: &[Node<'pr>], file: SourceFile<'_>) -> Option<BeginNode<'pr>> {
    for (index, ancestor) in ancestors.iter().enumerate().rev() {
        let Some(outer) = ancestor.as_begin_node() else { continue };
        let outer_begin = outer.begin_keyword_loc()?;
        let parent = semantic_begin_parent(&ancestors[..index])?;
        if assignment_around_begin(parent, outer_begin.start_offset(), file).is_some() {
            return Some(outer);
        }
    }
    None
}

fn collapse_nested_assignment_begin(source: &str) -> String {
    let Some(begin) = source.find("begin") else { return source.to_string() };
    let Some(end) = source.rfind("end") else { return source.to_string() };
    if begin >= end { return source.to_string(); }
    let before = source[..begin].to_string();
    let body = source[begin + 5..end].trim();
    if body.lines().count() == 1 {
        format!("{before}{}", collapse_nested_assignment_begin(body))
    } else {
        source.to_string()
    }
}
