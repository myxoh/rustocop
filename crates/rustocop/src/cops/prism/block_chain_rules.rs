use super::*;

define_cops! {
    MultilineBlockChain => "Style/MultilineBlockChain" => call(multiline_block_chain),
}

fn multiline_block_chain(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(current_block) = node.block().and_then(|block| block.as_block_node()) else {
        return;
    };
    let mut receiver = node.receiver();
    let mut prior_block = None;
    while let Some(call) = receiver.and_then(|receiver| receiver.as_call_node()) {
        if let Some(block) = call.block().and_then(|block| block.as_block_node()) {
            prior_block = Some(block);
            break;
        }
        receiver = call.receiver();
    }
    let (Some(block), Some(_)) = (prior_block, node.message_loc()) else {
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
    let chain_start = node.location().start_offset();
    let current_end = call_send_end(node, context.source());
    let offense_end = if context.source_file().same_line(
        current_block.opening_loc().start_offset(),
        current_block.closing_loc().start_offset(),
    ) {
        let mut outer_blocks = context
            .ancestors()
            .iter()
            .filter_map(Node::as_call_node)
            .filter(|ancestor| ancestor.location().start_offset() == chain_start)
            .filter_map(|ancestor| {
                let block = ancestor.block().and_then(|block| block.as_block_node())?;
                Some((call_send_end(&ancestor, context.source()), block))
            })
            .filter(|(end, _)| *end > current_end)
            .collect::<Vec<_>>();
        outer_blocks.sort_by_key(|(end, _)| *end);
        let mut end = current_end;
        for (outer_end, block) in outer_blocks {
            end = outer_end;
            if !context.source_file().same_line(
                block.opening_loc().start_offset(),
                block.closing_loc().start_offset(),
            ) {
                break;
            }
        }
        end
    } else {
        current_end
    };
    context.report(
        "Avoid multi-line chains of blocks.",
        closing.start_offset()..offense_end,
    );
}

fn call_send_end(node: &CallNode<'_>, source: &str) -> usize {
    let mut end = node
        .block()
        .and_then(|block| block.as_block_node())
        .map_or_else(|| node.location().end_offset(), |block| block.opening_loc().start_offset());
    while end > node.location().start_offset()
        && source.as_bytes().get(end - 1).is_some_and(u8::is_ascii_whitespace)
    {
        end -= 1;
    }
    end
}
