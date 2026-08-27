use super::*;

mod block_layout;
use block_layout::multiline_block_layout;

define_cops! {
    MultilineArrayBraceLayout => "Layout/MultilineArrayBraceLayout" => compatibility_prism_node(as_array_node, multiline_array_brace_layout),
    MultilineHashBraceLayout => "Layout/MultilineHashBraceLayout" => compatibility_prism_node(as_hash_node, multiline_hash_brace_layout),
    MultilineMethodCallBraceLayout => "Layout/MultilineMethodCallBraceLayout" => compatibility_prism_node(as_call_node, multiline_method_call_brace_layout),
    MultilineBlockLayout => "Layout/MultilineBlockLayout" => compatibility_prism_any_node(multiline_block_layout),
}

#[derive(Clone, Copy)]
enum DelimiterKind {
    Array,
    Hash,
    MethodCall,
}

impl DelimiterKind {
    fn same_line_message(self) -> &'static str {
        match self {
            Self::Array => "The closing array brace must be on the same line as the last array element when the opening brace is on the same line as the first array element.",
            Self::Hash => "Closing hash brace must be on the same line as the last hash element when opening brace is on the same line as the first hash element.",
            Self::MethodCall => "Closing method call brace must be on the same line as the last argument when opening brace is on the same line as the first argument.",
        }
    }

    fn new_line_message(self) -> &'static str {
        match self {
            Self::Array => "The closing array brace must be on the line after the last array element when the opening brace is on a separate line from the first array element.",
            Self::Hash => "Closing hash brace must be on the line after the last hash element when opening brace is on a separate line from the first hash element.",
            Self::MethodCall => "Closing method call brace must be on the line after the last argument when opening brace is on a separate line from the first argument.",
        }
    }

    fn always_same_line_message(self) -> &'static str {
        match self {
            Self::Array => {
                "The closing array brace must be on the same line as the last array element."
            }
            Self::Hash => "Closing hash brace must be on the same line as the last hash element.",
            Self::MethodCall => {
                "Closing method call brace must be on the same line as the last argument."
            }
        }
    }

    fn always_new_line_message(self) -> &'static str {
        match self {
            Self::Array => {
                "The closing array brace must be on the line after the last array element."
            }
            Self::Hash => "Closing hash brace must be on the line after the last hash element.",
            Self::MethodCall => {
                "Closing method call brace must be on the line after the last argument."
            }
        }
    }
}

fn multiline_array_brace_layout(
    node: &ruby_prism::ArrayNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let (Some(opening), Some(closing)) = (node.opening_loc(), node.closing_loc()) else {
        return;
    };
    let elements = node.elements().iter().collect::<Vec<_>>();
    check_delimiter_layout(
        context,
        opening,
        closing,
        &elements,
        node.location().start_offset()..node.location().end_offset(),
        DelimiterKind::Array,
    );
}

fn multiline_hash_brace_layout(node: &ruby_prism::HashNode<'_>, context: &mut CopContext<'_, '_>) {
    let opening = node.opening_loc();
    let closing = node.closing_loc();
    let elements = node.elements().iter().collect::<Vec<_>>();
    check_delimiter_layout(
        context,
        opening,
        closing,
        &elements,
        node.location().start_offset()..node.location().end_offset(),
        DelimiterKind::Hash,
    );
}

fn multiline_method_call_brace_layout(
    node: &ruby_prism::CallNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let (Some(opening), Some(closing), Some(arguments)) =
        (node.opening_loc(), node.closing_loc(), node.arguments())
    else {
        return;
    };
    if context.source_file().at(&opening) != "(" {
        return;
    }
    let mut elements = arguments.arguments().iter().collect::<Vec<_>>();
    if let Some(block_argument) = node
        .block()
        .filter(|block| block.as_block_argument_node().is_some())
    {
        elements.push(block_argument);
    }
    let container = opening.start_offset()..closing.end_offset();
    check_delimiter_layout(
        context,
        opening,
        closing,
        &elements,
        container,
        DelimiterKind::MethodCall,
    );
}

fn check_delimiter_layout(
    context: &mut CopContext<'_, '_>,
    opening: ruby_prism::Location<'_>,
    closing: ruby_prism::Location<'_>,
    elements: &[Node<'_>],
    container: std::ops::Range<usize>,
    kind: DelimiterKind,
) {
    let Some(first) = elements.first() else {
        return;
    };
    let Some(last) = elements.last() else { return };
    let file = context.source_file();
    if file.same_line(container.start, container.end)
        || kind_is_single_line_call(kind, file, &opening, &closing)
        || unsafe_last_line_heredoc(file, last)
    {
        return;
    }

    let opening_with_first =
        file.same_line(opening.start_offset(), first.location().start_offset());
    let closing_with_last = file.same_line(closing.start_offset(), last.location().end_offset());
    let style = context.policy().enforced_style("symmetrical");
    let wants_same_line = match style {
        "same_line" => true,
        "new_line" => false,
        _ => opening_with_first,
    };
    if wants_same_line == closing_with_last {
        return;
    }
    let message = match (style, wants_same_line) {
        ("same_line", _) => kind.always_same_line_message(),
        ("new_line", _) => kind.always_new_line_message(),
        (_, true) => kind.same_line_message(),
        (_, false) => kind.new_line_message(),
    };
    if wants_same_line {
        correct_closing_to_same_line(context, &closing, last, elements, kind, message);
    } else {
        context.insert(message, &closing, closing.start_offset(), "\n");
    }
}

fn kind_is_single_line_call(
    kind: DelimiterKind,
    file: SourceFile<'_>,
    opening: &ruby_prism::Location<'_>,
    closing: &ruby_prism::Location<'_>,
) -> bool {
    matches!(kind, DelimiterKind::MethodCall)
        && file.same_line(opening.start_offset(), closing.start_offset())
}

fn unsafe_last_line_heredoc(file: SourceFile<'_>, last: &Node<'_>) -> bool {
    let location = last.location();
    node_heredoc_ranges(last).iter().any(|range| {
        location.start_offset() <= range.start
            && range.start < location.end_offset()
            && file.line_start(range.end.saturating_sub(1))
                >= file.line_start(location.end_offset())
    })
}

fn node_heredoc_ranges(node: &Node<'_>) -> Vec<std::ops::Range<usize>> {
    #[derive(Default)]
    struct Heredocs(Vec<std::ops::Range<usize>>);

    impl Heredocs {
        fn push(
            &mut self,
            opening: Option<ruby_prism::Location<'_>>,
            closing: Option<ruby_prism::Location<'_>>,
        ) {
            let (Some(opening), Some(closing)) = (opening, closing) else {
                return;
            };
            if opening.as_slice().starts_with(b"<<") {
                self.0.push(opening.start_offset()..closing.end_offset());
            }
        }
    }

    impl<'pr> ruby_prism::Visit<'pr> for Heredocs {
        fn visit_string_node(&mut self, node: &ruby_prism::StringNode<'pr>) {
            self.push(node.opening_loc(), node.closing_loc());
        }

        fn visit_interpolated_string_node(
            &mut self,
            node: &ruby_prism::InterpolatedStringNode<'pr>,
        ) {
            self.push(node.opening_loc(), node.closing_loc());
            ruby_prism::visit_interpolated_string_node(self, node);
        }
    }

    let mut heredocs = Heredocs::default();
    heredocs.visit(node);
    heredocs.0
}

fn correct_closing_to_same_line(
    context: &mut CopContext<'_, '_>,
    closing: &ruby_prism::Location<'_>,
    last: &Node<'_>,
    elements: &[Node<'_>],
    kind: DelimiterKind,
    message: &str,
) {
    let file = context.source_file();
    let last_end = trailing_comma_end(file, last.location().end_offset(), closing.start_offset());
    let last_line_end = file.line_end(last.location().end_offset());
    let comment_after_last = file
        .slice(last.location().end_offset()..last_line_end)
        .is_some_and(|tail| tail.contains('#'));
    let sensitive_comment = comment_after_last
        && (delimiter_is_argument(context, elements)
            || delimiter_is_chained(context)
            || closing_has_chain(file, closing.end_offset()));
    if sensitive_comment {
        context.report(message, closing);
        return;
    }
    let removal_start = file.line_start(closing.start_offset()).saturating_sub(1);
    let mut removal_end = closing.end_offset();
    let mut closing_text = file.at(closing).to_string();
    if matches!(kind, DelimiterKind::MethodCall) {
        while file.as_str().as_bytes().get(removal_end) == Some(&b')') {
            removal_end += 1;
            closing_text.push(')');
        }
    }
    if file.as_str().as_bytes().get(removal_end) == Some(&b',') {
        removal_end += 1;
        closing_text.push(',');
    }
    if matches!(kind, DelimiterKind::MethodCall)
        && elements
            .iter()
            .any(|element| !node_heredoc_ranges(element).is_empty())
    {
        let suffix = file
            .slice(closing.end_offset()..file.line_end(closing.end_offset()))
            .unwrap_or_default();
        if suffix.starts_with(['.', '&']) {
            closing_text.push_str(suffix);
            removal_end = file.line_end(closing.end_offset());
        }
    }
    context.add_offense(closing, message, |corrector| {
        corrector.remove(removal_start..removal_end);
        corrector.replace(last_end..last_end, closing_text);
    });
}

fn trailing_comma_end(file: SourceFile<'_>, start: usize, closing: usize) -> usize {
    let Some(between) = file.slice(start..closing) else {
        return start;
    };
    let whitespace = between.bytes().take_while(u8::is_ascii_whitespace).count();
    if between.as_bytes().get(whitespace) == Some(&b',') {
        start + whitespace + 1
    } else {
        start
    }
}

fn closing_has_chain(file: SourceFile<'_>, closing_end: usize) -> bool {
    file.slice(closing_end..file.line_end(closing_end))
        .is_some_and(|suffix| suffix.trim_start().starts_with(['.', '&']))
}

fn delimiter_is_argument(context: &CopContext<'_, '_>, _elements: &[Node<'_>]) -> bool {
    context.ancestors().iter().any(|ancestor| {
        ancestor.as_arguments_node().is_some() || ancestor.as_keyword_hash_node().is_some()
    })
}

fn delimiter_is_chained(context: &CopContext<'_, '_>) -> bool {
    context
        .parent()
        .is_some_and(|parent| parent.as_call_node().is_some())
}
