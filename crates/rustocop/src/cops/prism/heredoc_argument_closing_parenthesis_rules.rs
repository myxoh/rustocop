use super::*;

define_cops! {
    HeredocArgumentClosingParenthesis => "Layout/HeredocArgumentClosingParenthesis" => call(on_send),
}

fn on_send(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let (Some(_opening), Some(closing), Some(arguments)) =
        (node.opening_loc(), node.closing_loc(), node.arguments())
    else {
        return;
    };
    let Some(heredoc) = arguments
        .arguments()
        .iter()
        .filter_map(|argument| heredoc_locations(&argument))
        .last()
    else {
        return;
    };
    let file = context.source_file();
    if file.same_line(heredoc.opening.start_offset(), closing.start_offset())
        || context
            .ancestors()
            .iter()
            .any(|ancestor| ancestor_has_end_keyword(ancestor, file))
    {
        return;
    }
    if heredoc.closing.end_offset() < closing.start_offset() {
        let between = &context.source()[heredoc.closing.end_offset()..closing.start_offset()];
        let nested_call_followup = arguments
            .arguments()
            .iter()
            .any(|argument| {
                argument.as_call_node().is_some()
                    && argument.location().start_offset() < heredoc.opening.start_offset()
                    && context.source()
                        [argument.location().start_offset()..heredoc.opening.start_offset()]
                        .contains('(')
            });
        if !between.trim().is_empty()
            && (!nested_call_followup
                || between
                    .lines()
                    .any(|line| line.trim_start().starts_with(')')))
        {
            return;
        }
    }

    let message = "Put the closing parenthesis for a method call with a HEREDOC parameter on the same line as the HEREDOC opening.";
    let last_argument = arguments.arguments().last().expect("nonempty arguments");
    let last_argument_end = last_argument.location().end_offset();
    let closing_line = file.line(closing.start_offset());
    let trimmed_closing_line = closing_line.trim();
    let safe_to_remove_closing_line = trimmed_closing_line == ")"
        || trimmed_closing_line
            .strip_prefix(')')
            .and_then(|suffix| suffix.strip_suffix(','))
            .is_some_and(|spaces| spaces.len() <= 20 && spaces.bytes().all(|byte| byte == b' '));
    let removal_end = closing.end_offset();
    let removal = if safe_to_remove_closing_line {
        file.line_start(closing.start_offset()).saturating_sub(1)..removal_end
    } else {
        closing.start_offset()..removal_end
    };
    let mut edits = vec![
        (last_argument_end..last_argument_end, ")".to_string()),
        (removal, String::new()),
    ];

    let between_argument_and_closing = &context.source()[last_argument_end..closing.start_offset()];
    if let (Some(comma), Some(newline)) = (
        between_argument_and_closing.find(','),
        between_argument_and_closing.find('\n'),
    ) {
        if comma < newline {
            edits.push((
                last_argument_end..last_argument_end + comma + 1,
                String::new(),
            ));
        }
    }

    let node_end = node.location().end_offset();
    let mut external_comma_offset = 0;
    while external_comma_offset < 20
        && context
            .source()
            .as_bytes()
            .get(node_end + external_comma_offset)
            == Some(&b' ')
    {
        external_comma_offset += 1;
    }
    if context
        .source()
        .as_bytes()
        .get(node_end + external_comma_offset)
        == Some(&b',')
    {
        edits.push((
            node_end..node_end + external_comma_offset + 1,
            String::new(),
        ));
        edits.push((last_argument_end..last_argument_end, ",".to_string()));
    }

    context.replace_many(message, &closing, edits);
}

struct HeredocLocations<'pr> {
    opening: ruby_prism::Location<'pr>,
    closing: ruby_prism::Location<'pr>,
}

fn heredoc_locations<'pr>(node: &Node<'pr>) -> Option<HeredocLocations<'pr>> {
    struct Finder<'pr>(Option<HeredocLocations<'pr>>);

    impl<'pr> Visit<'pr> for Finder<'pr> {
        fn visit_string_node(&mut self, node: &ruby_prism::StringNode<'pr>) {
            if let (Some(opening), Some(closing)) = (node.opening_loc(), node.closing_loc()) {
                if opening.as_slice().starts_with(b"<<") {
                    self.0 = Some(HeredocLocations { opening, closing });
                    return;
                }
            }
            ruby_prism::visit_string_node(self, node);
        }

        fn visit_interpolated_string_node(
            &mut self,
            node: &ruby_prism::InterpolatedStringNode<'pr>,
        ) {
            if let (Some(opening), Some(closing)) = (node.opening_loc(), node.closing_loc()) {
                if opening.as_slice().starts_with(b"<<") {
                    self.0 = Some(HeredocLocations { opening, closing });
                    return;
                }
            }
            ruby_prism::visit_interpolated_string_node(self, node);
        }
    }

    let mut finder = Finder(None);
    finder.visit(node);
    finder.0
}

fn ancestor_has_end_keyword(node: &Node<'_>, file: SourceFile<'_>) -> bool {
    if node.as_program_node().is_some() || node.as_statements_node().is_some() {
        return false;
    }
    let source = file.node(node).trim_end();
    source == "end"
        || source.strip_suffix("end").is_some_and(|prefix| {
            prefix
                .as_bytes()
                .last()
                .is_some_and(|byte| byte.is_ascii_whitespace())
        })
}
