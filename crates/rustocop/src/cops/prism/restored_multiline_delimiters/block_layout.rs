use super::*;

const BODY_MESSAGE: &str = "Block body expression is on the same line as the block start.";
const ARGUMENT_MESSAGE: &str =
    "Block argument expression is not on the same line as the block start.";

pub(super) fn multiline_block_layout(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (location, opening, closing, parameters, body, start_column) =
        if let Some(block) = node.as_block_node() {
            let start_column = context
                .parent()
                .and_then(Node::as_call_node)
                .map(|call| context.source_file().column(call.location().start_offset()))
                .unwrap_or_else(|| context.source_file().column(block.location().start_offset()));
            (
                block.location(),
                block.opening_loc(),
                block.closing_loc(),
                block.parameters(),
                block.body(),
                start_column,
            )
        } else if let Some(lambda) = node.as_lambda_node() {
            (
                lambda.location(),
                lambda.opening_loc(),
                lambda.closing_loc(),
                lambda.parameters(),
                lambda.body(),
                context.source_file().column(lambda.location().start_offset()),
            )
        } else {
            return;
        };

    let file = context.source_file();
    if file.same_line(location.start_offset(), closing.end_offset()) {
        return;
    }

    let body_location = body.as_ref().map(Node::location);
    let body_start = body.as_ref().and_then(first_body_offset);
    let mut moved_parameters = false;
    if let Some(parameters) = parameters {
        let parameter_location = parameters.location();
        let parameter_source = file.at(&parameter_location);
        if parameter_source.trim_start().starts_with('|')
            && parameter_source.trim_end().ends_with('|')
            && !file.same_line(opening.end_offset(), parameter_location.end_offset())
        {
            let normalized = normalize_parameters(parameter_source);
            if parameters_fit(context, opening.end_offset(), &normalized) {
                let replacement_end = consume_horizontal_space(file, parameter_location.end_offset());
                context.add_offense(&parameter_location, ARGUMENT_MESSAGE, |corrector| {
                    corrector.replace(
                        opening.end_offset()..replacement_end,
                        format!(" {normalized}"),
                    );
                    if let Some(body_start) = body_start {
                        if file.same_line(parameter_location.end_offset(), body_start) {
                            corrector.replace(
                                body_start..body_start,
                                format!("\n{}", " ".repeat(start_column + 2)),
                            );
                        }
                    }
                });
                moved_parameters = true;
            }
        }
    }

    let Some(body_location) = body_location else { return };
    let Some(body_start) = body_start else { return };
    if moved_parameters || !file.same_line(opening.end_offset(), body_start) {
        return;
    }
    context.add_offense(&body_location, BODY_MESSAGE, |corrector| {
        corrector.replace(
            body_location.start_offset()..body_location.start_offset(),
            format!("\n{}", " ".repeat(start_column + 2)),
        );
    });
}

fn first_body_offset(node: &Node<'_>) -> Option<usize> {
    if let Some(statements) = node.as_statements_node() {
        return statements
            .body()
            .iter()
            .next()
            .map(|expression| expression.location().start_offset());
    }
    if let Some(begin) = node.as_begin_node() {
        return begin
            .statements()
            .and_then(|statements| statements.body().iter().next())
            .map(|expression| expression.location().start_offset());
    }
    Some(node.location().start_offset())
}

fn normalize_parameters(source: &str) -> String {
    let mut output = String::new();
    let mut pending_space = false;
    for character in source.trim().chars() {
        if character.is_whitespace() {
            pending_space = true;
            continue;
        }
        match character {
            ',' => {
                while output.ends_with(' ') {
                    output.pop();
                }
                output.push(',');
                pending_space = true;
            }
            '(' => {
                if pending_space && !output.is_empty() && !output.ends_with(['(', '|', ' ']) {
                    output.push(' ');
                }
                output.push('(');
                pending_space = false;
            }
            ')' | '|' => {
                while output.ends_with(' ') {
                    output.pop();
                }
                output.push(character);
                pending_space = false;
            }
            _ => {
                if pending_space && !output.is_empty() && !output.ends_with(['(', '|', ' ']) {
                    output.push(' ');
                }
                output.push(character);
                pending_space = false;
            }
        }
    }
    output
}

fn parameters_fit(context: &CopContext<'_, '_>, opening_end: usize, normalized: &str) -> bool {
    if context.related_config_value("Layout/LineLength", "Enabled") == Some("false") {
        return true;
    }
    let max = context
        .related_config_value("Layout/LineLength", "Max")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(120);
    context.source_file().column(opening_end) + 1 + normalized.chars().count() <= max
}

fn consume_horizontal_space(file: SourceFile<'_>, mut offset: usize) -> usize {
    while matches!(file.as_str().as_bytes().get(offset), Some(b' ' | b'\t')) {
        offset += 1;
    }
    offset
}
