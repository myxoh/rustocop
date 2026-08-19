use super::*;

define_rule!(WhileUntilDoRule);

const MSG: &str = "Do not use `do` with multi-line `{keyword}`.";

define_cops!(
    WhileUntilDo => "Style/WhileUntilDo" => node_rule_aliases(WhileUntilDoRule, on_while => [as_while_node, as_until_node]),
);

impl WhileUntilDoRule<'_, '_, '_> {
    fn on_while(&mut self, node: &Node<'_>) {
        let parts = if let Some(loop_node) = node.as_while_node() {
            loop_node.do_keyword_loc().map(|keyword| {
                (
                    "while",
                    loop_node.predicate().location().end_offset(),
                    keyword,
                )
            })
        } else if let Some(loop_node) = node.as_until_node() {
            loop_node.do_keyword_loc().map(|keyword| {
                (
                    "until",
                    loop_node.predicate().location().end_offset(),
                    keyword,
                )
            })
        } else {
            None
        };
        let Some((keyword_name, predicate_end, do_keyword)) = parts else {
            return;
        };
        return_unless!(self.multiline(node));

        let removal = predicate_end..do_keyword.end_offset();
        let message = MSG.replace("{keyword}", keyword_name);
        add_offense!(self, do_keyword, message: message, |corrector| {
            corrector.remove(removal);
        });
    }
}
