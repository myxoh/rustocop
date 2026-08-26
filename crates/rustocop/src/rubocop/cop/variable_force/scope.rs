// RuboCop 1.87.0
// Source: lib/rubocop/cop/variable_force/scope.rb
// Source SHA-256: 5bcf514b3c4377f3bd69e9c794587b234f2cf2fa7de1b1c72b6109af7a552fbe

use std::collections::BTreeMap;

use crate::rubocop::ast::node::core::NodeRef;

const SCOPE_TYPES: [&str; 8] = [
    "def", "defs", "module", "class", "sclass", "block", "numblock", "itblock",
];

pub(crate) struct Scope<'ast> {
    node: NodeRef<'ast>,
    pub(crate) variables: BTreeMap<String, usize>,
    naked_top_level: bool,
}

impl<'ast> Scope<'ast> {
    pub(crate) fn node(&self) -> NodeRef<'ast> {
        self.node
    }

    pub(crate) fn variables(&self) -> &BTreeMap<String, usize> {
        &self.variables
    }

    pub(crate) fn initialize(node: NodeRef<'ast>) -> Result<Self, String> {
        let naked_top_level = !SCOPE_TYPES.contains(&node.kind());
        if naked_top_level && node.parent().is_some() {
            return Err(format!(
                "Node type must be any of {SCOPE_TYPES:?}, passed {}",
                node.kind()
            ));
        }
        Ok(Self {
            node,
            variables: BTreeMap::new(),
            naked_top_level,
        })
    }

    pub(crate) fn equivalent(&self, other: &Self) -> bool {
        self.node == other.node
    }

    pub(crate) fn naked_top_level(&self) -> bool {
        self.naked_top_level
    }

    pub(crate) fn name(&self) -> Option<&'ast str> {
        self.node.method_name()
    }

    pub(crate) fn body_node(&self) -> Option<NodeRef<'ast>> {
        if self.naked_top_level() {
            return Some(self.node);
        }
        let child_index = match self.node.kind() {
            "module" | "sclass" => 1,
            "def" | "class" | "block" | "numblock" | "itblock" => 2,
            "defs" => 3,
            _ => return None,
        };
        self.node.node_child(child_index)
    }

    pub(crate) fn includes(&self, target_node: NodeRef<'_>) -> bool {
        !self.belong_to_outer_scope(target_node) && !self.belong_to_inner_scope(target_node)
    }

    pub(crate) fn each_node(&self) -> Vec<NodeRef<'ast>> {
        let mut nodes = Vec::new();
        if self.naked_top_level() {
            nodes.push(self.node);
        }
        self.scan_node(self.node, &mut nodes);
        nodes
    }

    pub(crate) fn scan_node(&self, node: NodeRef<'ast>, nodes: &mut Vec<NodeRef<'ast>>) {
        for child in node.child_nodes() {
            if !self.includes(child) {
                continue;
            }
            nodes.push(child);
            self.scan_node(child, nodes);
        }
    }

    pub(crate) fn belong_to_outer_scope(&self, target_node: NodeRef<'_>) -> bool {
        if !self.naked_top_level() && target_node == self.node {
            return true;
        }
        if self.ancestor_node(target_node) {
            return true;
        }
        let Some(parent) = target_node.parent().filter(|parent| *parent == self.node) else {
            return false;
        };
        outer_scope_child_indices(parent.kind()).is_some_and(|indices| {
            target_node
                .sibling_index()
                .is_some_and(|index| indices.contains(&index))
        })
    }

    pub(crate) fn belong_to_inner_scope(&self, target_node: NodeRef<'_>) -> bool {
        let Some(parent) = target_node.parent() else {
            return false;
        };
        if parent == self.node || !SCOPE_TYPES.contains(&parent.kind()) {
            return false;
        }
        outer_scope_child_indices(parent.kind()).is_none_or(|indices| {
            !target_node
                .sibling_index()
                .is_some_and(|index| indices.contains(&index))
        })
    }

    pub(crate) fn ancestor_node(&self, target_node: NodeRef<'_>) -> bool {
        self.node.ancestors().contains(&target_node)
    }
}

fn outer_scope_child_indices(kind: &str) -> Option<std::ops::RangeInclusive<usize>> {
    match kind {
        "defs" | "module" | "sclass" | "block" => Some(0..=0),
        "class" => Some(0..=1),
        _ => None,
    }
}

#[cfg(test)]
mod spec;
