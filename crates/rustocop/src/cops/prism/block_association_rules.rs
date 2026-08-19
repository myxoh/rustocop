use super::*;

define_cops! {
    AmbiguousBlockAssociation => "Lint/AmbiguousBlockAssociation" => call(ambiguous_block_association),
}

fn ambiguous_block_association(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.opening_loc().is_some() || argument_count(node) != 1 || operator_call(call_name(node)) {
        return;
    }
    let Some(argument) = only_argument(node) else {
        return;
    };
    let Some(block_call) = argument.as_call_node() else {
        return;
    };
    let Some(block) = block_call.block().and_then(|block| block.as_block_node()) else {
        return;
    };
    if block_call.receiver().is_some()
        || block_call.opening_loc().is_some()
        || block.opening_loc().as_slice() != b"{"
        || context.policy().allows_method(call_name(&block_call))
    {
        return;
    }

    let parameter = context.source_file().node(&argument);
    let method = String::from_utf8_lossy(call_name(&block_call));
    let message = format!(
        "Parenthesize the param `{parameter}` to make sure that the block will be associated with the `{method}` method call."
    );
    let Some(selector) = node.message_loc() else {
        return;
    };
    context.replace_many(
        message,
        node.location(),
        vec![
            (
                selector.end_offset()..argument.location().start_offset(),
                "(".to_string(),
            ),
            (
                argument.location().end_offset()..argument.location().end_offset(),
                ")".to_string(),
            ),
        ],
    );
}

fn operator_call(name: &[u8]) -> bool {
    matches!(
        name,
        b"=="
            | b"!="
            | b"<"
            | b">"
            | b"<="
            | b">="
            | b"<=>"
            | b"+"
            | b"-"
            | b"*"
            | b"/"
            | b"%"
            | b"**"
            | b"&"
            | b"|"
            | b"^"
            | b"<<"
            | b">>"
            | b"=~"
            | b"!~"
    )
}
