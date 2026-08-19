use ruby_prism::{CallNode, Node};

use super::CopContext;

/// RuboCop-shaped source and range vocabulary for typed Prism callbacks.
pub(super) trait CopContextNodeExt<'pr> {
    fn source_of(&self, node: &Node<'_>) -> &'pr str;
    fn multiline(&self, node: &Node<'_>) -> bool;
    fn selector_through(&self, node: &CallNode<'_>, end: usize) -> Option<std::ops::Range<usize>>;
}

impl<'context, 'pr> CopContextNodeExt<'pr> for CopContext<'context, 'pr> {
    fn source_of(&self, node: &Node<'_>) -> &'pr str {
        self.source_file().node(node)
    }

    fn multiline(&self, node: &Node<'_>) -> bool {
        self.source_of(node).contains('\n')
    }

    fn selector_through(&self, node: &CallNode<'_>, end: usize) -> Option<std::ops::Range<usize>> {
        Some(node.message_loc()?.start_offset()..end)
    }
}
