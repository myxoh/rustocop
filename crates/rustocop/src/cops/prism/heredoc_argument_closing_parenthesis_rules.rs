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
    if heredoc.closing.end_offset() < closing.start_offset()
        && !context.source()[heredoc.closing.end_offset()..closing.start_offset()]
            .trim()
            .is_empty()
    {
        return;
    }

    let last_argument = arguments.arguments().last().expect("nonempty arguments");
    let removal = if file.line(closing.start_offset()).trim() == ")" {
        file.full_line_range(closing.start_offset()..closing.end_offset())
    } else {
        closing.start_offset()..closing.end_offset()
    };
    context.replace_many(
        "Put the closing parenthesis for a method call with a HEREDOC parameter on the same line as the HEREDOC opening.",
        &closing,
        vec![
            (
                last_argument.location().end_offset()..last_argument.location().end_offset(),
                ")".to_string(),
            ),
            (removal, String::new()),
        ],
    );
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
