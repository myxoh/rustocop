use ruby_prism::CallNode;

use super::*;

define_cops! {
    MapToHash => "Style/MapToHash" => rubocop_callbacks(
        MapToHashRule,
        [on_send restrict [b"to_h"]]
    ),
}

impl MapToHashRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        return_if!(!self.target_ruby_version().at_least(2, 6));
        let Some(map) = map_before_conversion(node) else {
            return;
        };
        return_if!(node.block().is_some());

        let method = String::from_utf8_lossy(map.name().as_slice());
        let dot = node
            .call_operator_loc()
            .map(|location| String::from_utf8_lossy(location.as_slice()).into_owned())
            .unwrap_or_else(|| ".".to_string());
        let message = format!("Pass a block to `to_h` instead of calling `{method}{dot}to_h`.");
        let destructuring = destructuring_parameter(&map, &self.source_file());
        register_map_conversion(
            self.context,
            node,
            &map,
            "to_h",
            message,
            true,
            destructuring,
        );
    }
}

fn map_before_conversion<'pr>(node: &CallNode<'pr>) -> Option<CallNode<'pr>> {
    let map = node.receiver()?.as_call_node()?;
    if !matches!(map.name().as_slice(), b"map" | b"collect") {
        return None;
    }
    let literal_block = map
        .block()
        .is_some_and(|block| block.as_block_node().is_some());
    if literal_block && argument_count(&map) != 0 || !literal_block && !has_symbol_block_pass(&map) {
        return None;
    }
    Some(map)
}

fn has_symbol_block_pass(node: &CallNode<'_>) -> bool {
    node.block()
        .and_then(|block| block.as_block_argument_node())
        .is_some_and(|block| {
            block
                .expression()
                .is_some_and(|expression| expression.as_symbol_node().is_some())
        })
}

fn register_map_conversion(
    context: &mut CopContext<'_, '_>,
    node: &CallNode<'_>,
    map: &CallNode<'_>,
    replacement: &str,
    message: String,
    preserve_final_operator: bool,
    destructuring: Option<(std::ops::Range<usize>, String)>,
) {
    let (Some(map_selector), Some(final_selector), Some(final_operator)) = (
        map.message_loc(),
        node.message_loc(),
        node.call_operator_loc(),
    ) else {
        return;
    };
    let mut removal_start = final_operator.start_offset();
    let source = context.source().as_bytes();
    if source[..removal_start]
        .iter()
        .rev()
        .take_while(|byte| byte.is_ascii_whitespace() && **byte != b'\n')
        .count()
        > 0
    {
        while removal_start > 0 && source[removal_start - 1].is_ascii_whitespace() {
            removal_start -= 1;
            if source[removal_start] == b'\n' {
                break;
            }
        }
    }
    let removal = removal_start..final_selector.end_offset();
    add_offense!(context, &map_selector, message: message, |corrector| {
        corrector.replace(&map_selector, replacement);
        corrector.remove(removal);
        if preserve_final_operator {
            if let Some(map_operator) = map.call_operator_loc() {
                corrector.replace(
                    map_operator,
                    String::from_utf8_lossy(final_operator.as_slice()).into_owned(),
                );
            }
        }
        if let Some((range, replacement)) = destructuring {
            corrector.replace(range, replacement);
        }
    });
}

fn destructuring_parameter(
    map: &CallNode<'_>,
    source_file: &SourceFile<'_>,
) -> Option<(std::ops::Range<usize>, String)> {
    let block = map.block()?.as_block_node()?;
    let block_parameters = block.parameters()?.as_block_parameters_node()?;
    let parameters = block_parameters.parameters()?;
    if parameters.requireds().len() != 1 {
        return None;
    }
    let target = parameters.requireds().first()?.as_multi_target_node()?;
    let range = target.location().start_offset()..target.location().end_offset();
    let source = source_file.node(&target.as_node());
    let replacement = source.strip_prefix('(')?.strip_suffix(')')?.to_string();
    Some((range, replacement))
}
