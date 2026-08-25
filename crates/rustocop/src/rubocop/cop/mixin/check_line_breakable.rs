// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/check_line_breakable.rb
// Source SHA-256: 81f0cffda303555084c6141bdac1e6bc5d02d32bdca6dd61b6f285d32cd8301d

use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::processed_source::ProcessedSource;

pub(crate) struct CheckLineBreakable<'processed, 'source> {
    processed_source: &'processed ProcessedSource<'source>,
    configured_max: Option<usize>,
}

impl<'processed, 'source> CheckLineBreakable<'processed, 'source> {
    pub(crate) const fn new(
        processed_source: &'processed ProcessedSource<'source>,
        configured_max: Option<usize>,
    ) -> Self {
        Self {
            processed_source,
            configured_max,
        }
    }

    pub(crate) fn extract_breakable_node<'ast>(
        &self,
        node: NodeRef<'ast>,
        max: usize,
    ) -> Option<NodeRef<'ast>> {
        let elements = if node.call_type() {
            if self.chained_to_heredoc(node) {
                return None;
            }
            self.process_args(node.arguments())
        } else if matches!(node.kind(), "def" | "defs") {
            node.arguments_node()
                .map_or_else(Vec::new, NodeRef::child_nodes)
        } else if matches!(node.kind(), "array" | "hash") {
            node.child_nodes()
        } else {
            return None;
        };
        self.extract_breakable_node_from_elements(node, &elements, max)
    }

    pub(crate) fn extract_breakable_node_from_elements<'ast>(
        &self,
        node: NodeRef<'ast>,
        elements: &[NodeRef<'ast>],
        max: usize,
    ) -> Option<NodeRef<'ast>> {
        if !self.breakable_collection(node, elements) || self.safe_to_ignore(node) {
            return None;
        }
        let line = self
            .processed_source
            .lines()
            .get(node.first_line().saturating_sub(1))?;
        if self.processed_source.line_with_comment(node.first_line()) || line.chars().count() <= max
        {
            return None;
        }
        self.extract_first_element_over_column_limit(node, elements, max)
    }

    pub(crate) fn extract_first_element_over_column_limit<'ast>(
        &self,
        node: NodeRef<'ast>,
        elements: &[NodeRef<'ast>],
        max: usize,
    ) -> Option<NodeRef<'ast>> {
        let elements = if node.call_type()
            && !node.parenthesized_call()
            && !self.first_argument_is_heredoc(node)
        {
            elements.get(1..).unwrap_or_default()
        } else {
            elements
        };
        let mut index = 0;
        while elements
            .get(index)
            .is_some_and(|element| self.within_column_limit(Some(*element), max, node.first_line()))
        {
            index += 1;
        }
        index = self.shift_elements_for_heredoc_arg(node, elements, index)?;
        if index == 0 {
            elements.first().copied()
        } else {
            elements.get(index - 1).copied()
        }
    }

    pub(crate) fn first_argument_is_heredoc(&self, node: NodeRef<'_>) -> bool {
        node.first_argument().is_some_and(NodeRef::heredoc)
    }

    pub(crate) fn shift_elements_for_heredoc_arg(
        &self,
        node: NodeRef<'_>,
        elements: &[NodeRef<'_>],
        index: usize,
    ) -> Option<usize> {
        if !node.call_type() && node.kind() != "array" {
            return Some(index);
        }
        let Some(heredoc_index) = elements.iter().position(|argument| argument.heredoc()) else {
            return Some(index);
        };
        if heredoc_index == 0 {
            return None;
        }
        Some(if heredoc_index >= index {
            index
        } else {
            heredoc_index + 1
        })
    }

    pub(crate) fn within_column_limit(
        &self,
        element: Option<NodeRef<'_>>,
        max: usize,
        line: usize,
    ) -> bool {
        element.is_some_and(|element| element.column() <= max && element.first_line() == line)
    }

    pub(crate) fn safe_to_ignore(&self, node: NodeRef<'_>) -> bool {
        self.configured_max.is_none()
            || self.already_on_multiple_lines(node)
            || self.contained_by_breakable_collection_on_same_line(node)
            || self.contained_by_multiline_collection_that_could_be_broken_up(node)
    }

    pub(crate) fn breakable_collection(&self, node: NodeRef<'_>, elements: &[NodeRef<'_>]) -> bool {
        (node.kind() != "hash" || node.loc("begin").is_some()) && elements.len() >= 2
    }

    pub(crate) fn contained_by_breakable_collection_on_same_line(&self, node: NodeRef<'_>) -> bool {
        for ancestor in node.ancestors() {
            if ancestor.first_line() != node.first_line() {
                break;
            }
            let elements = if matches!(ancestor.kind(), "hash" | "array") {
                ancestor.child_nodes()
            } else if ancestor.call_type() {
                self.process_args(ancestor.arguments())
            } else {
                continue;
            };
            if self.breakable_collection(ancestor, &elements) {
                return true;
            }
        }
        false
    }

    pub(crate) fn contained_by_multiline_collection_that_could_be_broken_up(
        &self,
        node: NodeRef<'_>,
    ) -> bool {
        for ancestor in node.ancestors() {
            let elements = if matches!(ancestor.kind(), "hash" | "array") {
                ancestor.child_nodes()
            } else if ancestor.call_type() {
                self.process_args(ancestor.arguments())
            } else {
                continue;
            };
            if self.breakable_collection(ancestor, &elements)
                && self.children_could_be_broken_up(&elements)
            {
                return true;
            }
        }
        false
    }

    pub(crate) fn children_could_be_broken_up(&self, children: &[NodeRef<'_>]) -> bool {
        if self.all_on_same_line(children) {
            return false;
        }
        let mut last_seen_line = 0;
        for child in children {
            if last_seen_line >= child.first_line() {
                return true;
            }
            last_seen_line = child.last_line();
        }
        false
    }

    pub(crate) fn all_on_same_line(&self, nodes: &[NodeRef<'_>]) -> bool {
        nodes
            .first()
            .zip(nodes.last())
            .is_none_or(|(first, last)| first.first_line() == last.last_line())
    }

    pub(crate) fn process_args<'ast>(&self, args: Vec<NodeRef<'ast>>) -> Vec<NodeRef<'ast>> {
        let Some(last) = args.last().copied() else {
            return args;
        };
        if last.kind() == "hash" && !last.braces() {
            args[..args.len() - 1]
                .iter()
                .copied()
                .chain(last.child_nodes())
                .collect()
        } else {
            args
        }
    }

    pub(crate) fn already_on_multiple_lines(&self, node: NodeRef<'_>) -> bool {
        if matches!(node.kind(), "def" | "defs") {
            return node
                .last_argument()
                .is_some_and(|last| node.first_line() != last.last_line());
        }
        node.multiline()
    }

    pub(crate) fn chained_to_heredoc(&self, mut node: NodeRef<'_>) -> bool {
        while let Some(receiver) = node.receiver() {
            if matches!(receiver.kind(), "str" | "dstr") && receiver.heredoc() {
                return true;
            }
            node = receiver;
        }
        false
    }
}

#[cfg(test)]
mod spec;
