use super::*;

define_cops! {
    TrailingCommaInArguments => "Style/TrailingCommaInArguments" => node(as_call_node, trailing_comma_in_arguments),
}

fn trailing_comma_in_arguments(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let (Some(opening), Some(closing)) = (node.opening_loc(), node.closing_loc()) else {
        return;
    };
    if opening.as_slice() != b"(" && !(call_name(node) == b"[]" && opening.as_slice() == b"[") {
        return;
    }
    let Some(arguments) = node.arguments() else {
        return;
    };
    let raw_items = arguments.arguments().iter().collect::<Vec<_>>();
    let block_argument = node.block().filter(|block| block.as_block_argument_node().is_some());
    let last_item = block_argument.as_ref().or_else(|| raw_items.last());
    let Some(last_item) = last_item else {
        return;
    };

    let mut elements = flattened_elements(&raw_items, context.source_file());
    if let Some(block_argument) = block_argument.as_ref() {
        let location = block_argument.location();
        elements.push((location.start_offset(), location.end_offset()));
    }
    let last_location = last_item.location();
    let last_start = last_location.start_offset();
    let last_end = last_location.end_offset();
    let close = closing.start_offset();
    if last_end > close {
        return;
    }
    let tail = &context.source()[last_end..close];
    let heredoc = raw_items.iter().any(contains_heredoc);
    let comma_offset = leading_comma_offset(tail, heredoc);
    let comma = comma_offset.map(|offset| last_end + offset);
    let style = context
        .config_value("EnforcedStyleForMultiline")
        .unwrap_or("no_comma");
    let multiline = call_is_multiline(node, &elements, &closing, context.source_file());
    let required = match style {
        "comma" => multiline && elements_on_separate_lines(&elements, close, context.source_file()),
        "consistent_comma" => {
            multiline
                && !method_name_and_arguments_on_same_line(
                    node,
                    &last_location,
                    last_item.as_hash_node().is_some(),
                    &closing,
                    context.source_file(),
                )
        }
        "diff_comma" => {
            let after_item = comma_offset.map_or(tail, |offset| &tail[offset + 1..]);
            let rest_of_line = after_item.split('\n').next().unwrap_or(after_item).trim();
            multiline
                && after_item.contains('\n')
                && (rest_of_line.is_empty() || rest_of_line.starts_with('#'))
        }
        _ => false,
    };

    if let Some(comma) = comma.filter(|_| !required) {
        let extra = match style {
            "comma" => ", unless each item is on its own line",
            "consistent_comma" => ", unless items are split onto multiple lines",
            "diff_comma" => ", unless that item immediately precedes a newline",
            _ => "",
        };
        context.remove(
            format!("Avoid comma after the last parameter of a method call{extra}."),
            comma..comma + 1,
            comma..comma + 1,
        );
    } else if comma.is_none() && required && block_argument.is_none() {
        let item_source = &context.source()[last_start..last_end];
        let final_line = item_source.rfind('\n').map_or(0, |offset| offset + 1);
        let offense_start = last_start
            + final_line
            + item_source[final_line..]
                .bytes()
                .take_while(|byte| byte.is_ascii_whitespace())
                .count();
        context.insert(
            "Put a comma after the last parameter of a multiline method call.",
            offense_start..last_end,
            last_end,
            ",",
        );
    }
}

fn flattened_elements<'pr>(items: &[Node<'pr>], file: SourceFile<'_>) -> Vec<(usize, usize)> {
    let mut elements = Vec::new();
    for item in items {
        if let Some(hash) = item.as_keyword_hash_node().filter(|hash| {
            let location = hash.location();
            !file.same_line(
                location.start_offset(),
                location.end_offset().saturating_sub(1),
            )
        }) {
            elements.extend(hash.elements().iter().map(|element| {
                let location = element.location();
                (location.start_offset(), location.end_offset())
            }));
        } else {
            let location = item.location();
            elements.push((location.start_offset(), location.end_offset()));
        }
    }
    elements
}

fn call_is_multiline(
    node: &CallNode<'_>,
    elements: &[(usize, usize)],
    closing: &ruby_prism::Location<'_>,
    file: SourceFile<'_>,
) -> bool {
    let location = node.location();
    let syntactically_multiline = !file.same_line(
        location.start_offset(),
        closing.end_offset().saturating_sub(1),
    );
    let closing_begins_line = file.as_str()[file.line_start(closing.start_offset())..closing.start_offset()]
        .trim()
        .is_empty();
    syntactically_multiline && !(elements.len() == 1 && !closing_begins_line)
}

fn elements_on_separate_lines(
    elements: &[(usize, usize)],
    close: usize,
    file: SourceFile<'_>,
) -> bool {
    elements.windows(2).all(|pair| {
        !file.same_line(pair[0].1.saturating_sub(1), pair[1].0)
    }) && elements
        .last()
        .is_some_and(|last| !file.same_line(last.1.saturating_sub(1), close))
}

fn method_name_and_arguments_on_same_line(
    node: &CallNode<'_>,
    last_item: &ruby_prism::Location<'_>,
    last_item_is_hash: bool,
    closing: &ruby_prism::Location<'_>,
    file: SourceFile<'_>,
) -> bool {
    let last_line_offset = last_item.end_offset().saturating_sub(1);
    if !file.same_line(closing.start_offset(), last_line_offset) {
        return false;
    }
    if last_item_is_hash {
        return true;
    }
    let selector = node
        .message_loc()
        .map_or(node.location().start_offset(), |location| location.start_offset());
    file.same_line(selector, last_line_offset)
}

fn leading_comma_offset(source: &str, heredoc: bool) -> Option<usize> {
    let mut offset = 0;
    for byte in source.bytes() {
        if byte == b',' {
            return Some(offset);
        }
        let whitespace = if heredoc {
            matches!(byte, b' ' | b'\t' | b'\r')
        } else {
            byte.is_ascii_whitespace()
        };
        if !whitespace {
            return None;
        }
        offset += 1;
    }
    None
}

fn contains_heredoc(node: &Node<'_>) -> bool {
    let mut finder = HeredocFinder(false);
    finder.visit(node);
    finder.0
}

struct HeredocFinder(bool);

impl<'pr> Visit<'pr> for HeredocFinder {
    fn visit_string_node(&mut self, node: &ruby_prism::StringNode<'pr>) {
        if node
            .opening_loc()
            .is_some_and(|opening| opening.as_slice().starts_with(b"<<"))
        {
            self.0 = true;
        }
    }

    fn visit_interpolated_string_node(&mut self, node: &ruby_prism::InterpolatedStringNode<'pr>) {
        if node
            .opening_loc()
            .is_some_and(|opening| opening.as_slice().starts_with(b"<<"))
        {
            self.0 = true;
        }
        ruby_prism::visit_interpolated_string_node(self, node);
    }
}
