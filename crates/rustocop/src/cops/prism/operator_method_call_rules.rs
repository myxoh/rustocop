use ruby_prism::{CallNode, Node};

use super::*;

define_rule!(OperatorMethodCallRule);

define_cops! {
    SpaceAroundMethodCallOperator => "Layout/SpaceAroundMethodCallOperator" => any_node(space_around_method_call_operator),
    OperatorMethodCall => "Style/OperatorMethodCall" => call_rule(
        OperatorMethodCallRule,
        on_send,
        restrict [b"|", b"^", b"&", b"<=>", b"==", b"===", b"=~", b">", b">=", b"<", b"<=", b"<<", b">>", b"+", b"-", b"*", b"/", b"%", b"**", b"~", b"!", b"!=", b"!~"]
    ),
}

fn space_around_method_call_operator(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let Some(call) = node.as_call_node() else {
        return;
    };
    let (Some(receiver), Some(operator), Some(selector)) =
        (call.receiver(), call.call_operator_loc(), call.message_loc())
    else {
        return;
    };
    for range in [
        receiver.location().end_offset()..operator.start_offset(),
        operator.end_offset()..selector.start_offset(),
    ] {
        if range.start < range.end
            && context.source()[range.clone()]
                .bytes()
                .all(|byte| matches!(byte, b' ' | b'\t'))
        {
            context.remove(
                "Avoid using spaces around a method call operator.",
                range.clone(),
                range,
            );
        }
    }
}

impl OperatorMethodCallRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        let Some(dot) = node.call_operator_loc().filter(|dot| dot.as_slice() == b".") else { return };
        let Some(receiver) = node.receiver() else { return };
        return_if!(constant_receiver(&receiver));
        let arguments = node.arguments().map(|arguments| arguments.arguments().iter().collect::<Vec<_>>()).unwrap_or_default();
        return_unless!(arguments.len() == 1);
        let rhs = &arguments[0];
        return_if!(invalid_operator_argument(rhs));
        return_if!(node.opening_loc().is_some() && method_call_with_parenthesized_arg(rhs));
        let Some(selector) = node.message_loc() else { return };
        return_if!(matches!(node.name().as_slice(), b"~" | b"!") && selector.as_slice() != node.name().as_slice());

        let chained = self.ancestors().iter().rev().find_map(Node::as_call_node).is_some_and(|parent| {
            parent.receiver().is_some_and(|parent_receiver| same_location(&parent_receiver, &node.as_node()))
        });
        let receiver_source = self.source_file().node(&receiver);
        let rhs_source = self.source_file().node(rhs);
        let operator = String::from_utf8_lossy(node.name().as_slice());
        if chained {
            return_if!(node.opening_loc().is_some() && method_call_with_parenthesized_arg(rhs));
            let replacement = format!("({receiver_source} {operator} {rhs_source})");
            add_offense!(self, dot, message: "Redundant dot detected.", |corrector| {
                corrector.replace(node.location(), replacement);
            });
            return;
        }

        let insert_space = selector.end_offset() == rhs.location().start_offset()
            || node.name().as_slice() == b"/"
                && self.source()[selector.end_offset()..rhs.location().start_offset()].contains('(');
        let dot = dot.start_offset()..dot.end_offset();
        add_offense!(self, dot.clone(), message: "Redundant dot detected.", |corrector| {
            corrector.replace(dot, " ");
            if insert_space {
                corrector.replace(selector.end_offset()..selector.end_offset(), " ");
            }
        });
    }
}

fn constant_receiver(node: &Node<'_>) -> bool {
    node.as_constant_read_node().is_some() || node.as_constant_path_node().is_some()
}

fn invalid_operator_argument(node: &Node<'_>) -> bool {
    node.as_splat_node().is_some() || node.as_assoc_splat_node().is_some()
        || node.as_forwarding_arguments_node().is_some() || node.as_block_argument_node().is_some()
        || node.as_keyword_hash_node().is_some_and(|hash| hash.elements().iter().next().is_some_and(|element| element.as_assoc_splat_node().is_some()))
}

fn method_call_with_parenthesized_arg(node: &Node<'_>) -> bool {
    // Parser's `argument.children.first` is the receiver for sends and the
    // stored value/name for literals and variable reads. A bare send has a
    // nil receiver even when it has arguments, so it must remain eligible.
    node.as_call_node().is_some_and(|call| call.receiver().is_some())
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_array_node().is_some_and(|array| array.elements().iter().next().is_some())
        || node.as_hash_node().is_some_and(|hash| hash.elements().iter().next().is_some())
        || node.as_local_variable_read_node().is_some()
        || node.as_instance_variable_read_node().is_some()
        || node.as_class_variable_read_node().is_some()
        || node.as_global_variable_read_node().is_some()
}
