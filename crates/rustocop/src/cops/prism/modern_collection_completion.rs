use super::*;

define_cops! {
    ArrayIntersect => "Style/ArrayIntersect" => source(array_intersect),
    RedundantMinMaxBy => "Style/RedundantMinMaxBy" => source(redundant_min_max_by),
    RedundantSort => "Style/RedundantSort" => source(redundant_sort),
    TallyMethod => "Style/TallyMethod" => call(tally_method),
    ZeroLengthPredicate => "Style/ZeroLengthPredicate" => call(zero_length_predicate),
}

fn array_intersect(context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(3, 1) {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        let code = line.trim();
        let Some(prefix) = code
            .strip_suffix(".any?")
            .or_else(|| code.strip_suffix(".empty?"))
            .or_else(|| code.strip_suffix(".none?"))
        else {
            continue;
        };
        let negated = code.ends_with(".empty?") || code.ends_with(".none?");
        let inner = prefix.trim_matches(['(', ')']);
        let Some((left, right)) = inner.split_once(" & ") else {
            continue;
        };
        let replacement = format!(
            "{}{}.intersect?({})",
            if negated { "!" } else { "" },
            left.trim(),
            right.trim()
        );
        let start = offset + line.find(code).unwrap_or(0);
        context.replace(
            format!("Use `{replacement}` instead of `{code}`."),
            start..start + code.len(),
            start..start + code.len(),
            replacement,
        );
    }
}

fn redundant_min_max_by(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for method in ["max_by", "min_by"] {
        let needle = format!(".{method} {{ |");
        let mut search = 0;
        while let Some(relative) = source[search..].find(&needle) {
            let dot = search + relative;
            let Some(pipe_relative) = source[dot + needle.len()..].find('|') else {
                break;
            };
            let pipe = dot + needle.len() + pipe_relative;
            let parameter = source[dot + needle.len()..pipe].trim();
            let Some(close_relative) = source[pipe + 1..].find('}') else {
                break;
            };
            let end = pipe + 1 + close_relative + 1;
            if source[pipe + 1..end - 1].trim() != parameter {
                search = end;
                continue;
            }
            let preferred = method.trim_end_matches("_by");
            context.replace(
                format!("Use `{preferred}` instead of `{method} {{ |{parameter}| {parameter} }}`."),
                dot + 1..end,
                dot + 1..end,
                preferred,
            );
            search = end;
        }
    }
    for method in ["max_by", "min_by"] {
        let needle = format!(".{method} do |");
        let mut search = 0;
        while let Some(relative) = source[search..].find(&needle) {
            let dot = search + relative;
            let Some(pipe_relative) = source[dot + needle.len()..].find('|') else {
                break;
            };
            let pipe = dot + needle.len() + pipe_relative;
            let parameter = source[dot + needle.len()..pipe].trim();
            let Some(end_relative) = source[pipe + 1..].find("\nend") else {
                break;
            };
            let end = pipe + 1 + end_relative + "\nend".len();
            if source[pipe + 1..pipe + 1 + end_relative].trim() != parameter {
                search = end;
                continue;
            }
            let preferred = method.trim_end_matches("_by");
            context.replace(
                format!("Use `{preferred}` instead of `{}`.", &source[dot + 1..end]),
                dot + 1..end,
                dot + 1..end,
                preferred,
            );
            search = end;
        }
    }
}

fn redundant_sort(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for (suffix, replacement, message_name) in [
        (".sort.first", ".min", "min"),
        (".sort.last", ".max", "max"),
        (".sort_by.first", ".min_by", "min_by"),
        (".sort_by.last", ".max_by", "max_by"),
    ] {
        for start in source
            .match_indices(suffix)
            .map(|(at, _)| at)
            .collect::<Vec<_>>()
        {
            context.replace(
                format!(
                    "Use `{message_name}` instead of `{}...{}`.",
                    suffix.split('.').nth(1).unwrap_or("sort"),
                    suffix.rsplit('.').next().unwrap_or_default()
                ),
                start + 1..start + suffix.len(),
                start..start + suffix.len(),
                replacement,
            );
        }
        let safe_suffix = suffix.replace('.', "&.");
        for start in source
            .match_indices(&safe_suffix)
            .map(|(at, _)| at)
            .collect::<Vec<_>>()
        {
            context.replace(
                format!(
                    "Use `{message_name}` instead of `{}...{}`.",
                    suffix.split('.').nth(1).unwrap_or("sort"),
                    suffix.rsplit('.').next().unwrap_or_default()
                ),
                start + 2..start + safe_suffix.len(),
                start..start + safe_suffix.len(),
                replacement.replace('.', "&."),
            );
        }
    }
}

fn tally_method(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(2, 7) {
        return;
    }
    if call_name(node) == b"each_with_object" && each_with_object_tally(node, context) {
        let Some(selector) = node.message_loc() else {
            return;
        };
        context.replace(
            "Use `tally` instead of `each_with_object`.",
            &selector,
            selector.start_offset()..node.location().end_offset(),
            "tally",
        );
    } else if call_name(node) == b"transform_values" && transform_values_tally(node) {
        let Some(group_by) = node.receiver().and_then(|receiver| receiver.as_call_node()) else {
            return;
        };
        let Some(selector) = group_by.message_loc() else {
            return;
        };
        context.replace(
            "Use `tally` instead of `group_by` and `transform_values`.",
            &selector,
            selector.start_offset()..node.location().end_offset(),
            "tally",
        );
    }
}

fn each_with_object_tally(node: &CallNode<'_>, context: &CopContext<'_, '_>) -> bool {
    if argument_count(node) != 1 {
        return false;
    }
    let Some(initializer) = only_argument(node).and_then(|argument| argument.as_call_node()) else {
        return false;
    };
    if call_name(&initializer) != b"new" || argument_count(&initializer) != 1 {
        return false;
    }
    let hash_receiver = initializer
        .receiver()
        .is_some_and(|receiver| matches!(context.source_file().node(&receiver), "Hash" | "::Hash"));
    let zero = only_argument(&initializer)
        .and_then(|argument| argument.as_integer_node())
        .is_some_and(|integer| TryInto::<i32>::try_into(integer.value()).ok() == Some(0));
    let Some(block) = node.block().and_then(|block| block.as_block_node()) else {
        return false;
    };
    let Some(body) = block.body().and_then(single_expression) else {
        return false;
    };
    let Some(write) = body.as_index_operator_write_node() else {
        return false;
    };
    if write.binary_operator().as_slice() != b"+"
        || !write
            .value()
            .as_integer_node()
            .is_some_and(|integer| TryInto::<i32>::try_into(integer.value()).ok() == Some(1))
        || write
            .arguments()
            .is_none_or(|arguments| arguments.arguments().len() != 1)
    {
        return false;
    }
    let Some(receiver) = write
        .receiver()
        .and_then(|receiver| receiver.as_local_variable_read_node())
    else {
        return false;
    };
    let Some(key) = write
        .arguments()
        .and_then(|arguments| arguments.arguments().first())
        .and_then(|argument| argument.as_local_variable_read_node())
    else {
        return false;
    };
    hash_receiver && zero && tally_block_parameters(&block, receiver.name().as_slice(), key.name().as_slice())
}

fn tally_block_parameters(block: &ruby_prism::BlockNode<'_>, hash: &[u8], element: &[u8]) -> bool {
    let Some(parameters) = block.parameters() else {
        return false;
    };
    if let Some(numbered) = parameters.as_numbered_parameters_node() {
        return numbered.maximum() == 2 && hash == b"_2" && element == b"_1";
    }
    let Some(parameters) = parameters
        .as_block_parameters_node()
        .and_then(|parameters| parameters.parameters())
    else {
        return false;
    };
    if parameters.requireds().len() != 2
        || !parameters.optionals().is_empty()
        || parameters.rest().is_some()
        || !parameters.posts().is_empty()
        || !parameters.keywords().is_empty()
        || parameters.keyword_rest().is_some()
        || parameters.block().is_some()
    {
        return false;
    }
    let Some(first) = parameters
        .requireds()
        .first()
        .and_then(|parameter| parameter.as_required_parameter_node())
    else {
        return false;
    };
    let Some(second) = parameters
        .requireds()
        .last()
        .and_then(|parameter| parameter.as_required_parameter_node())
    else {
        return false;
    };
    first.name().as_slice() == element && second.name().as_slice() == hash
}

fn transform_values_tally(node: &CallNode<'_>) -> bool {
    if argument_count(node) != 0 {
        return false;
    }
    let Some(group_by) = node.receiver().and_then(|receiver| receiver.as_call_node()) else {
        return false;
    };
    call_name(&group_by) == b"group_by"
        && argument_count(&group_by) == 0
        && group_by_identity(&group_by)
        && transform_counts(node)
}

fn group_by_identity(node: &CallNode<'_>) -> bool {
    let Some(block) = node.block() else {
        return false;
    };
    if let Some(argument) = block.as_block_argument_node() {
        return argument
            .expression()
            .and_then(|expression| expression.as_symbol_node())
            .is_some_and(|symbol| symbol.unescaped() == b"itself");
    }
    block
        .as_block_node()
        .is_some_and(|block| identity_block(&block))
}

fn identity_block(block: &ruby_prism::BlockNode<'_>) -> bool {
    let (Some(parameters), Some(body)) = (block.parameters(), block.body().and_then(single_expression))
    else {
        return false;
    };
    if let Some(numbered) = parameters.as_numbered_parameters_node() {
        return numbered.maximum() == 1
            && body
                .as_local_variable_read_node()
                .is_some_and(|read| read.name().as_slice() == b"_1");
    }
    if parameters.as_it_parameters_node().is_some() {
        return body.as_it_local_variable_read_node().is_some();
    }
    let Some(parameters) = parameters
        .as_block_parameters_node()
        .and_then(|parameters| parameters.parameters())
    else {
        return false;
    };
    if parameters.requireds().len() != 1
        || !parameters.optionals().is_empty()
        || parameters.rest().is_some()
        || !parameters.posts().is_empty()
        || !parameters.keywords().is_empty()
        || parameters.keyword_rest().is_some()
        || parameters.block().is_some()
    {
        return false;
    }
    let Some(parameter) = parameters
        .requireds()
        .first()
        .and_then(|parameter| parameter.as_required_parameter_node())
    else {
        return false;
    };
    body.as_local_variable_read_node()
        .is_some_and(|read| read.name().as_slice() == parameter.name().as_slice())
}

fn transform_counts(node: &CallNode<'_>) -> bool {
    let Some(block) = node.block() else {
        return false;
    };
    if let Some(argument) = block.as_block_argument_node() {
        return argument
            .expression()
            .and_then(|expression| expression.as_symbol_node())
            .is_some_and(|symbol| counting_method(symbol.unescaped()));
    }
    let Some(block) = block.as_block_node() else {
        return false;
    };
    let Some(body) = block.body().and_then(single_expression) else {
        return false;
    };
    let Some(count) = body.as_call_node() else {
        return false;
    };
    if !counting_method(call_name(&count)) || argument_count(&count) != 0 {
        return false;
    }
    block_parameter_is_receiver(&block, count.receiver())
}

fn block_parameter_is_receiver(
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

fn single_expression(node: Node<'_>) -> Option<Node<'_>> {
    let statements = node.as_statements_node()?;
    (statements.body().len() == 1)
        .then(|| statements.body().first())
        .flatten()
}

fn counting_method(name: &[u8]) -> bool {
    matches!(name, b"count" | b"size" | b"length")
}

fn zero_length_predicate(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !matches!(call_name(node), b"length" | b"size")
        || argument_count(node) != 0
        || node.receiver().is_none()
    {
        return;
    }
    let Some(parent) = context.parent().and_then(Node::as_call_node) else {
        return;
    };
    if non_polymorphic_collection(node, context) {
        return;
    }

    if call_name(&parent) == b"zero?"
        && argument_count(&parent) == 0
        && parent.receiver().is_some_and(|receiver| same_call(&receiver, node))
    {
        let Some(selector) = node.message_loc() else {
            return;
        };
        let offense = selector.start_offset()..parent.location().end_offset();
        let current = context.source_file().slice(offense.clone()).unwrap_or_default();
        context.replace(
            format!("Use `empty?` instead of `{current}`."),
            offense.clone(),
            offense,
            "empty?",
        );
        return;
    }

    let operator = call_name(&parent);
    if !matches!(operator, b"==" | b"<" | b">" | b"!=") {
        return;
    }
    let (Some(left), Some(right)) = (parent.receiver(), only_argument(&parent)) else {
        return;
    };
    let length_on_left = same_call(&left, node);
    let length_on_right = same_call(&right, node);
    if !length_on_left && !length_on_right {
        return;
    }
    let other = if length_on_left { &right } else { &left };
    let Some(integer) = integer_value(other) else {
        return;
    };
    let zero = matches!(
        (length_on_left, operator, integer),
        (true, b"==", 0) | (false, b"==", 0) | (true, b"<", 1) | (false, b">", 1)
    );
    let nonzero = !call_operator_is(node, b"&.")
        && matches!(
            (length_on_left, operator, integer),
            (true, b">", 0) | (true, b"!=", 0) | (false, b"<", 0) | (false, b"!=", 0)
        );
    if !zero && !nonzero {
        return;
    }

    let receiver = node.receiver().expect("receiver checked above");
    let receiver_source = context.source_file().node(&receiver);
    let call_operator = node
        .call_operator_loc()
        .map_or(".", |location| context.source_file().at(&location));
    let replacement = format!(
        "{}{receiver_source}{call_operator}empty?",
        if nonzero { "!" } else { "" }
    );
    let method = String::from_utf8_lossy(call_name(node));
    let operator = String::from_utf8_lossy(operator);
    let current = if length_on_left {
        format!("{method} {operator} {integer}")
    } else {
        format!("{integer} {operator} {method}")
    };
    let preferred = if nonzero { "!empty?" } else { "empty?" };
    context.replace_call(
        &parent,
        format!("Use `{preferred}` instead of `{current}`."),
        replacement,
    );
}

fn same_call(node: &Node<'_>, expected: &CallNode<'_>) -> bool {
    node.as_call_node().is_some_and(|call| {
        call.location().start_offset() == expected.location().start_offset()
            && call.location().end_offset() == expected.location().end_offset()
    })
}

fn integer_value(node: &Node<'_>) -> Option<i32> {
    TryInto::<i32>::try_into(node.as_integer_node()?.value()).ok()
}

fn non_polymorphic_collection(node: &CallNode<'_>, context: &CopContext<'_, '_>) -> bool {
    if call_name(node) != b"size" {
        return false;
    }
    let Some(receiver) = node.receiver() else {
        return false;
    };
    let source = context.source_file().node(&receiver);
    let source = source.strip_prefix("::").unwrap_or(source);
    source.starts_with("File.stat(")
        || ["File", "Tempfile", "StringIO"].iter().any(|constant| {
            source.starts_with(&format!("{constant}.new"))
                || source.starts_with(&format!("{constant}.open"))
        })
}
