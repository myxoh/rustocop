use ruby_prism::{BlockNode, CallNode, Node, Visit};

use super::*;
use super::source_syntax::split_arguments;

define_cops! {
    HashEachMethods => "Style/HashEachMethods" => compatibility_prism_callbacks(HashEachMethodsRule, [on_block, on_send restrict [b"each"]]),
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
        return_unless!(node.block().is_some_and(|block| {
            block
                .as_block_argument_node()
                .and_then(|argument| argument.expression())
                .is_some_and(|expression| expression.as_symbol_node().is_some())
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
            let facts = hash_each_body_facts(
                block.body(),
                self.source_file(),
                root_receiver_source(&root, self.source_file()),
            );
            return_if!(facts.mutated, false);
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
        let facts = hash_each_body_facts(
            block.body(),
            self.source_file(),
            root_receiver_source(&receiver, self.source_file()),
        );
        return_if!(facts.mutated);
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
        let key_used = parameter_used(key, &facts.reads);
        let value_used = parameter_used(value, &facts.reads);
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

fn root_receiver_source<'a>(node: &Node<'_>, file: SourceFile<'a>) -> &'a str {
    node.as_call_node()
        .and_then(|call| call.receiver())
        .map_or_else(|| file.node(node), |receiver| root_receiver_source(&receiver, file))
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

#[derive(Default)]
struct HashEachBodyFacts {
    reads: std::collections::HashSet<Vec<u8>>,
    mutated: bool,
}

struct HashEachBodyVisitor<'a> {
    file: SourceFile<'a>,
    receiver: &'a str,
    facts: HashEachBodyFacts,
}

impl<'pr> Visit<'pr> for HashEachBodyVisitor<'_> {
    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode<'pr>) {
        self.facts.reads.insert(node.name().as_slice().to_vec());
    }

    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        if node.name().as_slice() == b"[]="
            && node
                .receiver()
                .is_some_and(|receiver| self.file.node(&receiver) == self.receiver)
        {
            self.facts.mutated = true;
        }
        ruby_prism::visit_call_node(self, node);
    }
}

fn hash_each_body_facts(
    body: Option<Node<'_>>,
    file: SourceFile<'_>,
    receiver: &str,
) -> HashEachBodyFacts {
    let mut visitor = HashEachBodyVisitor {
        file,
        receiver,
        facts: HashEachBodyFacts::default(),
    };
    if let Some(body) = body {
        visitor.visit(&body);
    }
    visitor.facts
}

fn parameter_used(
    parameter: &str,
    reads: &std::collections::HashSet<Vec<u8>>,
) -> bool {
    parameter
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .filter(|name| !name.is_empty() && *name != "_")
        .any(|name| reads.contains(name.as_bytes()))
}
