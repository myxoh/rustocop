use super::*;

define_cops! {
    MultilineBlockChain => "Style/MultilineBlockChain" => call(multiline_block_chain),
}

fn multiline_block_chain(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node
        .block()
        .and_then(|block| block.as_block_node())
        .is_none()
    {
        return;
    }
    let mut receiver = node.receiver();
    let mut prior_block = None;
    while let Some(call) = receiver.and_then(|receiver| receiver.as_call_node()) {
        if let Some(block) = call.block().and_then(|block| block.as_block_node()) {
            prior_block = Some(block);
            break;
        }
        receiver = call.receiver();
    }
    let (Some(block), Some(selector)) = (prior_block, node.message_loc()) else {
        return;
    };
    let opening = block.opening_loc();
    let closing = block.closing_loc();
    if context
        .source_file()
        .same_line(opening.start_offset(), closing.start_offset())
    {
        return;
    }
    context.report(
        "Avoid multi-line chains of blocks.",
        closing.start_offset()..selector.end_offset(),
    );
}
