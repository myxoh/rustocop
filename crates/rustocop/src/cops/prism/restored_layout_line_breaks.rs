use super::*;

define_cops! {
    FirstArrayElementLineBreak => "Layout/FirstArrayElementLineBreak" => compatibility_prism_node(as_array_node, first_array_element_line_break),
    FirstHashElementLineBreak => "Layout/FirstHashElementLineBreak" => compatibility_prism_node(as_hash_node, first_hash_element_line_break),
    FirstMethodArgumentLineBreak => "Layout/FirstMethodArgumentLineBreak" => compatibility_prism_any_node(first_method_argument_line_break),
}

fn first_array_element_line_break(
    node: &ruby_prism::ArrayNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let elements = node.elements().iter().collect::<Vec<_>>();
    let Some(first) = elements.first() else { return };
    let file = context.source_file();

    let opening_end = if let Some(opening) = node.opening_loc() {
        if !multiline_elements(file, opening.start_offset(), &elements)
            || !file.same_line(opening.start_offset(), first.location().start_offset())
        {
            return;
        }
        opening.end_offset()
    } else {
        if context.config_bool("AllowImplicitArrayLiterals", false)
            || file.same_line(node.location().start_offset(), node.location().end_offset())
            || !file
                .slice(file.line_start(first.location().start_offset())..first.location().start_offset())
                .is_some_and(|prefix| prefix.contains('='))
        {
            return;
        }
        first.location().start_offset()
    };

    if allowed_multiline_final_element(context, &elements) {
        return;
    }
    context.insert(
        "Add a line break before the first element of a multi-line array.",
        first.location(),
        opening_end,
        "\n",
    );
}

fn first_hash_element_line_break(
    node: &ruby_prism::HashNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let opening = node.opening_loc();
    let elements = node.elements().iter().collect::<Vec<_>>();
    let Some(first) = elements.first() else { return };
    let file = context.source_file();
    if !multiline_elements(file, opening.start_offset(), &elements)
        || !file.same_line(opening.start_offset(), first.location().start_offset())
        || allowed_multiline_final_element(context, &elements)
    {
        return;
    }
    context.insert(
        "Add a line break before the first element of a multi-line hash.",
        first.location(),
        first.location().start_offset(),
        "\n",
    );
}

fn first_method_argument_line_break(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (opening, closing, arguments, call_start, allowed) = if let Some(call) = node.as_call_node() {
        let (Some(opening), Some(closing)) = (call.opening_loc(), call.closing_loc()) else {
            return;
        };
        let allowed = context
            .config_values("AllowedMethods")
            .iter()
            .any(|allowed| allowed.as_bytes() == call.name().as_slice());
        if context.source_file().at(&opening) != "(" {
            return;
        }
        let mut values = call
            .arguments()
            .map_or_else(Vec::new, |arguments| arguments.arguments().iter().collect::<Vec<_>>());
        if let Some(block_argument) = call
            .block()
            .filter(|block| block.as_block_argument_node().is_some())
        {
            values.push(block_argument);
        }
        (opening, closing, values, call.location().start_offset(), allowed)
    } else if let Some(call) = node.as_super_node() {
        let (Some(opening), Some(closing), Some(arguments)) =
            (call.lparen_loc(), call.rparen_loc(), call.arguments())
        else {
            return;
        };
        let mut values = arguments.arguments().iter().collect::<Vec<_>>();
        if let Some(block_argument) = call
            .block()
            .filter(|block| block.as_block_argument_node().is_some())
        {
            values.push(block_argument);
        }
        (opening, closing, values, call.keyword_loc().start_offset(), false)
    } else {
        return;
    };
    let Some(first) = arguments.first() else { return };
    let offense = first.as_keyword_hash_node().map_or_else(
        || first.location().start_offset()..first.location().end_offset(),
        |hash| {
            let first = hash
                .elements()
                .iter()
                .next()
                .map_or_else(|| hash.location(), |element| element.location());
            let end = if arguments
                .last()
                .is_some_and(|argument| argument.as_block_argument_node().is_some())
            {
                hash.location().end_offset()
            } else {
                first.end_offset()
            };
            first.start_offset()..end
        },
    );
    let file = context.source_file();
    let first_is_heredoc = file
        .slice(opening.end_offset()..file.line_end(opening.end_offset()))
        .is_some_and(|source| source.trim_start().starts_with("<<"));
    let arguments_span_lines = arguments.iter().any(|argument| {
        !file.same_line(first.location().start_offset(), argument.location().start_offset())
            || !file.same_line(
                argument.location().start_offset(),
                argument.location().end_offset(),
            )
    });
    if file.same_line(opening.start_offset(), closing.start_offset())
        || !file.same_line(opening.start_offset(), first.location().start_offset())
        || !file.same_line(call_start, first.location().start_offset())
        || first_is_heredoc
        || !arguments_span_lines
        || allowed_multiline_final_element(context, &arguments)
        || allowed
    {
        return;
    }
    context.insert(
        "Add a line break before the first argument of a multi-line method argument list.",
        offense,
        opening.end_offset(),
        "\n",
    );
}

fn multiline_elements(file: SourceFile<'_>, opening: usize, elements: &[Node<'_>]) -> bool {
    elements.iter().any(|element| {
        let location = element.location();
        !file.same_line(opening, location.start_offset())
            || (!file.node(element).trim_start().starts_with("<<")
                && !file.same_line(location.start_offset(), location.end_offset()))
    })
}

fn allowed_multiline_final_element(
    context: &CopContext<'_, '_>,
    elements: &[Node<'_>],
) -> bool {
    context.config_bool("AllowMultilineFinalElement", false)
        && elements.last().is_some_and(|last| {
            let file = context.source_file();
            !file.same_line(last.location().start_offset(), last.location().end_offset())
                || file.node(last).trim_start().starts_with("<<")
        })
}
