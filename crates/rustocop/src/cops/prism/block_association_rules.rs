use super::*;

define_cops! {
    AmbiguousBlockAssociation => "Lint/AmbiguousBlockAssociation" => compatibility_prism_call(ambiguous_block_association),
}

fn ambiguous_block_association(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.opening_loc().is_some() || argument_count(node) == 0 || operator_call(call_name(node)) {
        return;
    }
    if call_name(node).ends_with(b"=") {
        return;
    }
    let Some(argument) = node.arguments().and_then(|arguments| arguments.arguments().last()) else {
        return;
    };
    let Some(block_call) = argument.as_call_node() else {
        return;
    };
    let Some(block) = block_call.block().and_then(|block| block.as_block_node()) else {
        return;
    };
    if block_call.opening_loc().is_some()
        || block.opening_loc().as_slice() != b"{"
        || matches!(block_call.name().as_slice(), b"lambda" | b"proc")
        || context
            .source_file()
            .node(&argument)
            .trim_start()
            .starts_with("Proc.new")
    {
        return;
    }

    let parameter = context.source_file().node(&argument);
    let method = context
        .source()
        .get(block_call.location().start_offset()..block.opening_loc().start_offset())
        .unwrap_or_default()
        .trim_end();
    if context.policy().allows_method(call_name(&block_call))
        || context.policy().allows_method(method.as_bytes())
    {
        return;
    }
    let message = format!(
        "Parenthesize the param `{parameter}` to make sure that the block will be associated with the `{method}` method call."
    );
    let (Some(selector), Some(first_argument)) = (
        node.message_loc(),
        node.arguments().and_then(|arguments| arguments.arguments().first()),
    ) else {
        return;
    };
    context.replace_many(
        message,
        node.location(),
        vec![
            (
                selector.end_offset()..first_argument.location().start_offset(),
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
