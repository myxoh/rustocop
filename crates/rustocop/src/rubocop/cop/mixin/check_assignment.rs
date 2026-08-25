// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/check_assignment.rb
// Source SHA-256: 95fa3d54fe986015fde7733d49651d9c0aaa4a147e1401b8825e1b0619db2523

use crate::rubocop::ast::node::core::NodeRef;

pub(crate) fn extract_rhs(node: NodeRef<'_>) -> Option<NodeRef<'_>> {
    if node.call_type() {
        node.last_argument()
    } else if node.assignment() {
        node.rhs()
    } else {
        None
    }
}

pub(crate) trait CheckAssignment {
    fn check_assignment(&mut self, node: NodeRef<'_>, rhs: NodeRef<'_>);

    fn on_lvasgn(&mut self, node: NodeRef<'_>) {
        self.dispatch_assignment(node);
    }

    fn on_ivasgn(&mut self, node: NodeRef<'_>) {
        self.on_lvasgn(node);
    }

    fn on_cvasgn(&mut self, node: NodeRef<'_>) {
        self.on_lvasgn(node);
    }

    fn on_gvasgn(&mut self, node: NodeRef<'_>) {
        self.on_lvasgn(node);
    }

    fn on_casgn(&mut self, node: NodeRef<'_>) {
        self.on_lvasgn(node);
    }

    fn on_masgn(&mut self, node: NodeRef<'_>) {
        self.on_lvasgn(node);
    }

    fn on_op_asgn(&mut self, node: NodeRef<'_>) {
        self.on_lvasgn(node);
    }

    fn on_or_asgn(&mut self, node: NodeRef<'_>) {
        self.on_lvasgn(node);
    }

    fn on_and_asgn(&mut self, node: NodeRef<'_>) {
        self.on_lvasgn(node);
    }

    fn on_send(&mut self, node: NodeRef<'_>) {
        if let Some(rhs) = extract_rhs(node) {
            self.check_assignment(node, rhs);
        }
    }

    fn dispatch_assignment(&mut self, node: NodeRef<'_>) {
        self.check_assignment(
            node,
            extract_rhs(node).expect("assignment callbacks require an RHS"),
        );
    }
}
