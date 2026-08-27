use super::*;

define_cops! {
    TrailingCommaInArguments => "Style/TrailingCommaInArguments" => compatibility_prism_node(as_call_node, trailing_comma_in_arguments),
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
    let block_argument = node
        .block()
        .filter(|block| block.as_block_argument_node().is_some());
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
    let required = compatibility_requires_trailing_comma(
        style,
        node,
        &elements,
        &closing,
        last_item.as_hash_node().is_some(),
        context,
    );

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

fn compatibility_requires_trailing_comma(
    style: &str,
    node: &CallNode<'_>,
    elements: &[(usize, usize)],
    closing: &ruby_prism::Location<'_>,
    last_item_is_hash: bool,
    context: &CopContext<'_, '_>,
) -> bool {
    use crate::rubocop::cop::mixin::trailing_comma::{
        Item, Location, TrailingComma, TrailingCommaStyle,
    };

    fn line_number(context: &CopContext<'_, '_>, offset: usize) -> usize {
        context.line_index(offset) + 1
    }

    fn item_location(context: &CopContext<'_, '_>, range: std::ops::Range<usize>) -> Location {
        let source = context.source();
        Location {
            line: line_number(context, range.start),
            last_line: line_number(context, range.end.saturating_sub(1)),
            source: source.get(range.clone()).unwrap_or_default().to_string(),
            begins_its_line: source
                [source[..range.start].rfind('\n').map_or(0, |at| at + 1)..range.start]
                .trim()
                .is_empty(),
            bytes: range,
        }
    }

    fn item(context: &CopContext<'_, '_>, start: usize, end: usize, braces: bool) -> Item {
        let source = context.source();
        Item {
            kind: if braces { "hash" } else { "argument" }.to_string(),
            source_range: item_location(context, start..end),
            children: Vec::new(),
            arguments: Vec::new(),
            call_type: false,
            multiline: source[start..end].contains('\n'),
            braces,
            block_pass: false,
            heredoc_body: false,
            end_location: None,
            selector_line: None,
        }
    }

    let source = context.source();

    let mut arguments = elements
        .iter()
        .enumerate()
        .map(|(index, &(start, end))| {
            item(
                context,
                start,
                end,
                last_item_is_hash && index + 1 == elements.len(),
            )
        })
        .collect::<Vec<_>>();
    let children = arguments.clone();
    let location = node.location();
    let end_location = item_location(context, closing.start_offset()..closing.end_offset());
    let selector_line = node
        .message_loc()
        .map(|selector| line_number(context, selector.start_offset()));
    let node = Item {
        kind: "send".to_string(),
        source_range: item_location(context, location.start_offset()..location.end_offset()),
        children,
        arguments: std::mem::take(&mut arguments),
        call_type: true,
        multiline: source[location.start_offset()..closing.end_offset()].contains('\n'),
        braces: false,
        block_pass: false,
        heredoc_body: false,
        end_location: Some(end_location),
        selector_line,
    };
    let style = match style {
        "comma" => TrailingCommaStyle::Comma,
        "consistent_comma" => TrailingCommaStyle::ConsistentComma,
        "diff_comma" => TrailingCommaStyle::DiffComma,
        _ => TrailingCommaStyle::NoComma,
    };
    TrailingComma { style }.should_have_comma(style, &node)
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

fn leading_comma_offset(source: &str, heredoc: bool) -> Option<usize> {
    for (offset, byte) in source.bytes().enumerate() {
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
