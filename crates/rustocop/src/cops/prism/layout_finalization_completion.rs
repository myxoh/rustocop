use super::*;

define_cops! {
    EmptyComment => "Layout/EmptyComment" => source(empty_comment),
    EmptyLineAfterMagicComment => "Layout/EmptyLineAfterMagicComment" => source(empty_line_after_magic_comment),
    SpaceAroundEqualsInParameterDefault => "Layout/SpaceAroundEqualsInParameterDefault" => node(as_optional_parameter_node, space_around_parameter_equals),
    SpaceInLambdaLiteral => "Layout/SpaceInLambdaLiteral" => node(as_lambda_node, space_in_lambda_literal),
    TrailingEmptyLines => "Layout/TrailingEmptyLines" => source(trailing_empty_lines),
    TrailingBodyOnMethodDefinition => "Style/TrailingBodyOnMethodDefinition" => node(as_def_node, trailing_method_body),
}

fn empty_comment(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let comments = context.source_file().comment_ranges();
    let heredocs = context.source_file().heredoc_ranges();
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        let trimmed = line.trim_start();
        let comment_at = comments
            .iter()
            .find(|comment| comment.start >= offset && comment.start <= offset + line.len())
            .filter(|comment| {
                !heredocs
                    .iter()
                    .any(|heredoc| heredoc.start <= comment.start && comment.start < heredoc.end)
            })
            .map(|comment| comment.start - offset);
        let Some(comment_at) = comment_at else {
            continue;
        };
        if !context.config_bool("AllowBorderComment", true)
            && !trimmed.is_empty()
            && trimmed.bytes().all(|byte| byte == b'#')
        {
            let indent = line.len() - trimmed.len();
            context.remove(
                "Source code comment is empty.",
                offset + indent..offset + line.len(),
                offset
                    ..offset
                        + line.len()
                        + usize::from(
                            context.source().as_bytes().get(offset + line.len()) == Some(&b'\n'),
                        ),
            );
            continue;
        }
        let inline = Some(comment_at)
            .filter(|at| !line[..*at].trim().is_empty() && line[*at + 1..].trim().is_empty());
        if !matches!(trimmed.trim_end(), "#" | "# ") && inline.is_none() {
            continue;
        }
        if inline.is_none()
            && context.config_bool("AllowBorderComment", true)
            && context.config_bool("AllowMarginComment", true)
            && comment_block_has_content(&lines, index)
        {
            continue;
        }
        let indent = inline.unwrap_or(line.len() - trimmed.len());
        let edit_start = if inline.is_some() {
            line[..indent].trim_end().len()
        } else {
            0
        };
        context.remove(
            "Source code comment is empty.",
            offset + indent..offset + line.len(),
            offset + edit_start
                ..offset
                    + line.len()
                    + usize::from(
                        inline.is_none()
                            && context.source().as_bytes().get(offset + line.len()) == Some(&b'\n'),
                    ),
        );
    }
}

fn comment_block_has_content(lines: &[(usize, &str)], index: usize) -> bool {
    let has_content = |line: &str| {
        let line = line.trim_start();
        line.starts_with('#') && !line.trim_start_matches('#').trim().is_empty()
    };
    let is_comment = |line: &str| line.trim_start().starts_with('#');
    let mut before = index;
    while before > 0 && is_comment(lines[before - 1].1) {
        before -= 1;
    }
    let mut after = index + 1;
    while after < lines.len() && is_comment(lines[after].1) {
        after += 1;
    }
    lines[before..after]
        .iter()
        .any(|(_, line)| has_content(line))
}

fn empty_line_after_magic_comment(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut last_magic = None;
    for (index, (_, line)) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if index == 0 && trimmed.starts_with("#!") {
            continue;
        }
        if is_magic_comment(trimmed)
        {
            last_magic = Some(index);
            continue;
        }
        if trimmed.starts_with('#') && last_magic.is_some() {
            continue;
        }
        break;
    }
    let Some(magic) = last_magic else { return };
    let Some((offset, line)) = lines.get(magic + 1) else {
        return;
    };
    if line.trim().is_empty() {
        return;
    }
    context.insert(
        "Add an empty line after magic comments.",
        *offset..(*offset + 1).min(context.source().len()),
        *offset,
        "\n",
    );
}

fn is_magic_comment(line: &str) -> bool {
    line.starts_with("# frozen_string_literal:")
        || line.starts_with("# encoding:")
        || line.starts_with("# coding:")
        || (line.starts_with("# -*-")
            && (line.contains(" encoding:") || line.contains(" coding:")))
        || line.starts_with("# warn_indent:")
        || line.starts_with("# shareable_constant_value:")
        || line.starts_with("# typed:")
        || matches!(line, "# rbs_inline: enabled" | "# rbs_inline: disabled")
}

fn space_in_lambda_literal(
    node: &ruby_prism::LambdaNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let require_space = context.policy().enforced_style("require_no_space") == "require_space";
    let Some(arguments) = node
        .parameters()
        .and_then(|parameters| parameters.as_block_parameters_node())
    else {
        return;
    };
    if arguments.parameters().is_none() && arguments.locals().iter().next().is_none() {
        return;
    }
    let argument_start = arguments
        .opening_loc()
        .map_or_else(|| arguments.location().start_offset(), |opening| opening.start_offset());
    let argument_end = arguments
        .closing_loc()
        .map_or_else(|| arguments.location().end_offset(), |closing| closing.end_offset());
    let arrow = node.operator_loc();
    let between = arrow.end_offset()..argument_start;
    if require_space && between.is_empty() {
        context.insert(
            "Use a space between `->` and `(` in lambda literals.",
            arrow.start_offset()..argument_end,
            arrow.end_offset(),
            " ",
        );
    } else if !require_space && !between.is_empty() {
        context.remove(
            "Do not use spaces between `->` and `(` in lambda literals.",
            between.clone(),
            between,
        );
    }
}

fn space_around_parameter_equals(
    node: &ruby_prism::OptionalParameterNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let use_space = context.policy().enforced_style("space") == "space";
    let operator = node.operator_loc();
    let at = operator.start_offset();
    let source = context.source().as_bytes();
    let left_space = at > 0 && source[at - 1] == b' ';
    let right_space = source.get(operator.end_offset()) == Some(&b' ');
    if use_space && (!left_space || !right_space) {
        let range = at - usize::from(left_space)
            ..operator.end_offset() + usize::from(right_space);
        context.replace(
            "Surrounding space missing in default value assignment.",
            range.clone(),
            range,
            " = ",
        );
    } else if !use_space && (left_space || right_space) {
        let range = at - usize::from(left_space)
            ..operator.end_offset() + usize::from(right_space);
        context.replace(
            "Surrounding space detected in default value assignment.",
            range.clone(),
            range,
            "=",
        );
    }
}

fn trailing_empty_lines(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    if source.is_empty() {
        return;
    }
    if source.lines().any(|line| line.trim() == "__END__") {
        return;
    }
    let first_line = source.lines().next().unwrap_or_default().trim_end();
    if first_line.ends_with('%') || first_line.ends_with("%Q") || first_line.ends_with("%q") {
        return;
    }
    let content_end = source.trim_end().len();
    let trailing = &source[content_end..];
    let newline_count = trailing.matches('\n').count();
    let blank_lines = newline_count.saturating_sub(1);
    let final_blank = context.policy().enforced_style("final_newline") == "final_blank_line";
    let wanted = if final_blank { 2 } else { 1 };
    if newline_count == wanted {
        return;
    }
    let message = if final_blank && newline_count == 1 {
        "Trailing blank line missing.".to_string()
    } else if newline_count == 0 {
        "Final newline missing.".to_string()
    } else if final_blank {
        format!("{blank_lines} trailing blank lines instead of 1 detected.")
    } else {
        format!("{blank_lines} trailing blank lines detected.")
    };
    let offense = if newline_count == 0 || (final_blank && newline_count == 1) {
        source.len()..source.len()
    } else {
        content_end + 1..source.len()
    };
    context.replace(
        message,
        offense,
        content_end..source.len(),
        "\n".repeat(wanted),
    );
}

fn trailing_method_body(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.equal_loc().is_some() {
        return;
    }
    let location = node.location();
    let file = context.source_file();
    if file.same_line(location.start_offset(), location.end_offset().saturating_sub(1)) {
        return;
    }
    let Some(body) = node.body() else {
        return;
    };
    let header_end = node
        .rparen_loc()
        .map_or_else(
            || {
                node.parameters()
                    .map_or(node.name_loc().end_offset(), |parameters| {
                        parameters.location().end_offset()
                    })
            },
            |rparen| rparen.end_offset(),
        );
    super::structural_completion_rules::report_trailing_body(
        location.start_offset(),
        header_end,
        body,
        "Place the first line of a multi-line method definition's body on its own line.",
        context,
    );
}
