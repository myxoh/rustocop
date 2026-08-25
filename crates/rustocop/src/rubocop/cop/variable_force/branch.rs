// RuboCop 1.87.0
// Source: lib/rubocop/cop/variable_force/branch.rb
// Source SHA-256: 9e1146dca1c84c350032aea5e8f4338d1e3fe1d972e1d942e81839f0802bf661

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::RangeInclusive;

use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::cop::variable_force::scope::Scope;

const CLASSES: [&str; 15] = [
    "if",
    "while",
    "until",
    "while_post",
    "until_post",
    "case",
    "case_match",
    "for",
    "and",
    "and_asgn",
    "or",
    "or_asgn",
    "op_asgn",
    "rescue",
    "ensure",
];

#[derive(Clone, Copy)]
pub(crate) struct Branch<'ast, 'scope> {
    child_node: NodeRef<'ast>,
    scope: Option<&'scope Scope<'ast>>,
}

impl<'ast, 'scope> Branch<'ast, 'scope> {
    pub(crate) fn child_node(&self) -> NodeRef<'ast> {
        self.child_node
    }
    pub(crate) fn scope(&self) -> Option<&'scope Scope<'ast>> {
        self.scope
    }

    pub(crate) fn predicate(&self, name: &str) -> bool {
        let Some(control) = self.control_node() else {
            return false;
        };
        let index = self.child_node.sibling_index();
        match (name, control.kind()) {
            ("truthy_body", "if")
            | ("loop_body", "while" | "until" | "while_post" | "until_post") => index == Some(1),
            ("falsey_body", "if") | ("else_body", "case" | "case_match") => index == Some(2),
            ("target", "for")
            | ("left_body", "and" | "and_asgn" | "or" | "or_asgn" | "op_asgn") => index == Some(0),
            ("collection", "for")
            | ("right_body", "and" | "and_asgn" | "or" | "or_asgn" | "op_asgn") => index == Some(1),
            ("element", "case" | "case_match") | ("main_body", "rescue" | "ensure") => {
                index == Some(0)
            }
            ("when_clause", "case")
            | ("in_pattern", "case_match")
            | ("rescue_clause", "rescue") => index.is_some_and(|index| index > 0),
            ("ensure_body", "ensure") => index == Some(control.children().len().saturating_sub(1)),
            _ => false,
        }
    }
    pub(crate) fn of(
        target_node: NodeRef<'ast>,
        scope: Option<&'scope Scope<'ast>>,
    ) -> Option<Self> {
        for node in std::iter::once(target_node).chain(target_node.ancestors()) {
            let parent = node.parent()?;
            if scope.is_some_and(|scope| !scope.includes(node)) {
                return None;
            }
            if Self::classes().contains(&parent.kind()) {
                let branch = Self {
                    child_node: node,
                    scope,
                };
                if branch.branched() {
                    return Some(branch);
                }
            }
        }
        None
    }

    pub(crate) const fn classes() -> &'static [&'static str] {
        &CLASSES
    }

    pub(crate) fn inherited(subclass: &str) -> Vec<String> {
        Self::classes()
            .iter()
            .map(|name| (*name).to_owned())
            .chain(std::iter::once(subclass.to_owned()))
            .collect()
    }

    pub(crate) fn branch_type(class_name: &str) -> String {
        let short = class_name.rsplit("::").next().unwrap_or(class_name);
        let mut output = String::new();
        for (index, character) in short.chars().enumerate() {
            if index > 0 && character.is_uppercase() {
                output.push('_');
            }
            output.extend(character.to_lowercase());
        }
        output
    }

    pub(crate) fn define_predicate(&self, child_index: RangeInclusive<usize>) -> bool {
        self.child_node
            .sibling_index()
            .is_some_and(|index| child_index.contains(&index))
    }

    pub(crate) fn control_node(&self) -> Option<NodeRef<'ast>> {
        self.child_node.parent()
    }

    pub(crate) fn parent(&self) -> Option<Self> {
        Self::of(self.control_node()?, self.scope)
    }

    pub(crate) fn each_ancestor(&self, include_self: bool) -> Vec<Self> {
        let mut ancestors = Vec::new();
        if include_self {
            ancestors.push(*self);
        }
        self.scan_ancestors(&mut ancestors);
        ancestors
    }

    pub(crate) fn branched(&self) -> bool {
        !self.always_run()
    }

    pub(crate) fn always_run(&self) -> bool {
        let Some(control) = self.control_node() else {
            return true;
        };
        let index = self.child_node.sibling_index();
        match control.kind() {
            "if" | "while" | "until" | "while_post" | "until_post" => self.conditional_clause(),
            "case" | "case_match" => index == Some(0),
            "for" => matches!(index, Some(0 | 1)),
            "and" | "and_asgn" | "or" | "or_asgn" | "op_asgn" => index == Some(0),
            "ensure" => index == Some(control.children().len().saturating_sub(1)),
            "rescue" => false,
            _ => true,
        }
    }

    pub(crate) fn may_jump_to_other_branch(&self) -> bool {
        self.control_node()
            .is_some_and(|control| matches!(control.kind(), "rescue" | "ensure"))
            && self.child_node.sibling_index() == Some(0)
    }

    pub(crate) fn may_run_incompletely(&self) -> bool {
        self.may_jump_to_other_branch()
    }

    pub(crate) fn exclusive_with(&self, other: Option<Self>) -> bool {
        let Some(other) = other else {
            return false;
        };
        if self.may_jump_to_other_branch() {
            return false;
        }
        for ancestor in other.each_ancestor(true) {
            if self.control_node() == ancestor.control_node() {
                return self.child_node != ancestor.child_node;
            }
        }
        self.parent()
            .is_some_and(|parent| parent.exclusive_with(Some(other)))
    }

    pub(crate) fn equivalent(&self, other: Option<Self>) -> bool {
        other.is_some_and(|other| {
            self.control_node() == other.control_node() && self.child_node == other.child_node
        })
    }

    pub(crate) fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.control_node().map(NodeRef::id).hash(&mut hasher);
        self.control_node().map(NodeRef::id).hash(&mut hasher);
        hasher.finish()
    }

    pub(crate) fn scan_ancestors(&self, ancestors: &mut Vec<Self>) {
        let mut branch = *self;
        while let Some(parent) = branch.parent() {
            ancestors.push(parent);
            branch = parent;
        }
    }

    pub(crate) fn conditional_clause(&self) -> bool {
        self.define_predicate(0..=0)
    }
}

#[cfg(test)]
mod spec;
