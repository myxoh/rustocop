// RuboCop 1.87.0
// Source: lib/rubocop/cop/variable_force/variable_table.rb
// Source SHA-256: e654e1b6a8d46416e11d03550a164560b839d0ab39b2ac74ad2c05c4cdd50b84

use std::collections::BTreeMap;

use crate::rubocop::ast::node::core::NodeRef;

#[derive(Clone, Debug)]
pub(crate) struct TableVariable<'ast> {
    pub(crate) name: String,
    pub(crate) declaration_node: NodeRef<'ast>,
    pub(crate) scope_level: usize,
    pub(crate) assignments: Vec<NodeRef<'ast>>,
    pub(crate) references: Vec<NodeRef<'ast>>,
    pub(crate) captured_by_block: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct TableScope<'ast> {
    pub(crate) node: NodeRef<'ast>,
    pub(crate) variables: BTreeMap<String, TableVariable<'ast>>,
}

pub(crate) struct VariableTable<'ast> {
    scope_stack: Vec<TableScope<'ast>>,
    hook_events: Vec<String>,
}

impl<'ast> VariableTable<'ast> {
    pub(crate) fn initialize() -> Self {
        Self {
            scope_stack: Vec::new(),
            hook_events: Vec::new(),
        }
    }

    pub(crate) fn invoke_hook(&mut self, hook_name: &str) {
        self.hook_events.push(hook_name.to_owned());
    }

    pub(crate) fn scope_stack(&self) -> &[TableScope<'ast>] {
        &self.scope_stack
    }

    pub(crate) fn push_scope(&mut self, scope_node: NodeRef<'ast>) -> &TableScope<'ast> {
        self.invoke_hook("before_entering_scope");
        self.scope_stack.push(TableScope {
            node: scope_node,
            variables: BTreeMap::new(),
        });
        self.invoke_hook("after_entering_scope");
        self.scope_stack.last().unwrap()
    }

    pub(crate) fn pop_scope(&mut self) -> Option<TableScope<'ast>> {
        self.current_scope()?;
        self.invoke_hook("before_leaving_scope");
        let scope = self.scope_stack.pop();
        self.invoke_hook("after_leaving_scope");
        scope
    }

    pub(crate) fn current_scope(&self) -> Option<&TableScope<'ast>> {
        self.scope_stack.last()
    }

    pub(crate) fn current_scope_level(&self) -> usize {
        self.scope_stack.len()
    }

    pub(crate) fn declare_variable(
        &mut self,
        name: &str,
        node: NodeRef<'ast>,
    ) -> Option<&TableVariable<'ast>> {
        let level = self.current_scope_level();
        self.invoke_hook("before_declaring_variable");
        let variable = TableVariable {
            name: name.to_owned(),
            declaration_node: node,
            scope_level: level,
            assignments: Vec::new(),
            references: Vec::new(),
            captured_by_block: false,
        };
        self.scope_stack
            .last_mut()?
            .variables
            .insert(name.to_owned(), variable);
        self.invoke_hook("after_declaring_variable");
        self.scope_stack.last()?.variables.get(name)
    }

    pub(crate) fn assign_to_variable(
        &mut self,
        name: &str,
        node: NodeRef<'ast>,
    ) -> Result<(), String> {
        let Some((scope, _)) = self.find_variable_location(name) else {
            return Err(format!("Assigning to undeclared local variable \"{name}\""));
        };
        self.mark_variable_as_captured_by_block_if_so(scope, name);
        self.scope_stack[scope]
            .variables
            .get_mut(name)
            .unwrap()
            .assignments
            .push(node);
        Ok(())
    }

    pub(crate) fn reference_variable(&mut self, name: &str, node: NodeRef<'ast>) -> bool {
        let Some((scope, _)) = self.find_variable_location(name) else {
            return false;
        };
        self.mark_variable_as_captured_by_block_if_so(scope, name);
        self.scope_stack[scope]
            .variables
            .get_mut(name)
            .unwrap()
            .references
            .push(node);
        true
    }

    pub(crate) fn find_variable(&self, name: &str) -> Option<&TableVariable<'ast>> {
        self.find_variable_location(name)
            .map(|(_, variable)| variable)
    }

    pub(crate) fn variable_exist(&self, name: &str) -> bool {
        self.find_variable(name).is_some()
    }

    pub(crate) fn accessible_variables(&self) -> Vec<&TableVariable<'ast>> {
        let mut variables = Vec::new();
        for scope in self.scope_stack.iter().rev() {
            variables.extend(scope.variables.values());
            if !any_block_type(scope.node) {
                break;
            }
        }
        variables
    }

    pub(crate) fn mark_variable_as_captured_by_block_if_so(
        &mut self,
        variable_scope: usize,
        name: &str,
    ) {
        let Some(current) = self.scope_stack.last() else {
            return;
        };
        if any_block_type(current.node) && variable_scope + 1 != self.current_scope_level() {
            if let Some(variable) = self.scope_stack[variable_scope].variables.get_mut(name) {
                variable.captured_by_block = true;
            }
        }
    }

    fn find_variable_location(&self, name: &str) -> Option<(usize, &TableVariable<'ast>)> {
        for (index, scope) in self.scope_stack.iter().enumerate().rev() {
            if let Some(variable) = scope.variables.get(name) {
                return Some((index, variable));
            }
            if !any_block_type(scope.node) {
                return None;
            }
        }
        None
    }
}

fn any_block_type(node: NodeRef<'_>) -> bool {
    matches!(node.kind(), "block" | "numblock" | "itblock")
}

#[cfg(test)]
mod spec;
