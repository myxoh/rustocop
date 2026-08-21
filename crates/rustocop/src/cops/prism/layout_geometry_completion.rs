use super::*;

define_cops! {
    MultilineMethodParameterLineBreaks => "Layout/MultilineMethodParameterLineBreaks" => source(parameter_line_breaks),
    SpaceBeforeBlockBraces => "Layout/SpaceBeforeBlockBraces" => source(space_before_block_braces),
    BlockEndNewline => "Layout/BlockEndNewline" => node(as_block_node, block_end_newline),
    DefEndAlignment => "Layout/DefEndAlignment" => node(as_def_node, def_end_alignment),
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

fn block_end_newline(node: &ruby_prism::BlockNode<'_>, context: &mut CopContext<'_, '_>) {
    let opening = node.opening_loc();
    let closing = node.closing_loc();
    let file = context.source_file();
    if file.same_line(opening.start_offset(), closing.start_offset()) {
        return;
    }

    let line_start = context.source()[..closing.start_offset()]
        .rfind('\n')
        .map_or(0, |newline| newline + 1);
    let before_closing = &context.source()[line_start..closing.start_offset()];
    if before_closing.trim().is_empty() || before_closing.trim_start().starts_with(';') {
        return;
    }

    let whitespace_start =
        line_start + before_closing.trim_end_matches([' ', '\t']).len();
    let line = context.source()[..closing.start_offset()]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let column = closing.start_offset() - line_start + 1;
    let message = format!("Expression at {line}, {column} should be on its own line.");
    let block_prefix = &context.source()[opening.end_offset()..closing.start_offset()];
    if let Some(marker) = last_heredoc_marker(block_prefix) {
        if let Some((terminator_start, _)) = file
            .lines()
            .find(|(start, line)| *start >= closing.end_offset() && line.trim() == marker)
        {
            let insertion = file.line_range(terminator_start).end;
            context.replace_many(
                message,
                closing.start_offset()..closing.end_offset(),
                vec![
                    (whitespace_start..closing.end_offset(), String::new()),
                    (insertion..insertion, format!("{}\n", file.at(&closing))),
                ],
            );
            return;
        }
    }
    context.replace(
        message,
        closing.start_offset()..closing.end_offset(),
        whitespace_start..closing.start_offset(),
        "\n",
    );
}

fn last_heredoc_marker(source: &str) -> Option<&str> {
    source
        .match_indices("<<")
        .filter_map(|(offset, _)| {
            let mut rest = &source[offset + 2..];
            rest = rest.strip_prefix(['~', '-']).unwrap_or(rest);
            if rest.starts_with(['\'', '"']) {
                rest = &rest[1..];
            }
            let length = rest
                .bytes()
                .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                .count();
            (length > 0).then_some(&rest[..length])
        })
        .last()
}

fn def_end_alignment(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(end_keyword) = node.end_keyword_loc() else {
        return;
    };
    let keyword = node.def_keyword_loc();
    let source = context.source();
    let def_line_start = source[..keyword.start_offset()]
        .rfind('\n')
        .map_or(0, |newline| newline + 1);
    let end_line_start = source[..end_keyword.start_offset()]
        .rfind('\n')
        .map_or(0, |newline| newline + 1);
    let start_of_line_column = source[def_line_start..]
        .bytes()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count();
    let def_column = keyword.start_offset() - def_line_start;
    let actual = end_keyword.start_offset() - end_line_start;
    let expected = if context
        .config_value("EnforcedStyleAlignWith")
        .unwrap_or("start_of_line")
        == "def"
    {
        def_column
    } else {
        start_of_line_column
    };
    if actual == expected {
        return;
    }

    let def_line = source[..keyword.start_offset()]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let end_line = source[..end_keyword.start_offset()]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let reference_end = keyword.end_offset();
    let reference_start = if expected == def_column {
        keyword.start_offset()
    } else {
        def_line_start + start_of_line_column
    };
    let reference = source[reference_start..reference_end].trim_end();
    context.replace(
        format!(
            "`end` at {end_line}, {actual} is not aligned with `{reference}` at {def_line}, {expected}."
        ),
        end_keyword.start_offset()..end_keyword.end_offset(),
        end_line_start..end_keyword.start_offset(),
        " ".repeat(expected),
    );
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
