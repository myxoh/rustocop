// RuboCop 1.87.0
// Source: lib/rubocop/cop/ignored_node.rb
// Source SHA-256: 6ada9cefc156452c121f91f7e85e34359c8d59ceb055ff346b7a32d098b34462

use std::ops::Range;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct NodeIdentity(pub(crate) usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NodeLocation {
    pub(crate) expression: Range<usize>,
    pub(crate) heredoc_end: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NodeRef {
    pub(crate) identity: NodeIdentity,
    pub(crate) location: NodeLocation,
}

#[derive(Default)]
pub(crate) struct IgnoredNode {
    ignored_nodes: Vec<NodeRef>,
}

impl IgnoredNode {
    pub(crate) fn ignore_node(&mut self, node: NodeRef) {
        self.ignored_nodes.push(node);
    }

    pub(crate) fn part_of_ignored_node(&self, node: &NodeRef) -> bool {
        self.ignored_nodes.iter().any(|ignored| {
            if ignored.location.expression.start > node.location.expression.start {
                return false;
            }
            ignored
                .location
                .heredoc_end
                .unwrap_or(ignored.location.expression.end)
                >= node.location.expression.end
        })
    }

    pub(crate) fn ignored_node(&self, node: &NodeRef) -> bool {
        self.ignored_nodes
            .iter()
            .any(|ignored| ignored.identity == node.identity)
    }

    pub(crate) fn ignored_nodes(&self) -> &[NodeRef] {
        &self.ignored_nodes
    }
}
