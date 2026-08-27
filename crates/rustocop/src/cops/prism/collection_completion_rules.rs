use ruby_prism::CallNode;

use super::*;

define_cops! {
    CollectionCompact => "Style/CollectionCompact" => call(collection_compact),
    EachWithObject => "Style/EachWithObject" => call(each_with_object),
}

fn concat_array_literals(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) != b"concat" || node.receiver().is_none() {
        return;
    }
    let Some(arguments) = node.arguments() else {
        return;
    };
    let arrays = arguments
        .arguments()
        .iter()
        .map(|argument| argument.as_array_node())
        .collect::<Option<Vec<_>>>();
    let Some(arrays) = arrays else {
        return;
    };
    if arrays.is_empty() {
        return;
    }
    let file = context.source_file();
    let selector = node.message_loc().expect("concat has a selector");
    let offense = selector.start_offset()..node.location().end_offset();
    let original = context.source()[offense.clone()].to_string();
    let dynamic_percent = arrays.iter().any(|array| {
        let source = file.node(&array.as_node());
        (source.starts_with("%I") || source.starts_with("%W")) && source.contains("#{")
    });
    if dynamic_percent {
        context.report(
            format!("Use `push` with elements as arguments without array brackets instead of `{original}`."),
            offense,
        );
        return;
    }
    let rendered = if arrays.len() == 1 {
        let array = &arrays[0];
        let source = file.node(&array.as_node());
        if source.contains('\n') {
            let (Some(opening), Some(closing)) = (array.opening_loc(), array.closing_loc()) else {
                return;
            };
            context.source()[opening.end_offset()..closing.start_offset()].to_string()
        } else {
            array
                .elements()
                .iter()
                .map(|element| render_pushed_element(&element, source, file))
                .collect::<Vec<_>>()
                .join(", ")
        }
    } else {
        arrays
            .iter()
            .flat_map(|array| {
                let source = file.node(&array.as_node());
                array
                    .elements()
                    .iter()
                    .map(move |element| render_pushed_element(&element, source, file))
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let advertised = arrays
        .iter()
        .flat_map(|array| {
            let source = file.node(&array.as_node());
            array
                .elements()
                .iter()
                .map(move |element| render_pushed_element(&element, source, file))
        })
        .collect::<Vec<_>>()
        .join(", ");
    let preferred = format!("push({rendered})");
    context.replace(
        format!("Use `push({advertised})` instead of `{original}`."),
        offense.clone(),
        offense,
        preferred,
    );
}

fn render_pushed_element(node: &Node<'_>, array_source: &str, file: SourceFile<'_>) -> String {
    if array_source.starts_with("%i") {
        return static_symbol(node)
            .map(|symbol| format!(":{}", String::from_utf8_lossy(&symbol)))
            .unwrap_or_else(|| file.node(node).to_string());
    }
    if array_source.starts_with("%w") {
        return static_string(node)
            .map(|string| format!("{:?}", String::from_utf8_lossy(&string)))
            .unwrap_or_else(|| file.node(node).to_string());
    }
    file.node(node).to_string()
}

fn collection_compact(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(receiver) = node.receiver() else {
        return;
    };
    let method = call_name(node);
    if (!context.target_ruby_version().at_least(3, 1)
        && receiver
            .as_call_node()
            .is_some_and(|call| matches!(call_name(&call), b"to_enum" | b"lazy")))
        || (!context.target_ruby_version().at_least(2, 6)
            && matches!(method, b"filter" | b"filter!"))
        || (!context.target_ruby_version().at_least(2, 4)
            && matches!(method, b"reject" | b"reject!"))
    {
        return;
    }
    if receiver_uses_allowed_name(&receiver, context) {
        return;
    }
    let preferred = match method {
        b"reject" | b"select" | b"filter" | b"grep_v" => "compact",
        b"reject!" | b"select!" | b"filter!" => "compact!",
        _ => return,
    };
    let selector = node.message_loc().expect("selected calls have a selector");
    let offense = selector.start_offset()..node.location().end_offset();
    let original = context.source()[offense.clone()].to_string();

    let matches = if method == b"grep_v" {
        only_argument(node).is_some_and(|argument| {
            argument.as_nil_node().is_some() || root_constant(Some(argument), b"NilClass")
        })
    } else if original.contains("&:nil?") {
        matches!(method, b"reject" | b"reject!")
    } else {
        let Some(block) = node.block().and_then(|block| block.as_block_node()) else {
            return;
        };
        let Some(body) = block.body() else {
            return;
        };
        collection_compact_block_matches(&block, &body, method, context.source_file())
    };
    if !matches {
        return;
    }
    context.replace(
        format!("Use `{preferred}` instead of `{original}`."),
        offense.clone(),
        offense,
        preferred,
    );
}

fn collection_compact_block_matches(
    block: &ruby_prism::BlockNode<'_>,
    body: &Node<'_>,
    method: &[u8],
    file: SourceFile<'_>,
) -> bool {
    let body = file.node(body).chars().filter(|ch| !ch.is_whitespace()).collect::<String>();
    let parameter = block
        .parameters()
        .and_then(|parameters| parameters.as_block_parameters_node())
        .and_then(|parameters| parameters.parameters())
        .and_then(|parameters| parameters.requireds().last())
        .and_then(|parameter| parameter.as_required_parameter_node())
        .map(|parameter| String::from_utf8_lossy(parameter.name().as_slice()).into_owned())
        .unwrap_or_else(|| {
            if body.contains("_1") {
                "_1".to_string()
            } else {
                "it".to_string()
            }
        });
    let nil = format!("{parameter}.nil?");
    let safe_nil = format!("{parameter}&.nil?");
    if matches!(method, b"reject" | b"reject!") {
        body == nil || body == safe_nil
    } else {
        body == format!("!{nil}") || body == format!("{safe_nil}&.!")
    }
}

fn receiver_uses_allowed_name(node: &Node<'_>, context: &CopContext<'_, '_>) -> bool {
    node.as_call_node().is_some_and(|call| {
        context.policy().allows_receiver(call_name(&call))
            || call
                .receiver()
                .as_ref()
                .is_some_and(|receiver| receiver_uses_allowed_name(receiver, context))
    })
}

fn each_with_object(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !matches!(node.name().as_slice(), b"inject" | b"reduce")
        || only_argument(node).is_none_or(|argument| each_with_object_basic_literal(&argument))
    {
        return;
    }
    let Some(block) = node.block().and_then(|block| block.as_block_node()) else {
        return;
    };
    let Some(statements) = block.body().and_then(|body| body.as_statements_node()) else {
        return;
    };
    let Some(last) = statements.body().last() else {
        return;
    };
    let file = context.source_file();
    let last_source = file.node(&last).trim();
    let parameter_node = block.parameters();
    let parameters = parameter_node
        .as_ref()
        .map(|parameters| file.node(parameters));
    let explicit = parameters.and_then(two_parameters);
    let numbered = parameter_node
        .as_ref()
        .is_some_and(|parameters| parameters.as_numbered_parameters_node().is_some())
        && last_source == "_1";
    let accumulator = explicit.map(|(first, _)| first).unwrap_or("_1");
    if (!numbered && explicit.is_none())
        || last_source != accumulator
        || accumulator_assigned_in_block(file.node(&block.as_node()), accumulator)
    {
        return;
    }

    let selector = node
        .message_loc()
        .expect("inject and reduce have selectors");
    let mut edits = vec![(
        selector.start_offset()..selector.end_offset(),
        "each_with_object".to_string(),
    )];
    if let (Some(parameter_node), Some((first, second))) = (parameter_node, explicit) {
        let location = parameter_node.location();
        edits.push((
            location.start_offset()..location.end_offset(),
            format!("|{second}, {first}|"),
        ));
    } else {
        let body_location = statements.location();
        let swap_end = body_location.end_offset();
        for (offset, name) in file
            .slice(body_location.start_offset()..swap_end)
            .unwrap_or_default()
            .match_indices("_1")
        {
            let start = body_location.start_offset() + offset;
            edits.push((start..start + name.len(), "_2".to_string()));
        }
        for (offset, name) in file
            .slice(body_location.start_offset()..swap_end)
            .unwrap_or_default()
            .match_indices("_2")
        {
            let start = body_location.start_offset() + offset;
            edits.push((start..start + name.len(), "_1".to_string()));
        }
    }
    if !numbered {
        let last_location = last.location();
        let removal = if file.same_line(
            block.opening_loc().start_offset(),
            last_location.start_offset(),
        ) {
            last_location.start_offset()..last_location.end_offset()
        } else {
            file.line_range(last_location.start_offset())
        };
        edits.push((removal, String::new()));
    }
    context.replace_many(
        format!(
            "Use `each_with_object` instead of `{}`.",
            String::from_utf8_lossy(node.name().as_slice())
        ),
        &selector,
        edits,
    );
}

fn each_with_object_basic_literal(node: &Node<'_>) -> bool {
    node.as_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_rational_node().is_some()
        || node.as_imaginary_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_nil_node().is_some()
}

fn two_parameters(source: &str) -> Option<(&str, &str)> {
    let source = source.strip_prefix('|')?.strip_suffix('|')?;
    let mut depth = 0_usize;
    let mut separator = None;
    for (index, byte) in source.bytes().enumerate() {
        match byte {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                if separator.is_some() {
                    return None;
                }
                separator = Some(index);
            }
            _ => {}
        }
    }
    let separator = separator?;
    let first = source[..separator].trim();
    let second = source[separator + 1..].trim();
    (!first.is_empty() && !second.is_empty()).then_some((first, second))
}

fn accumulator_assigned_in_block(source: &str, accumulator: &str) -> bool {
    [" =", " +=", " -=", " *=", " /=", " ||=", " &&="]
        .iter()
        .any(|operator| source.contains(&format!("{accumulator}{operator}")))
}
