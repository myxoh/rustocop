use ruby_prism::CallNode;

use super::*;

define_cops! {
    HashLookupMethod => "Style/HashLookupMethod" => call(hash_lookup_method),
}

fn hash_lookup_method(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let style = context.policy().enforced_style("brackets").to_string();
    if style == "brackets" {
        fetch_to_brackets(node, context);
    } else if style == "fetch" {
        brackets_to_fetch(node, context);
    }
}

fn fetch_to_brackets(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !bracket_convertible_fetch(node) {
        return;
    }
    if context.ancestors().iter().rev().any(|ancestor| {
        ancestor.as_call_node().is_some_and(|parent| {
            bracket_convertible_fetch(&parent)
                && parent.receiver().is_some_and(|receiver| {
                    receiver.location().start_offset() == node.location().start_offset()
                        && receiver.location().end_offset() == node.location().end_offset()
                })
        })
    }) {
        return;
    }
    fetch_chain_to_brackets(node, context);
}

fn bracket_convertible_fetch(node: &CallNode<'_>) -> bool {
    node.name().as_slice() == b"fetch"
        && node.receiver().is_some()
        && node.block().is_none()
        && argument_count(node) == 1
}

fn fetch_chain_to_brackets(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    fetch_to_brackets_one(node, context);
    if let Some(inner) = node.receiver().and_then(|receiver| receiver.as_call_node()) {
        if bracket_convertible_fetch(&inner) {
            fetch_chain_to_brackets(&inner, context);
        }
    }
}

fn fetch_to_brackets_one(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let selector = node.message_loc().expect("fetch has a selector");
    let argument = only_argument(node).expect("checked above");
    let operator = node.call_operator_loc();
    if operator
        .as_ref()
        .is_some_and(|operator| operator.as_slice() == b"&.")
    {
        let receiver = node.receiver().expect("checked above");
        let location = node.location();
        context.replace(
            "Use `Hash#[]` instead of `Hash#fetch`.",
            &selector,
            &location,
            format!(
                "({}[{}])",
                context.source_file().node(&receiver),
                context.source_file().node(&argument)
            ),
        );
        return;
    }
    let (Some(operator), Some(opening), Some(closing)) =
        (operator, node.opening_loc(), node.closing_loc())
    else {
        return;
    };
    context.replace_many(
        "Use `Hash#[]` instead of `Hash#fetch`.",
        &selector,
        vec![
            (
                operator.start_offset()..opening.end_offset(),
                "[".to_string(),
            ),
            (
                closing.start_offset()..closing.end_offset(),
                "]".to_string(),
            ),
        ],
    );
}

fn brackets_to_fetch(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.name().as_slice() != b"[]" || node.receiver().is_none() || argument_count(node) != 1 {
        return;
    }
    let location = node.location();
    let source = context.source_file().at(&location);
    if source.contains("&.[]") {
        let selector = node.message_loc().expect("[] has a selector");
        context.replace(
            "Use `Hash#fetch` instead of `Hash#[]`.",
            &location,
            &selector,
            "fetch",
        );
    } else {
        let (Some(opening), Some(closing)) = (node.opening_loc(), node.closing_loc()) else {
            return;
        };
        context.replace_many(
            "Use `Hash#fetch` instead of `Hash#[]`.",
            &location,
            vec![
                (
                    opening.start_offset()..opening.end_offset(),
                    ".fetch(".to_string(),
                ),
                (
                    closing.start_offset()..closing.end_offset(),
                    ")".to_string(),
                ),
            ],
        );
    }
}
