use super::*;

define_rule!(BlockEndNewlineRule);

define_cops! {
    MultilineMethodParameterLineBreaks => "Layout/MultilineMethodParameterLineBreaks" => source(parameter_line_breaks),
    SpaceBeforeBlockBraces => "Layout/SpaceBeforeBlockBraces" => source(space_before_block_braces),
    BlockEndNewline => "Layout/BlockEndNewline" => node_rule_aliases(
        BlockEndNewlineRule,
        on_block => [as_block_node, as_lambda_node]
    ),
    DefEndAlignment => "Layout/DefEndAlignment" => node(as_def_node, def_end_alignment),
    MultilineMethodArgumentLineBreaks => "Layout/MultilineMethodArgumentLineBreaks" => source(argument_line_breaks),
    ParameterAlignment => "Layout/ParameterAlignment" => node(as_def_node, parameter_alignment),
}

fn parameter_line_breaks(context: &mut CopContext<'_, '_>) {
    comma_line_breaks(context, true);
}
fn argument_line_breaks(context: &mut CopContext<'_, '_>) {
    comma_line_breaks(context, false);
}

fn comma_line_breaks(context: &mut CopContext<'_, '_>, parameters: bool) {
    let source = context.source();
    if !source.contains('\n') {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        let commas = line
            .match_indices(',')
            .map(|(at, _)| at)
            .collect::<Vec<_>>();
        for comma in commas.into_iter().skip(1) {
            let rest = &line[comma + 1..];
            let leading = rest.len() - rest.trim_start().len();
            let value = rest
                .trim_start()
                .split(',')
                .next()
                .unwrap_or_default()
                .trim_end();
            if value.is_empty() {
                continue;
            }
            let start = offset + comma + 1 + leading;
            let message = if parameters {
                "Each parameter in a multi-line method definition must start on a separate line."
            } else {
                "Each argument in a multi-line method call must start on a separate line."
            };
            context.replace(
                message,
                start..start + value.len(),
                offset + comma + 1..start,
                "\n",
            );
        }
    }
}

fn space_before_block_braces(context: &mut CopContext<'_, '_>) {
    let default_no_space = context.policy().enforced_style("space") == "no_space";
    for offset in context.source_file().code_offsets("{") {
        let empty = context.source().as_bytes().get(offset + 1) == Some(&b'}');
        let no_space = if empty {
            context
                .config_value("EnforcedStyleForEmptyBraces")
                .unwrap_or("no_space")
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
        } else if !no_space
            && before.is_some_and(|byte| {
                !byte.is_ascii_whitespace() && !matches!(byte, b'{' | b'(' | b'[')
            })
        {
            context.insert(
                "Space missing to the left of {.",
                offset..offset + 1,
                offset,
                " ",
            );
        }
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
    let Some(last) = body.and_then(last_block_body_expression) else {
        return;
    };
    let edit = last.location().end_offset()..closing.start_offset();
    let between = self.source().get(edit.clone()).unwrap_or_default();
    let preserved = between.trim_start_matches(char::is_whitespace);
    if preserved.starts_with(';') {
        return;
    }
    let line = self.source()[..closing.start_offset()].matches('\n').count() + 1;
    let column = self.source()[line_start..closing.start_offset()]
        .chars()
        .count()
        + 1;
    self.replace(
        format!("Expression at {line}, {column} should be on its own line."),
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
    let def_line = context.source()[..keyword.start_offset()].matches('\n').count() + 1;
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
        file.indentation(node.def_keyword_loc().start_offset()).len() + width
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

fn definition_parameter_nodes<'pr>(
    parameters: &ruby_prism::ParametersNode<'pr>,
) -> Vec<Node<'pr>> {
    let mut result = parameters.requireds().iter().collect::<Vec<_>>();
    result.extend(parameters.optionals().iter());
    result.extend(parameters.rest());
    result.extend(parameters.posts().iter());
    result.extend(parameters.keywords().iter());
    result.extend(parameters.keyword_rest());
    result.extend(parameters.block().map(|block| block.as_node()));
    result
}
