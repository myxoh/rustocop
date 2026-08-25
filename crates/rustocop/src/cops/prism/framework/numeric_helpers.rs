use ruby_prism::Node;

use super::{call_name, first_argument};

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
            .next()
            .is_some_and(|statement| float_expression(Some(&statement)));
    }
    node.as_call_node().is_some_and(|call| {
        let name = call_name(&call);
        if matches!(name, b"+" | b"-" | b"*" | b"**" | b"/" | b"%") {
            return float_expression(call.receiver().as_ref())
                || first_argument(&call)
                    .as_ref()
                    .is_some_and(|argument| float_expression(Some(argument)));
        }
        if matches!(name, b"to_f" | b"Float" | b"fdiv") {
            return true;
        }
        call.receiver().as_ref().is_some_and(|receiver| {
            receiver.as_float_node().is_some()
                && (matches!(
                    name,
                    b"@-" | b"abs" | b"magnitude" | b"modulo" | b"next_float" | b"prev_float" | b"quo"
                ) || matches!(name, b"ceil" | b"floor" | b"round" | b"truncate")
                    && first_argument(&call).is_some_and(|argument| {
                        argument.as_integer_node().is_some_and(|integer| {
                            integer
                                .value()
                                .to_u32_digits()
                                .1
                                .iter()
                                .any(|digit| *digit != 0)
                        })
                    }))
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
