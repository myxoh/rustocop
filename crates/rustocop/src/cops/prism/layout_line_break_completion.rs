use super::source_syntax::{matching_delimiter, top_level_elements};
use super::*;

define_cops! {
    MultilineHashKeyLineBreaks => "Layout/MultilineHashKeyLineBreaks" => compatibility_prism_any_node(multiline_hash_key_line_breaks),
    SingleLineBlockChain => "Layout/SingleLineBlockChain" => compatibility_prism_any_node(single_line_block_chain),
}

#[allow(dead_code)]
fn first_array_element_line_break(context: &mut CopContext<'_, '_>) {
    first_literal_element(context, b'[', b']', "array", |source, opening| {
        opening == 0
            || !matches!(source.as_bytes()[opening - 1], b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b']')
    });
    for marker in ["%w(", "%W(", "%i(", "%I("] {
        for (opening, _) in context.source().match_indices(marker) {
            let delimiter = opening + marker.len() - 1;
            report_percent_first_element(context, delimiter);
        }
    }
    implicit_array_assignment(context);
}

#[allow(dead_code)]
fn first_hash_element_line_break(context: &mut CopContext<'_, '_>) {
    first_literal_element(context, b'{', b'}', "hash", |source, opening| {
        let rest = &source[opening + 1..];
        rest.find('}').is_some_and(|end| {
            let body = &rest[..end];
            body.contains(':') || body.contains("=>")
        })
    });
}

fn first_literal_element(
    context: &mut CopContext<'_, '_>,
    opening_byte: u8,
    closing_byte: u8,
    collection: &str,
    allowed: impl Fn(&str, usize) -> bool,
) {
    let source = context.source();
    for opening in source
        .bytes()
        .enumerate()
        .filter_map(|(offset, byte)| (byte == opening_byte).then_some(offset))
        .collect::<Vec<_>>()
    {
        if allowed(source, opening) {
            report_first_element(context, opening, opening_byte, closing_byte, collection);
        }
    }
}

fn report_first_element(
    context: &mut CopContext<'_, '_>,
    opening: usize,
    opening_byte: u8,
    closing_byte: u8,
    collection: &str,
) {
    let source = context.source();
    let Some(closing) = matching_delimiter(source, opening, opening_byte, closing_byte) else {
        return;
    };
    if !source[opening..=closing].contains('\n') || source[opening + 1..].starts_with('\n') {
        return;
    }
    let elements = top_level_elements(source, opening + 1, closing);
    let Some(first) = elements.first() else {
        return;
    };
    if context.config_bool("AllowMultilineFinalElement", false)
        && elements
            .last()
            .is_some_and(|last| source[last.clone()].contains('\n'))
    {
        return;
    }
    let start =
        first.start + source[first.clone()].len() - source[first.clone()].trim_start().len();
    if source[opening + 1..start].contains('\n') {
        return;
    }
    let end = first.end - (source[first.clone()].len() - source[first.clone()].trim_end().len());
    context.insert(
        format!("Add a line break before the first element of a multi-line {collection}."),
        start..end.max(start),
        start,
        "\n",
    );
}

fn report_percent_first_element(context: &mut CopContext<'_, '_>, opening: usize) {
    let source = context.source();
    let Some(closing) = matching_delimiter(source, opening, b'(', b')') else {
        return;
    };
    if !source[opening..=closing].contains('\n') || source[opening + 1..].starts_with('\n') {
        return;
    }
    context.insert(
        "Add a line break before the first element of a multi-line array.",
        opening + 1..opening + 1,
        opening + 1,
        "\n",
    );
}

fn implicit_array_assignment(context: &mut CopContext<'_, '_>) {
    if context.config_bool("AllowImplicitArrayLiterals", false) {
        return;
    }
    let source = context.source();
    for (line_start, line) in context.source_file().lines() {
        let Some(equal) = line.find("= ") else {
            continue;
        };
        let after = line_start + equal + 2;
        if line[equal + 2..].trim_start().starts_with('[')
            || !source[after..].contains("\n")
            || !line[equal + 2..].contains(',')
        {
            continue;
        }
        let offense = after..after;
        context.insert(
            "Add a line break before the first element of a multi-line array.",
            offense.clone(),
            offense.start,
            "\n",
        );
    }
}

#[allow(dead_code)]
fn first_method_argument_line_break(context: &mut CopContext<'_, '_>) {
    first_parenthesized_list(context, false);
}

fn first_parenthesized_list(context: &mut CopContext<'_, '_>, definition: bool) {
    let source = context.source();
    for opening in source
        .bytes()
        .enumerate()
        .filter_map(|(offset, byte)| (byte == b'(').then_some(offset))
        .collect::<Vec<_>>()
    {
        let line_start = source[..opening].rfind('\n').map_or(0, |at| at + 1);
        let prefix = source[line_start..opening].trim_start();
        if prefix.starts_with("def ") != definition {
            continue;
        }
        if !definition
            && (prefix.is_empty()
                || ["if", "unless", "while", "until"]
                    .iter()
                    .any(|word| prefix.ends_with(word)))
        {
            continue;
        }
        if !definition {
            let method = prefix
                .split(|character: char| {
                    !(character.is_alphanumeric() || matches!(character, '_' | '!' | '?'))
                })
                .next_back()
                .unwrap_or_default();
            if context
                .config_values("AllowedMethods")
                .iter()
                .any(|allowed| allowed == method)
            {
                continue;
            }
        }
        let Some(closing) = matching_delimiter(source, opening, b'(', b')') else {
            continue;
        };
        if !source[opening..=closing].contains('\n') || source[opening + 1..].starts_with('\n') {
            continue;
        }
        let first_line = source[opening + 1..closing]
            .split_once('\n')
            .map_or(&source[opening + 1..closing], |(line, _)| line);
        if first_line.trim_start().starts_with('#') {
            continue;
        }
        let Some(first) = top_level_elements(source, opening + 1, closing)
            .first()
            .cloned()
        else {
            continue;
        };
        let elements = top_level_elements(source, opening + 1, closing);
        if context.config_bool("AllowMultilineFinalElement", false)
            && elements
                .last()
                .is_some_and(|last| source[last.clone()].contains('\n'))
        {
            continue;
        }
        let start = leading_code_offset(source, first.start, first.end);
        if source[opening + 1..start].contains('\n') {
            continue;
        }
        let end =
            first.end - (source[first.clone()].len() - source[first.clone()].trim_end().len());
        let kind = if definition { "parameter" } else { "argument" };
        let list = if definition {
            "method parameter list"
        } else {
            "method argument list"
        };
        context.insert(
            format!("Add a line break before the first {kind} of a multi-line {list}."),
            start..end.max(start),
            start,
            "\n",
        );
    }
}

fn leading_code_offset(source: &str, mut start: usize, end: usize) -> usize {
    while start < end {
        let whitespace = source[start..end]
            .len()
            - source[start..end].trim_start().len();
        start += whitespace;
        if source.as_bytes().get(start) != Some(&b'#') {
            break;
        }
        start = source[start..end]
            .find('\n')
            .map_or(end, |newline| start + newline + 1);
    }
    start
}

fn multiline_hash_key_line_breaks(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let Some(hash) = node.as_hash_node() else {
        return;
    };
    let location = hash.location();
    let elements = hash.elements().iter().collect::<Vec<_>>();
    let file = context.source_file();
    if file.same_line(location.start_offset(), location.end_offset()) {
        return;
    }
    let Some(first) = elements.first() else { return };
    let Some(last) = elements.last() else { return };
    let allow_multiline_final = context.config_bool("AllowMultilineFinalElement", false);
    let first_line = file.line_start(first.location().start_offset());
    let last_line = if allow_multiline_final {
        file.line_start(last.location().start_offset())
    } else {
        file.line_start(last.location().end_offset().saturating_sub(1))
    };
    if first_line == last_line {
        return;
    }
    let mut last_seen_line = None;
    let element_count = elements.len();
    for (index, element) in elements.iter().enumerate() {
        let element_location = element.location();
        let element_first_line = file.line_start(element_location.start_offset());
        if last_seen_line.is_some_and(|line| line >= element_first_line) {
            let message = "Each key in a multi-line hash must start on a separate line.";
            if allow_multiline_final
                && index + 2 == element_count
                && element_first_line
                    != file.line_start(element_location.end_offset().saturating_sub(1))
            {
                let final_start = elements[index + 1].location().start_offset();
                context.replace_many(
                    message,
                    &element_location,
                    vec![
                        (
                            element_location.start_offset()..element_location.start_offset(),
                            "\n".to_string(),
                        ),
                        (final_start..final_start, "\n".to_string()),
                    ],
                );
            } else {
                context.insert(
                    message,
                    &element_location,
                    element_location.start_offset(),
                    "\n",
                );
            }
        } else {
            last_seen_line = Some(file.line_start(
                element_location.end_offset().saturating_sub(1),
            ));
        }
    }
}

fn single_line_block_chain(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let Some(call) = node.as_call_node() else {
        return;
    };
    let Some(operator) = call.call_operator_loc() else {
        return;
    };
    let Some(receiver) = call.receiver() else {
        return;
    };
    let block_location = if let Some(receiver) = receiver.as_call_node() {
        let Some(block) = receiver.block().and_then(|block| block.as_block_node()) else {
            return;
        };
        block.location()
    } else if let Some(lambda) = receiver.as_lambda_node() {
        lambda.location()
    } else {
        return;
    };
    let file = context.source_file();
    if !file.same_line(block_location.start_offset(), block_location.end_offset())
        || !file.same_line(
            block_location.end_offset().saturating_sub(1),
            operator.start_offset(),
        )
    {
        return;
    }
    let end = call
        .message_loc()
        .map_or_else(|| call.opening_loc().map(|loc| loc.end_offset()), |loc| Some(loc.end_offset()));
    let Some(end) = end else {
        return;
    };
    if !file.same_line(operator.start_offset(), end.saturating_sub(1)) {
        return;
    }
    context.insert(
        "Put method call on a separate line if chained to a single line block.",
        operator.start_offset()..end,
        operator.start_offset(),
        "\n",
    );
}

fn condition_position(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (keyword, predicate) = if let Some(condition) = node.as_if_node() {
        let Some(keyword) = condition.if_keyword_loc() else {
            return;
        };
        (keyword, condition.predicate())
    } else if let Some(condition) = node.as_unless_node() {
        (condition.keyword_loc(), condition.predicate())
    } else if let Some(condition) = node.as_while_node() {
        (condition.keyword_loc(), condition.predicate())
    } else if let Some(condition) = node.as_until_node() {
        (condition.keyword_loc(), condition.predicate())
    } else {
        return;
    };
    let predicate_location = predicate.location();
    if keyword.start_offset() != node.location().start_offset()
        || context
            .source_file()
            .same_line(keyword.start_offset(), predicate_location.start_offset())
    {
        return;
    }

    let keyword_source = context.source_file().at(&keyword);
    let predicate_source = context.source_file().node(&predicate);
    let removal = context.source_file().full_line_range(
        predicate_location.start_offset()..predicate_location.end_offset(),
    );
    context.replace_many(
        format!("Place the condition on the same line as `{keyword_source}`."),
        &predicate_location,
        vec![
            (
                keyword.end_offset()..keyword.end_offset(),
                format!(" {predicate_source}"),
            ),
            (removal, String::new()),
        ],
    );
}
