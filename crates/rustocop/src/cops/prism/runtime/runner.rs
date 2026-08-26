use super::*;

pub(super) struct Runner<'registry, 'context> {
    pub(super) registry: &'registry Registry,
    pub(super) context: &'context mut Context,
    pub(super) source: &'context str,
    pub(super) ancestors: Vec<Node<'context>>,
    pub(super) investigation_states: &'context mut [Box<dyn Any>],
    pub(super) node_cops: &'registry [usize],
}

macro_rules! visit_typed_branch {
    ($runner:expr, $node:expr, $walk:path) => {{
        let generic = $node.as_node();
        let already_entered = $runner.current_node_matches(&generic);
        if already_entered {
            $walk($runner, $node);
        } else {
            $runner.visit_branch_node_enter(generic);
            $walk($runner, $node);
            $runner.visit_branch_node_leave();
        }
    }};
}

impl<'pr> Runner<'_, 'pr> {
    fn current_node_matches(&self, node: &Node<'pr>) -> bool {
        self.ancestors.last().is_some_and(|ancestor| {
            std::mem::discriminant(ancestor) == std::mem::discriminant(node)
                && ancestor.location().start_offset() == node.location().start_offset()
                && ancestor.location().end_offset() == node.location().end_offset()
        })
    }

    fn dispatch_node(&mut self, node: &Node<'pr>) {
        for index in self.node_cops {
            self.registry.cops[*index].on_node_with_state(
                node,
                &self.ancestors,
                self.source,
                self.context,
                self.investigation_states[*index].as_mut(),
            );
        }
    }
}

impl<'pr> Visit<'pr> for Runner<'_, 'pr> {
    fn visit_branch_node_enter(&mut self, node: Node<'pr>) {
        self.dispatch_node(&node);
        self.ancestors.push(node);
    }

    fn visit_branch_node_leave(&mut self) {
        self.ancestors.pop();
    }

    fn visit_leaf_node_enter(&mut self, node: Node<'pr>) {
        self.dispatch_node(&node);
    }

    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode<'pr>) {
        visit_typed_branch!(self, node, ruby_prism::visit_block_node);
    }

    fn visit_else_node(&mut self, node: &ruby_prism::ElseNode<'pr>) {
        visit_typed_branch!(self, node, ruby_prism::visit_else_node);
    }

    fn visit_rescue_node(&mut self, node: &ruby_prism::RescueNode<'pr>) {
        visit_typed_branch!(self, node, ruby_prism::visit_rescue_node);
    }

    fn visit_statements_node(&mut self, node: &ruby_prism::StatementsNode<'pr>) {
        let generic = node.as_node();
        if !self.current_node_matches(&generic) {
            self.dispatch_node(&generic);
        }
        ruby_prism::visit_statements_node(self, node);
    }
}
