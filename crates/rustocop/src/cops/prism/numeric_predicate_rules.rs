use super::*;

define_cops! {
    NumericPredicate => "Style/NumericPredicate" => call(numeric_predicate),
}

fn numeric_predicate(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if context.policy().allows_method(call_name(node))
        || context.ancestors().iter().rev().any(|ancestor| {
            ancestor
                .as_call_node()
                .is_some_and(|call| context.policy().allows_method(call_name(&call)))
        })
    {
        return;
    }
    if context.policy().enforced_style("predicate") == "comparison" {
        comparison_style(node, context);
    } else {
        predicate_style(node, context);
    }
}

fn predicate_style(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let operator = call_name(node);
    if !matches!(operator, b"==" | b">" | b"<") {
        return;
    }
    let (Some(left), Some(right)) = (node.receiver(), only_argument(node)) else {
        return;
    };
    let (value, reversed) = if value(&right) == Some(0) {
        (left, false)
    } else if value(&left) == Some(0) {
        (right, true)
    } else {
        return;
    };
    if value.as_global_variable_read_node().is_some() {
        return;
    }
    let predicate = match (operator, reversed) {
        (b"==", _) => "zero?",
        (b">", false) | (b"<", true) => "positive?",
        (b"<", false) | (b">", true) => "negative?",
        _ => return,
    };
    if predicate != "zero?" && !context.target_ruby_version().at_least(2, 3) {
        return;
    }
    let source = context.source_file().node(&value);
    let receiver = if simple_receiver(&value) {
        source.to_string()
    } else {
        format!("({source})")
    };
    let preferred = format!("{receiver}.{predicate}");
    let original = context.source_file().node(&node.as_node());
    context.replace_call(
        node,
        format!("Use `{preferred}` instead of `{original}`."),
        preferred,
    );
}

fn comparison_style(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let operator = match call_name(node) {
        b"zero?" => "==",
        b"positive?" => ">",
        b"negative?" => "<",
        _ => return,
    };
    if argument_count(node) != 0 {
        return;
    }
    let Some(receiver) = node.receiver() else {
        return;
    };
    let mut preferred = format!("{} {operator} 0", context.source_file().node(&receiver));
    let negated = context
        .parent()
        .and_then(Node::as_call_node)
        .is_some_and(|parent| {
            call_name(&parent) == b"!"
                && parent.receiver().is_some_and(|child| {
                    child.location().start_offset() == node.location().start_offset()
                        && child.location().end_offset() == node.location().end_offset()
                })
        });
    if negated {
        preferred = format!("({preferred})");
    }
    let original = context.source_file().node(&node.as_node());
    context.replace_call(
        node,
        format!("Use `{preferred}` instead of `{original}`."),
        preferred,
    );
}

fn value(node: &Node<'_>) -> Option<i32> {
    TryInto::<i32>::try_into(node.as_integer_node()?.value()).ok()
}

fn simple_receiver(node: &Node<'_>) -> bool {
    node.as_local_variable_read_node().is_some()
        || node.as_instance_variable_read_node().is_some()
        || node.as_class_variable_read_node().is_some()
        || node.as_constant_read_node().is_some()
        || node.as_constant_path_node().is_some()
        || node.as_call_node().is_some_and(|call| {
            call.receiver().is_none() && argument_count(&call) == 0 && call.block().is_none()
        })
}
