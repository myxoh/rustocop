use ruby_prism::CallNode;

use super::*;

define_rule!(RedundantFilterChainRule);

const MSG: &str = "Use `{prefer}` instead of `{first_method}.{second_method}`.";

define_cops! {
    RedundantFilterChain => "Style/RedundantFilterChain" => call_rule(
        RedundantFilterChainRule,
        on_send,
        restrict [b"any?", b"empty?", b"none?", b"one?", b"many?", b"present?"]
    ),
}

impl RedundantFilterChainRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        return_if!(argument_count(node) > 0 || node.block().is_some());
        let Some(select_node) = node.receiver().and_then(|receiver| receiver.as_call_node()) else {
            return;
        };
        return_unless!(matches!(
            select_node.name().as_slice(),
            b"select" | b"filter" | b"find_all"
        ));
        return_unless!(select_node.block().is_some());

        let second_method = node.name().as_slice();
        return_if!(
            matches!(second_method, b"many?" | b"present?")
                && self.related_config_value("AllCops", "ActiveSupportExtensionsEnabled")
                    != Some("true")
        );

        let prefer = match second_method {
            b"empty?" | b"none?" => "none?",
            b"present?" => "any?",
            b"any?" => "any?",
            b"one?" => "one?",
            b"many?" => "many?",
            _ => unreachable!("restricted callback"),
        };
        self.register_offense(&select_node, node, prefer);
    }

    fn register_offense(
        &mut self,
        select_node: &CallNode<'_>,
        predicate_node: &CallNode<'_>,
        replacement: &str,
    ) {
        let (Some(select_selector), Some(predicate_selector)) =
            (select_node.message_loc(), predicate_node.message_loc())
        else {
            return;
        };
        let first_method = String::from_utf8_lossy(select_node.name().as_slice());
        let second_method = String::from_utf8_lossy(predicate_node.name().as_slice());
        let message = MSG
            .replace("{prefer}", replacement)
            .replace("{first_method}", &first_method)
            .replace("{second_method}", &second_method);
        let offense = select_selector.start_offset()..predicate_selector.end_offset();
        let predicate_range = select_node.location().end_offset()..predicate_selector.end_offset();
        add_offense!(self, offense, message: message, |corrector| {
            corrector.remove(predicate_range);
            corrector.replace(select_selector, replacement);
        });
    }
}
