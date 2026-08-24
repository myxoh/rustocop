use super::*;

define_rule!(BlockEndNewlineRule);

define_cops! {
    MultilineMethodParameterLineBreaks => "Layout/MultilineMethodParameterLineBreaks" => node(as_def_node, parameter_line_breaks),
    SpaceBeforeBlockBraces => "Layout/SpaceBeforeBlockBraces" => any_node(space_before_block_braces),
    BlockEndNewline => "Layout/BlockEndNewline" => node_rule_aliases(
        BlockEndNewlineRule,
        on_block => [as_block_node, as_lambda_node]
    ),
    DefEndAlignment => "Layout/DefEndAlignment" => node(as_def_node, def_end_alignment),
    MultilineMethodArgumentLineBreaks => "Layout/MultilineMethodArgumentLineBreaks" => node(as_call_node, argument_line_breaks),
    ParameterAlignment => "Layout/ParameterAlignment" => node(as_def_node, parameter_alignment),
}

fn parameter_line_breaks(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(parameters) = node.parameters() else {
        return;
    };
    check_multiline_element_line_breaks(
        &definition_parameter_nodes(&parameters),
        context.config_bool("AllowMultilineFinalElement", false),
        "Each parameter in a multi-line method definition must start on a separate line.",
        context,
    );
}

fn argument_line_breaks(node: &ruby_prism::CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.name().as_slice() == b"[]=" {
        return;
    }
    let Some(arguments) = node.arguments() else {
        return;
    };
    let mut elements = arguments.arguments().iter().collect::<Vec<_>>();
    let block_argument = node
        .block()
        .filter(|block| block.as_block_argument_node().is_some());
    if block_argument.is_none() {
        if let Some(keyword_hash) = elements.last().and_then(Node::as_keyword_hash_node) {
            elements.pop();
            elements.extend(keyword_hash.elements().iter());
        }
    }
    elements.extend(block_argument);
    check_multiline_element_line_breaks(
        &elements,
        context.config_bool("AllowMultilineFinalElement", false),
        "Each argument in a multi-line method call must start on a separate line.",
        context,
    );
}

fn check_multiline_element_line_breaks(
    elements: &[Node<'_>],
    allow_multiline_final_element: bool,
    message: &str,
    context: &mut CopContext<'_, '_>,
) {
    let Some((first, last)) = elements.first().zip(elements.last()) else {
        return;
    };
    let file = context.source_file();
    let first_line = file.line_start(first.location().start_offset());
    let last_location = last.location();
    let last_line = file.line_start(if allow_multiline_final_element {
        last_location.start_offset()
    } else {
        last_location.end_offset().saturating_sub(1)
    });
    if first_line == last_line {
        return;
    }

    let mut last_seen_line = None;
    for (index, element) in elements.iter().enumerate() {
        let location = element.location();
        let first_line = file.line_start(location.start_offset());
        if last_seen_line.is_some_and(|line| line >= first_line) {
            let mut edits = vec![(
                location.start_offset()..location.start_offset(),
                "\n".to_string(),
            )];
            let end_line = file.line_start(location.end_offset().saturating_sub(1));
            if let Some(next) = elements.get(index + 1) {
                if first_line < end_line
                    && file.line_start(next.location().start_offset()) == end_line
                {
                    edits.push((
                        next.location().start_offset()..next.location().start_offset(),
                        "\n".to_string(),
                    ));
                }
            }
            context.replace_many(message, &location, edits);
        } else {
            last_seen_line = Some(file.line_start(location.end_offset().saturating_sub(1)));
        }
    }
}

fn space_before_block_braces(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let opening = if let Some(block) = node.as_block_node() {
        block.opening_loc()
    } else if let Some(lambda) = node.as_lambda_node() {
        lambda.opening_loc()
    } else {
        return;
    };
    if opening.as_slice() != b"{" {
        return;
    }
    let empty = node
        .as_block_node()
        .is_some_and(|block| block.body().is_none())
        || node
            .as_lambda_node()
            .is_some_and(|lambda| lambda.body().is_none());
    check_space_before_block_brace(opening.start_offset(), empty, context);
}

fn check_space_before_block_brace(offset: usize, empty: bool, context: &mut CopContext<'_, '_>) {
    let default_no_space = context.policy().enforced_style("space") == "no_space";
    let no_space = if empty {
        context
            .config_value("EnforcedStyleForEmptyBraces")
            .unwrap_or("space")
            == "no_space"
    } else {
        default_no_space
    };
    let before = context.source().as_bytes().get(offset.wrapping_sub(1));
    if no_space && before == Some(&b' ') {
        context.remove(
            "Space detected to the left of {.",
            offset - 1..offset,
            offset - 1..offset,
        );
    } else if !no_space && before.is_some_and(|byte| !byte.is_ascii_whitespace()) {
        context.insert(
            "Space missing to the left of {.",
            offset..offset + 1,
            offset,
            " ",
        );
    }
}

impl BlockEndNewlineRule<'_, '_, '_> {
    fn on_block(&mut self, node: &Node<'_>) {
        let (location, body, closing) = if let Some(block) = node.as_block_node() {
            (block.location(), block.body(), block.closing_loc())
        } else if let Some(lambda) = node.as_lambda_node() {
            (lambda.location(), lambda.body(), lambda.closing_loc())
        } else {
            return;
        };
        let file = self.source_file();
        let block_source = self
            .source()
            .get(location.start_offset()..closing.end_offset())
            .unwrap_or_default();
        if !block_source.contains('\n') {
            return;
        }
        let line_start = file.line_start(closing.start_offset());
        if self.source()[line_start..closing.start_offset()]
            .trim()
            .is_empty()
        {
            return;
        }
        let content_end = body
            .and_then(last_block_body_expression)
            .map(|last| last.location().end_offset())
            .or_else(|| {
                self.source()[location.start_offset()..closing.start_offset()]
                    .rfind('|')
                    .map(|offset| location.start_offset() + offset + 1)
            });
        let Some(mut content_end) = content_end else {
            return;
        };
        let original_between = self
            .source()
            .get(content_end..closing.start_offset())
            .unwrap_or_default();
        if original_between
            .trim_start_matches(char::is_whitespace)
            .starts_with(';')
        {
            return;
        }
        let closing_prefix = &self.source()[line_start..closing.start_offset()];
        content_end = content_end.max(line_start + closing_prefix.trim_end().len());
        let edit = content_end..closing.start_offset();
        let between = self.source().get(edit.clone()).unwrap_or_default();
        let preserved = between.trim_start_matches(char::is_whitespace);
        let line = self.source()[..closing.start_offset()]
            .matches('\n')
            .count()
            + 1;
        let column = self.source()[line_start..closing.start_offset()]
            .chars()
            .count()
            + 1;
        let message = format!("Expression at {line}, {column} should be on its own line.");
        let header = &self.source()[location.start_offset()..closing.start_offset()];
        let heredoc_marker = header.rsplit_once("<<").and_then(|(_, tail)| {
            let marker = tail
                .trim_start_matches(['~', '-', '`'])
                .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
                .next()
                .unwrap_or_default();
            (!marker.is_empty()).then_some(marker)
        });
        if let Some(marker) = heredoc_marker {
            let search_start = closing.end_offset();
            if let Some((terminator_offset, terminator_line)) = self
                .source_file()
                .lines()
                .find(|(offset, line)| *offset >= search_start && line.trim() == marker)
            {
                let terminator_end = terminator_offset + terminator_line.len();
                self.replace_many(
                    message,
                    closing.start_offset()..closing.end_offset(),
                    vec![
                        (content_end..closing.end_offset(), String::new()),
                        (terminator_end..terminator_end, "\n}".to_string()),
                    ],
                );
                return;
            }
        }
        self.replace(
            message,
            closing.start_offset()..closing.end_offset(),
            edit,
            format!("\n{preserved}"),
        );
    }
}

fn last_block_body_expression(node: Node<'_>) -> Option<Node<'_>> {
    if let Some(statements) = node.as_statements_node() {
        return statements.body().iter().last();
    }
    if let Some(begin) = node.as_begin_node() {
        return begin
            .statements()
            .and_then(|statements| statements.body().iter().last());
    }
    Some(node)
}

fn def_end_alignment(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(end) = node.end_keyword_loc() else {
        return;
    };
    let keyword = node.def_keyword_loc();
    let file = context.source_file();
    let def_line_start = file.line_start(keyword.start_offset());
    let end_line_start = file.line_start(end.start_offset());
    if def_line_start == end_line_start {
        return;
    }
    let keyword_column = context.source()[def_line_start..keyword.start_offset()]
        .chars()
        .count();
    let indentation = context.source()[def_line_start..keyword.start_offset()]
        .chars()
        .take_while(|character| character.is_whitespace())
        .count();
    let align_with_def = context
        .config_value("EnforcedStyleAlignWith")
        .is_some_and(|style| style == "def");
    let expected = if align_with_def {
        keyword_column
    } else {
        indentation
    };
    let actual = context.source()[end_line_start..end.start_offset()]
        .chars()
        .count();
    if actual == expected {
        return;
    }
    let def_line = context.source()[..keyword.start_offset()]
        .matches('\n')
        .count()
        + 1;
    let end_line = context.source()[..end.start_offset()].matches('\n').count() + 1;
    let (opening, reference_column) = if !align_with_def && indentation != keyword_column {
        (
            context.source()[def_line_start + indentation..keyword.end_offset()]
                .trim()
                .to_string(),
            indentation,
        )
    } else {
        ("def".to_string(), keyword_column)
    };
    context.replace(
        format!(
            "`end` at {end_line}, {actual} is not aligned with `{opening}` at {def_line}, {reference_column}."
        ),
        end.start_offset()..end.end_offset(),
        end_line_start..end.start_offset(),
        " ".repeat(expected),
    );
}

fn parameter_alignment(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(parameters) = node.parameters() else {
        return;
    };
    let parameters = definition_parameter_nodes(&parameters);
    if parameters.len() < 2 {
        return;
    }

    let file = context.source_file();
    let fixed = context.config_value("EnforcedStyle") == Some("with_fixed_indentation");
    let expected = if fixed {
        let width = context
            .config_value("IndentationWidth")
            .and_then(|value| value.parse::<usize>().ok())
            .or_else(|| {
                context
                    .related_config_value("Layout/IndentationWidth", "Width")
                    .and_then(|value| value.parse::<usize>().ok())
            })
            .unwrap_or(2);
        file.indentation(node.def_keyword_loc().start_offset())
            .len()
            + width
    } else {
        file.column(parameters[0].location().start_offset())
    };

    let mut previous_line_start = usize::MAX;
    for parameter in parameters {
        let location = parameter.location();
        let line_start = file.line_start(location.start_offset());
        if line_start == previous_line_start {
            continue;
        }
        previous_line_start = line_start;
        if !context.source()[line_start..location.start_offset()]
            .trim()
            .is_empty()
        {
            continue;
        }
        let actual = file.column(location.start_offset());
        if actual == expected {
            continue;
        }
        let message = if fixed {
            "Use one level of indentation for parameters following the first line of a multi-line method definition."
        } else {
            "Align the parameters of a method definition if they span more than one line."
        };
        context.replace(
            message,
            &location,
            line_start..location.start_offset(),
            " ".repeat(expected),
        );
    }
}

fn definition_parameter_nodes<'pr>(parameters: &ruby_prism::ParametersNode<'pr>) -> Vec<Node<'pr>> {
    let mut result = parameters.requireds().iter().collect::<Vec<_>>();
    result.extend(parameters.optionals().iter());
    result.extend(parameters.rest());
    result.extend(parameters.posts().iter());
    result.extend(parameters.keywords().iter());
    result.extend(parameters.keyword_rest());
    result.extend(parameters.block().map(|block| block.as_node()));
    result
}
