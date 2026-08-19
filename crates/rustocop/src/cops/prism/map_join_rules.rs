use super::*;

define_cops! {
    MapJoin => "Style/MapJoin" => call(map_join),
}

fn map_join(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) != b"join" {
        return;
    }
    let Some(map) = node.receiver().and_then(|receiver| receiver.as_call_node()) else {
        return;
    };
    if !matches!(call_name(&map), b"map" | b"collect") || !maps_to_string(&map) {
        return;
    }
    let Some(selector) = map.message_loc() else {
        return;
    };
    let method = String::from_utf8_lossy(call_name(&map));
    let message = format!("Remove redundant `{method}(&:to_s)` before `join`.");
    let file = context.source_file();
    if map.receiver().is_some()
        && !file.same_line(
            map.location().start_offset(),
            node.message_loc().unwrap().start_offset(),
        )
    {
        context.remove(message, &selector, file.line_range(selector.start_offset()));
        return;
    }
    let edit_start = map
        .receiver()
        .map_or(map.location().start_offset(), |receiver| {
            receiver.location().end_offset()
        });
    let join_selector = node.message_loc().expect("join has a selector");
    let replacement = if map.receiver().is_some() {
        node.call_operator_loc()
            .map(|operator| String::from_utf8_lossy(operator.as_slice()).into_owned())
            .unwrap_or_else(|| ".".to_string())
    } else {
        String::new()
    };
    context.replace(
        message,
        &selector,
        edit_start..join_selector.start_offset(),
        replacement,
    );
}

fn maps_to_string(node: &CallNode<'_>) -> bool {
    if argument_count(node) != 0 {
        return false;
    }
    let Some(block) = node.block() else {
        return false;
    };
    if let Some(argument) = block.as_block_argument_node() {
        return argument
            .expression()
            .and_then(|expression| expression.as_symbol_node())
            .is_some_and(|symbol| symbol.unescaped() == b"to_s");
    }
    let Some(block) = block.as_block_node() else {
        return false;
    };
    let Some(body) = block.body().and_then(single_expression) else {
        return false;
    };
    let Some(call) = body.as_call_node() else {
        return false;
    };
    if call_name(&call) != b"to_s" || argument_count(&call) != 0 {
        return false;
    }
    block_parameter_matches_receiver(&block, call.receiver())
}

fn single_expression(body: Node<'_>) -> Option<Node<'_>> {
    let statements = body.as_statements_node()?;
    (statements.body().len() == 1)
        .then(|| statements.body().first())
        .flatten()
}

fn block_parameter_matches_receiver(
    block: &ruby_prism::BlockNode<'_>,
    receiver: Option<Node<'_>>,
) -> bool {
    let (Some(parameters), Some(receiver)) = (block.parameters(), receiver) else {
        return false;
    };
    if let Some(numbered) = parameters.as_numbered_parameters_node() {
        return numbered.maximum() == 1
            && receiver
                .as_local_variable_read_node()
                .is_some_and(|read| read.name().as_slice() == b"_1");
    }
    if parameters.as_it_parameters_node().is_some() {
        return receiver.as_it_local_variable_read_node().is_some();
    }
    let Some(parameters) = parameters
        .as_block_parameters_node()
        .and_then(|parameters| parameters.parameters())
    else {
        return false;
    };
    if parameters.requireds().len() != 1 {
        return false;
    }
    let Some(parameter) = parameters
        .requireds()
        .first()
        .and_then(|parameter| parameter.as_required_parameter_node())
    else {
        return false;
    };
    receiver
        .as_local_variable_read_node()
        .is_some_and(|read| read.name().as_slice() == parameter.name().as_slice())
}
