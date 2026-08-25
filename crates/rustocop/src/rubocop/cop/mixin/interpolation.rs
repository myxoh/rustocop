// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/interpolation.rb
// Source SHA-256: 71e820e0b68830a67a449d291b417c1bd8d98f7748596c846b758af67ac53fe2

use crate::rubocop::ast::node::core::NodeRef;

pub(crate) trait Interpolation {
    fn on_interpolation(&mut self, begin_node: NodeRef<'_>);

    fn on_dstr(&mut self, node: NodeRef<'_>) {
        self.on_node_with_interpolations(node);
    }

    fn on_xstr(&mut self, node: NodeRef<'_>) {
        self.on_dstr(node);
    }

    fn on_dsym(&mut self, node: NodeRef<'_>) {
        self.on_dstr(node);
    }

    fn on_regexp(&mut self, node: NodeRef<'_>) {
        self.on_dstr(node);
    }

    fn on_node_with_interpolations(&mut self, node: NodeRef<'_>) {
        for begin_node in node
            .child_nodes()
            .into_iter()
            .filter(|child| child.kind() == "begin")
        {
            self.on_interpolation(begin_node);
        }
    }
}
