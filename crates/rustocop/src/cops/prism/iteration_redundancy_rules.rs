use super::*;

define_cops! {
    RedundantEach => "Style/RedundantEach" => call(redundant_each),
    RedundantWithIndex => "Lint/RedundantWithIndex" => call(redundant_with_index),
    RedundantWithObject => "Lint/RedundantWithObject" => call(redundant_with_object),
    UselessTimes => "Lint/UselessTimes" => call(useless_times),
}

fn redundant_each(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(inner) = node.receiver().and_then(|receiver| receiver.as_call_node()) else {
        return;
    };
    if inner.block().is_some() {
        return;
    }
    let inner_name = call_name(&inner);
    match call_name(node) {
        b"each" | b"reverse_each"
            if argument_count(node) == 0
                && argument_count(&inner) == 0
                && matches!(inner_name, b"each" | b"reverse_each") =>
        {
            if call_name(node) == inner_name && inner_name == b"reverse_each" {
                return;
            }
            let (Some(receiver), Some(inner_selector), Some(outer_selector)) =
                (inner.receiver(), inner.message_loc(), node.message_loc())
            else {
                return;
            };
            let offense = if inner_name == b"each" {
                inner_selector.start_offset()..outer_selector.start_offset()
            } else {
                node.call_operator_loc()
                    .map_or(outer_selector.start_offset(), |operator| {
                        operator.start_offset()
                    })..outer_selector.end_offset()
            };
            let edit = if call_name(node) == b"each" {
                node.call_operator_loc()
                    .map_or(outer_selector.start_offset(), |operator| {
                        operator.start_offset()
                    })..outer_selector.end_offset()
            } else {
                receiver.location().end_offset()..inner.location().end_offset()
            };
            context.remove("Remove redundant `each`.", offense, edit);
        }
        b"each_with_index" | b"each_with_object"
            if matches!(inner_name, b"each" | b"reverse_each")
                || inner_name.starts_with(b"each_") =>
        {
            let preferred = if call_name(node) == b"each_with_index" {
                "with_index"
            } else {
                "with_object"
            };
            let Some(selector) = node.message_loc() else {
                return;
            };
            if inner_name == b"each" && argument_count(&inner) == 0 {
                let Some(inner_selector) = inner.message_loc() else {
                    return;
                };
                context.replace(
                    "Remove redundant `each`.",
                    inner_selector.start_offset()..selector.start_offset(),
                    &selector,
                    preferred,
                );
            } else {
                context.replace(
                    format!("Use `{preferred}` to remove redundant `each`."),
                    &selector,
                    &selector,
                    preferred,
                );
            }
        }
        _ => {}
    }
}

fn useless_times(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) != b"times" || argument_count(node) != 0 {
        return;
    }
    let Some(receiver) = node.receiver() else {
        return;
    };
    let receiver_source = context.source_file().node(&receiver);
    let Ok(count) = receiver_source.parse::<i64>() else {
        return;
    };
    if count > 1 {
        return;
    }
    let location = node.location();
    let message = format!("Useless call to `{receiver_source}.times` detected.");
    if node.block().is_none()
        || context
            .parent()
            .is_some_and(|parent| parent.as_call_node().is_some())
        || count == 1 && times_block_argument_is_written(node, context.source_file())
    {
        context.report(message, &location);
        return;
    }
    let source = context.source_file().at(&location);
    let line_start = context.source_file().line_start(location.start_offset());
    let line_end = context.source()[location.end_offset()..]
        .find('\n')
        .map_or(context.source().len(), |at| location.end_offset() + at + 1);
    let indentation = &context.source()[line_start..location.start_offset()];
    let replacement = if count <= 0 {
        String::new()
    } else if let Some(method) = source
        .strip_prefix("1.times(&:")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        format!("{indentation}{method}\n")
    } else if let (Some(open), Some(close)) = (source.find('{'), source.rfind('}')) {
        let mut body = source[open + 1..close].trim().to_string();
        if body.starts_with('|') {
            if let Some(end) = body[1..].find('|').map(|at| at + 1) {
                let parameter = body[1..end].trim().to_string();
                body = body[end + 1..].trim().to_string();
                if !parameter.is_empty() {
                    body = replace_identifier(&body, &parameter, "0");
                }
            }
        }
        (!body.is_empty())
            .then(|| format!("{indentation}{body}\n"))
            .unwrap_or_default()
    } else if let Some(header_end) = source.find('\n') {
        let header = &source[..header_end];
        let parameter = header
            .find('|')
            .and_then(|start| {
                header[start + 1..]
                    .find('|')
                    .map(|end| &header[start + 1..start + 1 + end])
            })
            .map(str::trim)
            .filter(|name| !name.is_empty());
        let closing_start = source.rfind('\n').unwrap_or(source.len());
        let body = if closing_start > header_end {
            &source[header_end + 1..closing_start]
        } else {
            ""
        };
        let body_indentation = body
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.len() - line.trim_start().len())
            .min()
            .unwrap_or(0);
        let mut body = body
            .lines()
            .map(|line| {
                let line = &line[body_indentation.min(line.len())..];
                if line.is_empty() {
                    String::new()
                } else {
                    format!("{indentation}{line}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        if let Some(parameter) = parameter {
            body = replace_identifier(&body, parameter, "0");
        }
        (!body.trim().is_empty())
            .then(|| format!("{body}\n"))
            .unwrap_or_default()
    } else {
        String::new()
    };
    context.replace(message, &location, line_start..line_end, replacement);
}

fn replace_identifier(source: &str, name: &str, replacement: &str) -> String {
    let mut result = String::new();
    let mut search = 0usize;
    for (at, _) in source.match_indices(name) {
        let before = source.as_bytes().get(at.wrapping_sub(1));
        let after = source.as_bytes().get(at + name.len());
        if before.is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            || after.is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            continue;
        }
        result.push_str(&source[search..at]);
        result.push_str(replacement);
        search = at + name.len();
    }
    result.push_str(&source[search..]);
    result
}

fn times_block_argument_is_written(node: &CallNode<'_>, file: SourceFile<'_>) -> bool {
    let Some(block) = node.block() else {
        return false;
    };
    let source = file.node(&block);
    let Some(after_open) = source.find('|').map(|at| at + 1) else {
        return false;
    };
    let Some(close) = source[after_open..].find('|').map(|at| after_open + at) else {
        return false;
    };
    let argument = source[after_open..close].trim();
    if argument.is_empty() || argument.contains(',') {
        return false;
    }
    source[close + 1..].lines().any(|line| {
        let line = line.trim_start();
        line.starts_with(&format!("{argument} +="))
            || line.starts_with(&format!("{argument} -="))
            || line.starts_with(&format!("{argument} ="))
            || line.starts_with(&format!("{argument},"))
    })
}

fn redundant_with_index(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(block) = node.block().and_then(|block| block.as_block_node()) else {
        return;
    };
    if block_accepts_second_argument(&block) {
        return;
    }
    match call_name(node) {
        b"each_with_index" if node.receiver().is_some() && argument_count(node) == 0 => {
            context.replace_selector(node, "Use `each` instead of `each_with_index`.", "each");
        }
        b"with_index" if redundant_enumerator_receiver(node) => {
            remove_chained_enumerator_call(node, &block, context, "with_index");
        }
        _ => {}
    }
}

fn redundant_with_object(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(block) = node.block().and_then(|block| block.as_block_node()) else {
        return;
    };
    if block_accepts_second_argument(&block) {
        return;
    }
    match call_name(node) {
        b"each_with_object" if node.receiver().is_some() && argument_count(node) == 1 => {
            let Some(selector) = node.message_loc() else {
                return;
            };
            let call_end = call_end_before_block(&block, context.source());
            context.replace(
                "Use `each` instead of `each_with_object`.",
                selector.start_offset()..call_end,
                selector.start_offset()..call_end,
                "each",
            );
        }
        b"with_object" if argument_count(node) == 1 && redundant_enumerator_receiver(node) => {
            remove_chained_enumerator_call(node, &block, context, "with_object");
        }
        _ => {}
    }
}

fn remove_chained_enumerator_call(
    node: &CallNode<'_>,
    block: &ruby_prism::BlockNode<'_>,
    context: &mut CopContext<'_, '_>,
    method: &str,
) {
    let (Some(receiver), Some(selector)) = (node.receiver(), node.message_loc()) else {
        return;
    };
    let call_end = call_end_before_block(block, context.source());
    context.remove(
        format!("Remove redundant `{method}`."),
        selector.start_offset()..call_end,
        receiver.location().end_offset()..call_end,
    );
}

fn call_end_before_block(block: &ruby_prism::BlockNode<'_>, source: &str) -> usize {
    let mut end = block.location().start_offset();
    while end > 0 && matches!(source.as_bytes()[end - 1], b' ' | b'\t') {
        end -= 1;
    }
    end
}

fn redundant_enumerator_receiver(node: &CallNode<'_>) -> bool {
    node.receiver()
        .and_then(|receiver| receiver.as_call_node())
        .is_some_and(|receiver| {
            matches!(call_name(&receiver), b"each" | b"each_with_object")
                && receiver.block().is_none()
        })
}

fn block_accepts_second_argument(block: &ruby_prism::BlockNode<'_>) -> bool {
    let Some(parameters) = block.parameters() else {
        return false;
    };
    if let Some(numbered) = parameters.as_numbered_parameters_node() {
        return numbered.maximum() >= 2;
    }
    let Some(block_parameters) = parameters.as_block_parameters_node() else {
        return false;
    };
    let Some(parameters) = block_parameters.parameters() else {
        return false;
    };
    parameters.requireds().len() >= 2
        || !parameters.optionals().is_empty()
        || parameters.rest().is_some()
}
