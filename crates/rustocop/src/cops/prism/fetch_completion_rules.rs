use super::*;

define_cops! {
    UselessDefaultValueArgument => "Lint/UselessDefaultValueArgument" => call(useless_default_value_argument),
    RedundantFetchBlock => "Style/RedundantFetchBlock" => call(redundant_fetch_block),
}

fn useless_default_value_argument(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node
        .block()
        .and_then(|block| block.as_block_node())
        .is_none()
    {
        return;
    }
    let eligible = call_name(node) == b"fetch"
        && node.receiver().is_some_and(|receiver| {
            !context
                .policy()
                .allows_receiver(context.source_file().node(&receiver).as_bytes())
        })
        || call_name(node) == b"new" && root_constant(node.receiver(), b"Array");
    if !eligible {
        return;
    }
    let Some(arguments) = node.arguments() else {
        return;
    };
    let arguments = arguments.arguments();
    if arguments.len() != 2 {
        return;
    }
    let (Some(first), Some(default)) = (arguments.first(), arguments.iter().nth(1)) else {
        return;
    };
    if default.as_keyword_hash_node().is_some() || default.as_splat_node().is_some() {
        return;
    }
    context.remove_list_element(
        &default,
        Some(&first),
        None,
        "Block supersedes default value argument.",
    );
}

fn redundant_fetch_block(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) != b"fetch" || argument_count(node) != 1 || rails_cache_receiver(node) {
        return;
    }
    let Some(block) = node.block().and_then(|block| block.as_block_node()) else {
        return;
    };
    if block.parameters().is_some() {
        return;
    }
    let (fallback, empty_block) = match block.body() {
        None => ("nil".to_string(), true),
        Some(body) => {
            let Some(body) = single_expression(body) else {
                return;
            };
            if !safe_fallback(&body, context) {
                return;
            }
            (context.source_file().node(&body).to_string(), false)
        }
    };
    let Some(key) = only_argument(node) else {
        return;
    };
    let Some(selector) = node.message_loc() else {
        return;
    };
    let preferred = format!("fetch({}, {fallback})", context.source_file().node(&key));
    let offense = selector.start_offset()..block.location().end_offset();
    let block_display = if empty_block {
        "{}".to_string()
    } else {
        format!("{{ {fallback} }}")
    };
    let current = format!("fetch({}) {block_display}", context.source_file().node(&key));
    context.replace(
        format!("Use `{preferred}` instead of `{current}`."),
        offense.clone(),
        offense,
        preferred,
    );
}

fn safe_fallback(node: &Node<'_>, context: &CopContext<'_, '_>) -> bool {
    if node.as_symbol_node().is_some()
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_nil_node().is_some()
    {
        return true;
    }
    if constant_path(node).is_some() {
        return context.config_bool("SafeForConstants", false);
    }
    if node.as_string_node().is_some() {
        return context.source().lines().take(2).any(|line| {
            line.trim()
                .eq_ignore_ascii_case("# frozen_string_literal: true")
        });
    }
    let source = context.source_file().node(node);
    numeric_suffix_literal(source)
}

fn numeric_suffix_literal(source: &str) -> bool {
    let Some(number) = source
        .strip_suffix('r')
        .or_else(|| source.strip_suffix('i'))
    else {
        return false;
    };
    !number.is_empty()
        && number
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'+' | b'-'))
}

fn rails_cache_receiver(node: &CallNode<'_>) -> bool {
    node.receiver()
        .and_then(|receiver| receiver.as_call_node())
        .is_some_and(|receiver| {
            call_name(&receiver) == b"cache"
                && root_constant(receiver.receiver(), b"Rails")
                && argument_count(&receiver) == 0
        })
}
