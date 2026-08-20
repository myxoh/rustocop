use super::*;

define_rule!(BlockEndNewlineRule);

define_cops! {
    MultilineMethodParameterLineBreaks => "Layout/MultilineMethodParameterLineBreaks" => source(parameter_line_breaks),
    SpaceBeforeBlockBraces => "Layout/SpaceBeforeBlockBraces" => source(space_before_block_braces),
    BlockEndNewline => "Layout/BlockEndNewline" => node_rule_aliases(
        BlockEndNewlineRule,
        on_block => [as_block_node, as_lambda_node]
    ),
    DefEndAlignment => "Layout/DefEndAlignment" => source(def_end_alignment),
    MultilineMethodArgumentLineBreaks => "Layout/MultilineMethodArgumentLineBreaks" => source(argument_line_breaks),
    ParameterAlignment => "Layout/ParameterAlignment" => source(parameter_alignment),
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

fn def_end_alignment(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut stack = Vec::new();
    for (offset, line) in lines {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        if let Some(definition) = line.find("def ") {
            stack.push((
                offset,
                if definition > 0 { 0 } else { indent },
                line[..definition + 3].trim().to_string(),
            ));
        }
        if trimmed == "end" {
            let Some((def_offset, expected, opening)) = stack.pop() else {
                continue;
            };
            if indent != expected {
                let def_line = context.source()[..def_offset].matches('\n').count() + 1;
                let end_line = context.source()[..offset].matches('\n').count() + 1;
                context.replace(format!("`end` at {end_line}, {indent} is not aligned with `{opening}` at {def_line}, {expected}."), offset + indent..offset + line.len(), offset..offset + indent, " ".repeat(expected));
            }
        }
    }
}

fn parameter_alignment(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut expected = None;
    for (offset, line) in lines {
        if let Some(open) = line
            .find('(')
            .filter(|_| line.trim_start().starts_with("def "))
        {
            expected = Some(open + 1);
            continue;
        }
        let Some(column) = expected else { continue };
        if line.contains(')') {
            expected = None;
        }
        if line.trim().is_empty() || line.trim_start().starts_with(')') {
            continue;
        }
        let actual = line.len() - line.trim_start().len();
        if actual != column {
            context.replace(
                "Align the parameters of a method definition if they span more than one line.",
                offset + actual..offset + actual + 1,
                offset..offset + actual,
                " ".repeat(column),
            );
        }
    }
}
