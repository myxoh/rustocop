use super::*;

define_cops! {
    NumericPredicate => "Style/NumericPredicate" => call(numeric_predicate),
}

fn numeric_predicate(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if allowed_method_name(context, call_name(node))
        || context.ancestors().iter().rev().any(|ancestor| {
            ancestor
                .as_call_node()
                .is_some_and(|call| allowed_method_name(context, call_name(&call)))
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

fn allowed_method_name(context: &CopContext<'_, '_>, name: &[u8]) -> bool {
    use crate::rubocop::cop::mixin::allowed_methods::AllowedMethods;
    use crate::rubocop::cop::mixin::allowed_pattern::{AllowedPattern, PatternValue};

    let Ok(name) = std::str::from_utf8(name) else {
        return false;
    };
    let methods = AllowedMethods::new(
        context.config_values("AllowedMethods").to_vec(),
        Vec::new(),
        Vec::new(),
    );
    let patterns = AllowedPattern::new(
        context
            .config_values("AllowedPatterns")
            .iter()
            .cloned()
            .map(PatternValue::Source)
            .collect(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    methods.allowed_method(name) || patterns.matches_allowed_pattern(name)
}

fn predicate_style(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let operator = call_name(node);
    if !matches!(operator, b"==" | b">" | b"<") {
        return;
    }
    if node
        .call_operator_loc()
        .is_some_and(|operator| operator.as_slice() == b"&.")
    {
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
    if node.as_parentheses_node().is_some() || node.as_self_node().is_some() {
        return true;
    }
    node.as_call_node().is_none_or(|call| {
        call_name(&call) != b"[]"
            && !binary_operation(call_name(&call))
            && (argument_count(&call) == 0
                || call.call_operator_loc().is_some()
                || call.opening_loc().is_some()
                || call.block().is_some())
    })
}

fn binary_operation(name: &[u8]) -> bool {
    matches!(
        name,
        b"|" | b"^"
            | b"&"
            | b"<=>"
            | b"=="
            | b"==="
            | b"=~"
            | b">"
            | b">="
            | b"<"
            | b"<="
            | b"<<"
            | b">>"
            | b"+"
            | b"-"
            | b"*"
            | b"/"
            | b"%"
            | b"**"
    )
}
