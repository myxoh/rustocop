use super::*;

declare_cops!(BeginBlock);

struct BeginBlock;

impl Cop for BeginBlock {
    fn name(&self) -> &'static str {
        "Style/BeginBlock"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        _source: &str,
        context: &mut Context,
    ) {
        let Some(pre_execution) = node.as_pre_execution_node() else {
            return;
        };
        context.report(
            self.name(),
            "Avoid the use of `BEGIN` blocks.",
            pre_execution.keyword_loc(),
        );
    }
}
