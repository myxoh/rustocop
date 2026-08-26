// RuboCop 1.87.0
// Source: lib/rubocop/cop/variable_force.rb
// Source SHA-256: f6b843c42bc19bf9bd8a130951a183a08d8b3370981f499698bf04d2ff5e0328

use std::collections::BTreeSet;

use regex::Regex;

use crate::rubocop::ast::node::core::NodeRef;

use super::framework::{VariableScopeKind, VariableTable};

pub(crate) mod branch;
pub(crate) mod scope;
pub(crate) mod variable_table;

const ARGUMENT_DECLARATION_TYPES: [&str; 9] = [
    "arg",
    "optarg",
    "restarg",
    "kwarg",
    "kwoptarg",
    "kwrestarg",
    "blockarg",
    "shadowarg",
    "procarg0",
];
const OPERATOR_ASSIGNMENT_TYPES: [&str; 3] = ["or_asgn", "and_asgn", "op_asgn"];
const LOOP_TYPES: [&str; 5] = ["while_post", "until_post", "while", "until", "for"];
const SCOPE_TYPES: [&str; 8] = [
    "block", "numblock", "itblock", "class", "sclass", "defs", "module", "def",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DescendantReference<'ast> {
    Variable(&'ast str),
    Assignment(NodeRef<'ast>),
}

pub(crate) trait VariableForceHandler {
    fn on_variable_force_event(&mut self, hook: &str, table: &VariableTable);
}

impl DescendantReference<'_> {
    fn assignment(&self) -> bool {
        matches!(self, Self::Assignment(_))
    }

    pub(crate) fn name(&self) -> Option<&str> {
        match self {
            Self::Variable(name) => Some(name),
            Self::Assignment(_) => None,
        }
    }

    pub(crate) fn node(&self) -> Option<NodeRef<'_>> {
        match self {
            Self::Variable(_) => None,
            Self::Assignment(node) => Some(*node),
        }
    }
}

pub(crate) struct VariableForce {
    variable_table: VariableTable,
    scanned_nodes: BTreeSet<(usize, usize, String)>,
    events: Vec<&'static str>,
    handlers: Vec<Box<dyn VariableForceHandler>>,
}

impl VariableForce {
    pub(crate) fn new() -> Self {
        Self {
            variable_table: VariableTable::new(),
            scanned_nodes: BTreeSet::new(),
            events: Vec::new(),
            handlers: Vec::new(),
        }
    }

    pub(crate) fn variable_table(&self) -> &VariableTable {
        &self.variable_table
    }

    pub(crate) fn investigate(&mut self, root: NodeRef<'_>) {
        self.variable_table = VariableTable::new();
        self.scanned_nodes.clear();
        self.events.clear();
        self.notify_handlers("before_entering_scope");
        self.notify_handlers("after_entering_scope");
        self.process_node(root);
        self.notify_handlers("before_leaving_scope");
        self.notify_handlers("after_leaving_scope");
    }

    pub(crate) fn notify_handlers(&mut self, hook: &'static str) {
        self.events.push(hook);
        for handler in &mut self.handlers {
            handler.on_variable_force_event(hook, &self.variable_table);
        }
    }

    pub(crate) fn events(&self) -> &[&'static str] {
        &self.events
    }

    pub(crate) fn add_handler(&mut self, handler: Box<dyn VariableForceHandler>) {
        self.handlers.push(handler);
    }

    fn process_node(&mut self, node: NodeRef<'_>) {
        let skip = match self.node_handler_method_name(node) {
            Some("process_variable_assignment") => self.process_variable_assignment(node),
            Some("process_regexp_named_captures") => self.process_regexp_named_captures(node),
            Some("process_pattern_match_variable") => self.process_pattern_match_variable(node),
            Some("process_variable_multiple_assignment") => {
                self.process_variable_multiple_assignment(node)
            }
            Some("process_variable_referencing") => self.process_variable_referencing(node),
            Some("process_rescue") => self.process_rescue(node),
            Some("process_zero_arity_super") => self.process_zero_arity_super(node),
            Some("process_send") => self.process_send(node),
            Some("process_variable_declaration") => self.process_variable_declaration(node),
            Some("process_variable_operator_assignment") => {
                self.process_variable_operator_assignment(node)
            }
            Some("process_loop") => self.process_loop(node),
            Some("process_scope") => self.process_scope(node),
            _ => false,
        };
        if !skip {
            self.process_children(node);
        }
    }

    fn inspect_variables_in_scope(&mut self, scope_node: NodeRef<'_>) {
        let kind = match scope_node.kind() {
            "block" | "numblock" | "itblock" => VariableScopeKind::Block,
            "def" | "defs" => VariableScopeKind::Method,
            _ => VariableScopeKind::ClassOrModule,
        };
        self.notify_handlers("before_entering_scope");
        self.variable_table.enter_scope_kind(kind);
        self.notify_handlers("after_entering_scope");
        self.process_children(scope_node);
        self.notify_handlers("before_leaving_scope");
        self.variable_table.leave_scope();
        self.notify_handlers("after_leaving_scope");
    }

    fn process_children(&mut self, origin_node: NodeRef<'_>) {
        for child in origin_node.child_nodes() {
            if !self.scanned_node(child) {
                self.process_node(child);
            }
        }
    }

    fn skip_children(&self) -> bool {
        true
    }

    fn node_handler_method_name(&self, node: NodeRef<'_>) -> Option<&'static str> {
        match node.kind() {
            "lvasgn" => Some("process_variable_assignment"),
            "match_with_lvasgn" => Some("process_regexp_named_captures"),
            "match_var" => Some("process_pattern_match_variable"),
            "masgn" => Some("process_variable_multiple_assignment"),
            "lvar" => Some("process_variable_referencing"),
            "rescue" => Some("process_rescue"),
            "zsuper" => Some("process_zero_arity_super"),
            "send" => Some("process_send"),
            kind if ARGUMENT_DECLARATION_TYPES.contains(&kind) => {
                Some("process_variable_declaration")
            }
            kind if OPERATOR_ASSIGNMENT_TYPES.contains(&kind) => {
                Some("process_variable_operator_assignment")
            }
            kind if LOOP_TYPES.contains(&kind) => Some("process_loop"),
            kind if SCOPE_TYPES.contains(&kind) => Some("process_scope"),
            _ => None,
        }
    }

    fn process_variable_declaration(&mut self, node: NodeRef<'_>) -> bool {
        if let Some(name) = node.name() {
            self.notify_handlers("before_declaring_variable");
            self.variable_table
                .declare(name, node.source_range().unwrap_or(0..0), node.kind());
            self.notify_handlers("after_declaring_variable");
        }
        false
    }

    fn process_variable_assignment(&mut self, node: NodeRef<'_>) -> bool {
        let Some(name) = node.name() else {
            return false;
        };
        let range = node.source_range().unwrap_or(0..0);
        if !self.variable_table.variable_exists(name) {
            self.variable_table
                .declare(name, range.clone(), node.kind());
        }
        self.process_children(node);
        self.variable_table.assign_kind(name, range, node.kind());
        self.skip_children()
    }

    fn process_regexp_named_captures(&mut self, node: NodeRef<'_>) -> bool {
        let children = node.child_nodes();
        let Some(regexp) = children.first().copied() else {
            return false;
        };
        let names = self.regexp_captured_names(regexp);
        for name in &names {
            if !self.variable_table.variable_exists(name) {
                self.variable_table
                    .declare(name, node.source_range().unwrap_or(0..0), node.kind());
            }
        }
        if let Some(rhs) = children.get(1).copied() {
            self.process_node(rhs);
        }
        self.process_node(regexp);
        for name in names {
            self.variable_table.assign_kind(
                &name,
                node.source_range().unwrap_or(0..0),
                node.kind(),
            );
        }
        self.skip_children()
    }

    fn process_pattern_match_variable(&mut self, node: NodeRef<'_>) -> bool {
        if let Some(name) = node.name() {
            if !self.variable_table.variable_exists(name) {
                self.variable_table
                    .declare(name, node.source_range().unwrap_or(0..0), node.kind());
            }
        }
        self.skip_children()
    }

    fn regexp_captured_names(&self, node: NodeRef<'_>) -> Vec<String> {
        Regex::new(r"\(\?<([A-Za-z_]\w*)>")
            .unwrap()
            .captures_iter(&node.regexp_content())
            .filter_map(|capture| capture.get(1).map(|name| name.as_str().to_owned()))
            .collect()
    }

    fn process_variable_operator_assignment(&mut self, node: NodeRef<'_>) -> bool {
        let Some(lhs) = node.node_child(0) else {
            return false;
        };
        if lhs.kind() != "lvasgn" {
            return false;
        }
        let Some(name) = lhs.name() else { return false };
        let lhs_range = lhs.source_range().unwrap_or(0..0);
        if !self.variable_table.variable_exists(name) {
            self.variable_table
                .declare(name, lhs_range.clone(), lhs.kind());
        }
        self.variable_table
            .reference(name, node.source_range().unwrap_or(0..0));
        if let Some(rhs) = node.rhs() {
            self.process_node(rhs);
        }
        self.variable_table.assign_kind(name, lhs_range, lhs.kind());
        self.skip_children()
    }

    fn process_variable_multiple_assignment(&mut self, node: NodeRef<'_>) -> bool {
        if let Some(rhs) = node.node_child(1) {
            self.process_node(rhs);
        }
        if let Some(lhs) = node.node_child(0) {
            self.process_node(lhs);
        }
        self.skip_children()
    }

    fn process_variable_referencing(&mut self, node: NodeRef<'_>) -> bool {
        if let Some(name) = node.name() {
            self.variable_table
                .reference(name, node.source_range().unwrap_or(0..0));
        }
        false
    }

    fn process_loop(&mut self, node: NodeRef<'_>) -> bool {
        match node.kind() {
            "while_post" | "until_post" => {
                if let Some(body) = node.node_child(1) {
                    self.process_node(body);
                }
                if let Some(condition) = node.node_child(0) {
                    self.process_node(condition);
                }
            }
            "for" => {
                if let Some(collection) = node.collection() {
                    self.process_node(collection);
                }
                if let Some(variable) = node.node_child(0) {
                    self.process_node(variable);
                }
                if let Some(body) = node.body() {
                    self.process_node(body);
                }
            }
            _ => self.process_children(node),
        }
        self.mark_assignments_as_referenced_in_loop(node);
        self.skip_children()
    }

    fn process_rescue(&mut self, node: NodeRef<'_>) -> bool {
        let retry = node
            .each_descendant(&["retry"])
            .into_iter()
            .next()
            .is_some();
        retry && self.process_loop(node)
    }

    fn process_zero_arity_super(&mut self, node: NodeRef<'_>) -> bool {
        let names = self
            .variable_table
            .accessible_variables()
            .into_iter()
            .filter(|variable| variable.argument())
            .map(|variable| variable.name.clone())
            .collect::<Vec<_>>();
        for name in names {
            self.variable_table
                .reference(&name, node.source_range().unwrap_or(0..0));
        }
        false
    }

    fn process_scope(&mut self, node: NodeRef<'_>) -> bool {
        if node.kind() != "def" {
            for twisted in self.twisted_nodes(node) {
                self.process_node(twisted);
                self.scanned_nodes.insert(Self::node_identity(twisted));
            }
        }
        self.inspect_variables_in_scope(node);
        self.skip_children()
    }

    fn twisted_nodes<'ast>(&self, node: NodeRef<'ast>) -> Vec<NodeRef<'ast>> {
        let mut nodes = node.node_child(0).into_iter().collect::<Vec<_>>();
        if node.kind() == "class" {
            nodes.extend(node.node_child(1));
        }
        nodes
    }

    fn process_send(&mut self, node: NodeRef<'_>) -> bool {
        if node.method_name() != Some("binding") || !node.arguments().is_empty() {
            return false;
        }
        let names = self
            .variable_table
            .accessible_variables()
            .into_iter()
            .map(|variable| variable.name.clone())
            .collect::<Vec<_>>();
        for name in names {
            self.variable_table
                .reference(&name, node.source_range().unwrap_or(0..0));
        }
        false
    }

    fn mark_assignments_as_referenced_in_loop(&mut self, node: NodeRef<'_>) {
        let (references, assignments) = self.find_variables_in_loop(node);
        self.reference_assignments(&references, &assignments, node);
    }

    fn find_variables_in_loop<'ast>(
        &self,
        loop_node: NodeRef<'ast>,
    ) -> (Vec<&'ast str>, Vec<NodeRef<'ast>>) {
        let mut referenced = Vec::new();
        let mut assignments = Vec::new();
        self.each_descendant_reference(loop_node, |reference| match reference {
            DescendantReference::Variable(name) => referenced.push(name),
            DescendantReference::Assignment(node) => assignments.push(node),
        });
        (referenced, assignments)
    }

    fn each_descendant_reference<'ast>(
        &self,
        loop_node: NodeRef<'ast>,
        mut callback: impl FnMut(DescendantReference<'ast>),
    ) {
        for node in loop_node.descendants() {
            if let Some(reference) = self.descendant_reference(node) {
                callback(reference);
            }
        }
    }

    fn descendant_reference<'ast>(&self, node: NodeRef<'ast>) -> Option<DescendantReference<'ast>> {
        match node.kind() {
            "lvar" => node.name().map(DescendantReference::Variable),
            "lvasgn" => Some(DescendantReference::Assignment(node)),
            "op_asgn" | "or_asgn" | "and_asgn" => node
                .node_child(0)
                .filter(|lhs| lhs.kind() == "lvasgn")
                .and_then(NodeRef::name)
                .map(DescendantReference::Variable),
            _ => None,
        }
    }

    fn reference_assignments(
        &mut self,
        referenced_names: &[&str],
        assignments: &[NodeRef<'_>],
        loop_node: NodeRef<'_>,
    ) {
        let loop_range = loop_node.source_range().unwrap_or(0..0);
        for name in referenced_names {
            if assignments
                .iter()
                .any(|assignment| assignment.name() == Some(name))
            {
                self.variable_table.reference(name, loop_range.clone());
            }
        }
    }

    fn scanned_node(&self, node: NodeRef<'_>) -> bool {
        self.scanned_nodes.contains(&Self::node_identity(node))
    }

    fn scanned_nodes(&self) -> usize {
        self.scanned_nodes.len()
    }

    fn node_identity(node: NodeRef<'_>) -> (usize, usize, String) {
        let range = node.source_range().unwrap_or(0..0);
        (range.start, range.end, node.kind().to_owned())
    }
}

impl Default for VariableForce {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

    #[test]
    fn ports_rhs_first_scopes_loop_references_and_handler_dispatch() {
        let source = ProcessedSource::new(
            "foo = 1\nfoo = foo + 1\nwhile condition\n  foo = foo + 1\nend\n",
            3.4,
            None,
            ParserEngine::Prism,
        )
        .unwrap();
        let mut force = VariableForce::new();
        force.investigate(source.ast().unwrap());
        let foo = force
            .variable_table()
            .variables()
            .into_iter()
            .find(|variable| variable.name == "foo")
            .unwrap();
        assert_eq!(foo.assignments.len(), 3);
        assert!(foo.references.len() >= 2);
        assert_eq!(force.node_handler_method_name(source.ast().unwrap()), None);
        assert_eq!(force.scanned_nodes(), 0);
        assert!(!DescendantReference::Variable("foo").assignment());
    }
}
