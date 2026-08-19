use super::*;

pub(super) struct Runner<'registry, 'context> {
    pub(super) registry: &'registry Registry,
    pub(super) context: &'context mut Context,
    pub(super) source: &'context str,
    pub(super) ancestors: Vec<Node<'context>>,
}

macro_rules! visit_typed_branch {
    ($runner:expr, $node:expr, $cast:ident, $walk:path) => {{
        let generic = $node.as_node();
        let already_entered = $runner.ancestors.last().is_some_and(|ancestor| {
            ancestor.$cast().is_some_and(|ancestor| {
                ancestor.location().start_offset() == $node.location().start_offset()
                    && ancestor.location().end_offset() == $node.location().end_offset()
            })
        });
        if already_entered {
            $walk($runner, $node);
        } else {
            $runner.visit_branch_node_enter(generic);
            $walk($runner, $node);
            $runner.visit_branch_node_leave();
        }
    }};
}

impl<'pr> Visit<'pr> for Runner<'_, 'pr> {
    fn visit_branch_node_enter(&mut self, node: Node<'pr>) {
        for cop in self
            .registry
            .node_cops
            .iter()
            .map(|index| &self.registry.cops[*index])
        {
            cop.on_node(&node, &self.ancestors, self.source, self.context);
        }
        self.ancestors.push(node);
    }

    fn visit_branch_node_leave(&mut self) {
        self.ancestors.pop();
    }

    fn visit_leaf_node_enter(&mut self, node: Node<'pr>) {
        for cop in self
            .registry
            .node_cops
            .iter()
            .map(|index| &self.registry.cops[*index])
        {
            cop.on_node(&node, &self.ancestors, self.source, self.context);
        }
    }

    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode<'pr>) {
        visit_typed_branch!(self, node, as_block_node, ruby_prism::visit_block_node);
    }

    fn visit_else_node(&mut self, node: &ruby_prism::ElseNode<'pr>) {
        visit_typed_branch!(self, node, as_else_node, ruby_prism::visit_else_node);
    }

    fn visit_rescue_node(&mut self, node: &ruby_prism::RescueNode<'pr>) {
        visit_typed_branch!(self, node, as_rescue_node, ruby_prism::visit_rescue_node);
    }
}
