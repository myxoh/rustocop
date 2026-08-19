use ruby_prism::Node;

use super::{argument_count, call_name, first_argument};

pub(super) fn literal_zero(node: Option<&Node<'_>>) -> bool {
    let Some(node) = node else {
        return false;
    };
    if let Some(integer) = node.as_integer_node() {
        return integer
            .value()
            .to_u32_digits()
            .1
            .iter()
            .all(|digit| *digit == 0);
    }
    node.as_float_node()
        .is_some_and(|float| float.value() == 0.0)
}

pub(super) fn float_expression(node: Option<&Node<'_>>) -> bool {
    let Some(node) = node else {
        return false;
    };
    if node.as_float_node().is_some() {
        return true;
    }
    if let Some(parentheses) = node.as_parentheses_node() {
        return parentheses
            .body()
            .as_ref()
            .is_some_and(|body| float_expression(Some(body)));
    }
    if let Some(statements) = node.as_statements_node() {
        return statements
            .body()
            .iter()
            .any(|statement| float_expression(Some(&statement)));
    }
    node.as_call_node().is_some_and(|call| {
        matches!(call_name(&call), b"to_f" | b"fdiv" | b"Float")
            || matches!(call_name(&call), b"+" | b"-" | b"*" | b"**" | b"/" | b"%")
                && (float_expression(call.receiver().as_ref())
                    || first_argument(&call)
                        .as_ref()
                        .is_some_and(|argument| float_expression(Some(argument))))
            || call.receiver().as_ref().is_some_and(|receiver| {
                float_expression(Some(receiver))
                    && !(matches!(
                        call_name(&call),
                        b"ceil" | b"floor" | b"round" | b"truncate" | b"to_i"
                    ) && argument_count(&call) == 0)
            })
    })
}

pub(super) fn immutable_literal(node: &Node<'_>) -> bool {
    node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_nil_node().is_some()
        || node.as_regular_expression_node().is_some()
        || node.as_range_node().is_some()
}
