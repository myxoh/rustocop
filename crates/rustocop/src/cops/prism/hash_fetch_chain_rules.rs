use ruby_prism::{CallNode, Node};

use super::*;

define_cops! {
    HashFetchChain => "Style/HashFetchChain" => compatibility_prism_callbacks(HashFetchChainRule, [on_send restrict [b"fetch"]]),
}

impl HashFetchChainRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        return_if!(!self.target_ruby_version().at_least(2, 3));
        if self.parent().and_then(Node::as_call_node).is_some_and(|parent| {
            parent.name().as_slice() == b"fetch"
                && parent.receiver().is_some_and(|receiver| {
                    receiver.location().start_offset() == node.location().start_offset()
                        && receiver.location().end_offset() == node.location().end_offset()
                })
        }) {
            return;
        }
        let arguments = fetch_arguments(node);
        return_unless!(arguments.len() == 2 && arguments[1].as_nil_node().is_some());

        let mut current = node.as_node().as_call_node().expect("call round trip");
        let mut keys = Vec::new();
        let mut first_selector = None;
        loop {
            let arguments = fetch_arguments(&current);
            if arguments.len() != 2 || !diggable_default(&arguments[1], self.source_file()) {
                break;
            }
            keys.push(self.source_file().node(&arguments[0]).to_string());
            first_selector = current.message_loc();
            let Some(receiver) = current.receiver().and_then(|receiver| receiver.as_call_node()) else { break };
            return_if!(receiver.name().as_slice() != b"fetch" && keys.len() < 2);
            if receiver.name().as_slice() != b"fetch" {
                break;
            }
            current = receiver;
        }
        return_if!(keys.len() < 2);
        keys.reverse();
        let Some(selector) = first_selector else { return };
        let replacement = format!("dig({})", keys.join(", "));
        let message = format!("Use `{replacement}` instead.");
        let edit = selector.start_offset()..node.location().end_offset();
        add_offense!(self, edit.clone(), message: message, |corrector| {
            corrector.replace(edit, replacement);
        });
    }
}

fn fetch_arguments<'pr>(node: &CallNode<'pr>) -> Vec<Node<'pr>> {
    node.arguments()
        .map(|arguments| arguments.arguments().iter().collect())
        .unwrap_or_default()
}

fn diggable_default(node: &Node<'_>, file: SourceFile<'_>) -> bool {
    if node.as_nil_node().is_some()
        || node
            .as_hash_node()
            .is_some_and(|hash| hash.elements().is_empty())
    {
        return true;
    }
    node.as_call_node().is_some_and(|call| {
        call.name().as_slice() == b"new"
            && argument_count(&call) == 0
            && call.receiver().is_some_and(|receiver| {
                matches!(file.node(&receiver), "Hash" | "::Hash")
            })
    })
}
