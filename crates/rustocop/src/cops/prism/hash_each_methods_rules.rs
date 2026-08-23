use ruby_prism::{BlockNode, CallNode, Node};

use super::*;
use super::source_syntax::split_arguments;

define_cops! {
    HashEachMethods => "Style/HashEachMethods" => rubocop_callbacks(HashEachMethodsRule, [on_block, on_send restrict [b"each"]]),
}

impl HashEachMethodsRule<'_, '_, '_> {
    fn on_block(&mut self, block: &BlockNode<'_>) {
        let Some(each) = self.parent().and_then(Node::as_call_node) else { return };
        return_unless!(each.name().as_slice() == b"each");
        if self.register_keys_values_each(&each, Some(block)) {
            return;
        }
        self.check_unused_arguments(&each, block);
    }

    fn on_send(&mut self, node: &CallNode<'_>) {
        return_if!(node.block().is_some_and(|block| block.as_block_node().is_some()));
        return_if!(node.block().is_some_and(|block| {
            block
                .as_block_argument_node()
                .and_then(|argument| argument.expression())
                .is_none_or(|expression| expression.as_symbol_node().is_none())
        }));
        self.register_keys_values_each(node, None);
    }

    fn register_keys_values_each(
        &mut self,
        each: &CallNode<'_>,
        block: Option<&BlockNode<'_>>,
    ) -> bool {
        let Some(keys_values) = each.receiver().and_then(|receiver| receiver.as_call_node()) else { return false };
        let method = keys_values.name().as_slice();
        return_if!(!matches!(method, b"keys" | b"values"), false);
        return_if!(argument_count(&keys_values) != 0, false);
        let Some(root) = keys_values.receiver() else { return false };
        return_if!(array_converter(&root), false);
        let root_source = self.source_file().node(&root);
        return_if!(allowed_receiver(self, &root, root_source), false);
        if let Some(block) = block {
            let body_source = block
                .body()
                .map(|body| self.source_file().node(&body))
                .unwrap_or_default();
            return_if!(hash_mutated(body_source, root_source), false);
        }
        let Some(first_selector) = keys_values.message_loc() else { return false };
        let Some(last_selector) = each.message_loc() else { return false };
        let current = self
            .source_file()
            .slice(first_selector.start_offset()..last_selector.end_offset())
            .unwrap_or_default();
        let preferred = if method == b"keys" { "each_key" } else { "each_value" };
        let edit = first_selector.start_offset()..last_selector.end_offset();
        add_offense!(self, edit.clone(), message: format!("Use `{preferred}` instead of `{current}`."), |corrector| {
            corrector.replace(edit, preferred);
        });
        true
    }

    fn check_unused_arguments(&mut self, each: &CallNode<'_>, block: &BlockNode<'_>) {
        let Some(receiver) = each.receiver() else { return };
        return_if!(argument_count(each) != 0);
        return_if!(receiver.as_array_node().is_some() || array_converter(&receiver));
        let receiver_source = self.source_file().node(&receiver);
        return_if!(hash_mutated(self.source_file().node(&block.as_node()), receiver_source));
        let Some(parameters) = block.parameters().and_then(|parameters| parameters.as_block_parameters_node()) else { return };
        let source = self
            .source_file()
            .slice(parameters.location().start_offset()..parameters.location().end_offset())
            .unwrap_or_default();
        let inner_start = source.find('|').map_or(0, |offset| offset + 1);
        let inner_end = source.rfind('|').unwrap_or(source.len());
        let absolute_start = parameters.location().start_offset() + inner_start;
        let arguments = split_arguments(self.source(), absolute_start, parameters.location().start_offset() + inner_end);
        let [key_range, value_range] = arguments.as_slice() else { return };
        let key = self.source_file().slice(key_range.clone()).unwrap_or_default().trim();
        let value = self.source_file().slice(value_range.clone()).unwrap_or_default().trim();
        let body = self
            .source_file()
            .slice(parameters.location().end_offset()..block.closing_loc().start_offset())
            .unwrap_or_default();
        let key_used = parameter_used(key, body);
        let value_used = parameter_used(value, body);
        return_if!(key_used == value_used);
        let (preferred, used, unused) = if key_used {
            ("each_key", key, value)
        } else {
            ("each_value", value, key)
        };
        let current = String::from_utf8_lossy(each.name().as_slice());
        let message = format!(
            "Use `{preferred}` instead of `{current}` and remove the unused `{unused}` block argument."
        );
        let Some(selector) = each.message_loc() else { return };
        let offense = each.location().start_offset()..block.closing_loc().end_offset();
        let parameter_range = parameters.location().start_offset()..parameters.location().end_offset();
        add_offense!(self, offense, message: message, |corrector| {
            corrector.replace(selector, preferred);
            corrector.replace(parameter_range, format!("|{used}|"));
        });
    }
}

fn array_converter(node: &Node<'_>) -> bool {
    node.as_call_node().is_some_and(|call| {
        matches!(
            call.name().as_slice(),
            b"assoc" | b"chunk" | b"flatten" | b"rassoc" | b"sort" | b"sort_by" | b"to_a"
        )
    })
}

fn allowed_receiver(
    context: &CopContext<'_, '_>,
    receiver: &Node<'_>,
    source: &str,
) -> bool {
    context.config_values("AllowedReceivers").iter().any(|allowed| {
        source == allowed
            || receiver.as_call_node().is_some_and(|call| {
                String::from_utf8_lossy(call.name().as_slice()) == allowed.as_str()
            })
    })
}

fn hash_mutated(body: &str, receiver: &str) -> bool {
    body.contains(&format!("{receiver}[")) && body.contains("] =")
}

fn parameter_used(parameter: &str, body: &str) -> bool {
    parameter
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .filter(|name| !name.is_empty() && *name != "_")
        .any(|name| contains_word(body, name))
}

fn contains_word(source: &str, name: &str) -> bool {
    source
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .any(|word| word == name)
}
