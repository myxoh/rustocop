use ruby_prism::{CallNode, Node, RangeNode};

use super::*;

define_rule!(NegativeArrayIndexRule);

define_cops! {
    NegativeArrayIndex => "Style/NegativeArrayIndex" => call_rule(
        NegativeArrayIndexRule,
        on_send,
        restrict [b"[]"]
    ),
}

impl NegativeArrayIndexRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        let Some(receiver) = node.receiver() else {
            return;
        };
        let Some(index) = node.first_argument() else {
            return;
        };

        if let Some((range, parenthesized)) = array_index_range(&index) {
            if self.handle_range(&receiver, &index, &range, parenthesized) {
                return;
            }
        }

        let Some((length_receiver, negative_index)) = length_subtraction(&index) else {
            return;
        };
        return_if!(negative_index <= 0);
        return_unless!(receivers_match(
            length_receiver.as_ref(),
            &receiver,
            self.source_file()
        ));

        let receiver_source = self.source_file().node(&receiver);
        let current = format!("{receiver_source}[{}]", self.source_file().node(&index));
        let message = format!("Use `{receiver_source}[-{negative_index}]` instead of `{current}`.");
        add_offense!(self, index.location(), message: message, |corrector| {
            corrector.replace(index.location(), format!("-{negative_index}"));
        });
    }

    fn handle_range(
        &mut self,
        receiver: &Node<'_>,
        index: &Node<'_>,
        range: &RangeNode<'_>,
        parenthesized: bool,
    ) -> bool {
        let (Some(start), Some(end)) = (range.left(), range.right()) else {
            return false;
        };
        return_if!(!preserving_receiver(&start), false);
        let end_location = end.location();
        let original_end_source = self.source_file().node(&end).to_string();
        let (inner_end, end_parenthesized) = unwrap_parentheses(end);
        let Some((length_receiver, negative_index)) = length_subtraction(&inner_end) else {
            return false;
        };
        return_if!(negative_index <= 0, false);
        return_unless!(
            receivers_match_strict(length_receiver.as_ref(), receiver, self.source_file()),
            false
        );

        let receiver_source = self.source_file().node(receiver);
        let start_source = self.source_file().node(&start);
        let operator = String::from_utf8_lossy(range.operator_loc().as_slice());
        let end_source = if end_parenthesized {
            &original_end_source
        } else {
            self.source_file().node(&inner_end)
        };
        let range_source = format!("{start_source}{operator}{end_source}");
        let current = if parenthesized {
            format!("{receiver_source}[({range_source})]")
        } else {
            format!("{receiver_source}[{range_source}]")
        };
        let display_start = if parenthesized {
            format!("({start_source}")
        } else {
            start_source.to_string()
        };
        let display_index = if parenthesized {
            format!("{negative_index})")
        } else {
            negative_index.to_string()
        };
        let message = format!(
            "Use `{receiver_source}[{display_start}{operator}-{display_index}]` instead of `{current}`."
        );
        let replacement = if parenthesized {
            format!("({start_source}{operator}-{negative_index})")
        } else {
            format!("{start_source}{operator}-{negative_index}")
        };
        add_offense!(self, end_location, message: message, |corrector| {
            corrector.replace(index.location(), replacement);
        });
        true
    }
}

def_node_matcher! {
    fn length_subtraction<'pr>(node: &Node<'pr>) -> Option<(Option<Node<'pr>>, i32)> {
        let subtraction = node.as_call_node()?;
        if subtraction.name().as_slice() != b"-" {
            return None;
        }
        let length = subtraction.receiver()?.as_call_node()?;
        if !matches!(length.name().as_slice(), b"length" | b"size" | b"count") {
            return None;
        }
        if length.arguments_present() {
            return None;
        }
        let index = only_argument(&subtraction)?.as_integer_node()?;
        let index = TryInto::<i32>::try_into(index.value()).ok()?;
        Some((length.receiver(), index))
    }
}

fn array_index_range<'pr>(node: &Node<'pr>) -> Option<(RangeNode<'pr>, bool)> {
    if let Some(range) = node.as_range_node() {
        return Some((range, false));
    }
    let parentheses = node.as_parentheses_node()?;
    let expression = parentheses.body().and_then(single_expression)?;
    Some((expression.as_range_node()?, true))
}

fn unwrap_parentheses(node: Node<'_>) -> (Node<'_>, bool) {
    let Some(parentheses) = node.as_parentheses_node() else {
        return (node, false);
    };
    let Some(expression) = parentheses.body().and_then(single_expression) else {
        return (node, false);
    };
    (expression, true)
}

fn preserving_receiver(node: &Node<'_>) -> bool {
    let Some(call) = node.as_call_node() else {
        return true;
    };
    let Some(receiver) = call.receiver() else {
        return true;
    };
    matches!(
        call.name().as_slice(),
        b"sort" | b"reverse" | b"shuffle" | b"rotate"
    ) && preserving_receiver(&receiver)
}

fn receivers_match(
    length_receiver: Option<&Node<'_>>,
    array_receiver: &Node<'_>,
    file: SourceFile<'_>,
) -> bool {
    let Some(length_receiver) = length_receiver else {
        return array_receiver.as_self_node().is_some();
    };
    if !preserving_receiver(array_receiver) || !preserving_receiver(length_receiver) {
        return false;
    }
    file.node(length_receiver) == file.node(array_receiver)
        || array_receiver
            .as_call_node()
            .is_some_and(|call| call.receiver().is_some())
}

fn receivers_match_strict(
    length_receiver: Option<&Node<'_>>,
    array_receiver: &Node<'_>,
    file: SourceFile<'_>,
) -> bool {
    let Some(length_receiver) = length_receiver else {
        return false;
    };
    preserving_receiver(array_receiver) && file.node(length_receiver) == file.node(array_receiver)
}
