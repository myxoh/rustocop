use ruby_prism::{CallNode, Node};

use super::*;

define_rule!(OperatorMethodCallRule);

define_cops! {
    SpaceAroundMethodCallOperator => "Layout/SpaceAroundMethodCallOperator" => source(space_around_method_call_operator),
    OperatorMethodCall => "Style/OperatorMethodCall" => call_rule(
        OperatorMethodCallRule,
        on_send,
        restrict [b"|", b"^", b"&", b"<=>", b"==", b"===", b"=~", b">", b">=", b"<", b"<=", b"<<", b">>", b"+", b"-", b"*", b"/", b"%", b"**", b"~", b"!", b"!=", b"!~"]
    ),
}

fn space_around_method_call_operator(context: &mut CopContext<'_, '_>) {
    let file = context.source_file();
    let source = context.source();
    let literal_ranges = file.literal_ranges();
    let comment_ranges = file.comment_ranges();
    let data_section_start = file.data_section_start();
    let mut operators = file.code_offsets("::");
    operators.extend(file.code_offsets("&."));
    operators.extend(file.code_offsets(".").into_iter().filter(|offset| {
        source.as_bytes().get(offset.wrapping_sub(1)) != Some(&b'&')
    }));
    operators.sort_unstable();
    operators.dedup();
    for start in operators {
        if data_section_start.is_some_and(|data| data <= start)
            || comment_ranges
                .iter()
                .any(|range| range.start <= start && start < range.end)
            || literal_ranges
            .iter()
            .any(|range| range.start <= start && start < range.end)
        {
            continue;
        }
        if source[start..].starts_with('.')
            && (source.as_bytes().get(start.wrapping_sub(1)) == Some(&b'.')
                || source.as_bytes().get(start + 1) == Some(&b'.'))
        {
            continue;
        }
        let width = if source[start..].starts_with("::") || source[start..].starts_with("&.") {
            2
        } else {
            1
        };
        let end = start + width;
        let right_end = end
            + source[end..]
                .bytes()
                .take_while(|byte| matches!(byte, b' ' | b'\t'))
                .count();
        if right_end > end && source.as_bytes().get(right_end) != Some(&b'#') {
            context.remove(
                "Avoid using spaces around a method call operator.",
                end..right_end,
                end..right_end,
            );
        }
        let line_start = file.line_start(start);
        let left_start = source[line_start..start].trim_end_matches([' ', '\t']).len() + line_start;
        let receiver_prefix = source[line_start..left_start].trim_end();
        if !source[start..].starts_with("::")
            && left_start < start
            && receiver_prefix
                .as_bytes()
                .last()
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b')' | b']' | b'}'))
        {
            context.remove(
                "Avoid using spaces around a method call operator.",
                left_start..start,
                left_start..start,
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
        let super_receiver = receiver.as_super_node().is_some()
            || receiver.as_forwarding_super_node().is_some();
        return_if!(!super_receiver
            && node.opening_loc().is_some()
            && method_call_with_parenthesized_arg(rhs));
        let Some(selector) = node.message_loc() else { return };
        return_if!(matches!(node.name().as_slice(), b"~" | b"!") && selector.as_slice() != node.name().as_slice());

        let chained = self.ancestors().iter().rev().find_map(Node::as_call_node).is_some_and(|parent| {
            parent.receiver().is_some_and(|parent_receiver| same_location(&parent_receiver, &node.as_node()))
        });
        let receiver_source = self.source_file().node(&receiver);
        let rhs_source = self.source_file().node(rhs);
        let operator = String::from_utf8_lossy(node.name().as_slice());
        if chained {
            return_if!(!super_receiver
                && node.opening_loc().is_some()
                && method_call_with_parenthesized_arg(rhs));
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
