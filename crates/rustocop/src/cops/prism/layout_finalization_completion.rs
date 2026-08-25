use super::*;

define_cops! {
    EmptyComment => "Layout/EmptyComment" => source(empty_comment),
    EmptyLineAfterMagicComment => "Layout/EmptyLineAfterMagicComment" => source(empty_line_after_magic_comment),
    SpaceAfterNot => "Layout/SpaceAfterNot" => call(space_after_not),
    SpaceInsideRangeLiteral => "Layout/SpaceInsideRangeLiteral" => node(as_range_node, space_inside_range_literal),
    SpaceAroundEqualsInParameterDefault => "Layout/SpaceAroundEqualsInParameterDefault" => node(as_optional_parameter_node, space_around_parameter_equals),
    SpaceInLambdaLiteral => "Layout/SpaceInLambdaLiteral" => node(as_lambda_node, space_in_lambda_literal),
    TrailingEmptyLines => "Layout/TrailingEmptyLines" => source(trailing_empty_lines),
    TrailingBodyOnMethodDefinition => "Style/TrailingBodyOnMethodDefinition" => node(as_def_node, trailing_method_body),
    InlineComment => "Style/InlineComment" => source(inline_comment),
}

fn space_after_not(node: &ruby_prism::CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.name().as_slice() != b"!" {
        return;
    }
    let Some(receiver) = node.receiver() else {
        return;
    };
    let operator = node.message_loc().unwrap_or_else(|| node.location());
    if operator.as_slice() != b"!" {
        return;
    }
    if operator.end_offset() == receiver.location().start_offset() {
        return;
    }
    context.replace(
        "Do not leave space between `!` and its argument.",
        node.location(),
        operator.end_offset()..receiver.location().start_offset(),
        "",
    );
}

fn space_inside_range_literal(
    node: &ruby_prism::RangeNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let (Some(left), Some(right)) = (node.left(), node.right()) else {
        return;
    };
    let operator = node.operator_loc();
    let whitespace_after = &context.source()[operator.end_offset()..right.location().start_offset()];
    if left.location().end_offset() == operator.start_offset()
        && (operator.end_offset() == right.location().start_offset()
            || whitespace_after.starts_with('\n')) {
        return;
    }
    let source = context.source_file();
    let replacement = format!(
        "{}{}{}",
        source.at(&left.location()),
        source.at(&operator),
        source.at(&right.location())
    );
    context.replace(
        "Space inside range literal.",
        node.location(),
        node.location(),
        replacement,
    );
}

fn inline_comment(context: &mut CopContext<'_, '_>) {
    #[derive(Default)]
    struct EmbeddedRuby(Vec<std::ops::Range<usize>>);
    impl<'pr> Visit<'pr> for EmbeddedRuby {
        fn visit_embedded_statements_node(&mut self, node: &ruby_prism::EmbeddedStatementsNode<'pr>) {
            self.0.push(node.location().start_offset()..node.location().end_offset());
            ruby_prism::visit_embedded_statements_node(self, node);
        }
        fn visit_embedded_variable_node(&mut self, node: &ruby_prism::EmbeddedVariableNode<'pr>) {
            self.0.push(node.location().start_offset()..node.location().end_offset());
            ruby_prism::visit_embedded_variable_node(self, node);
        }
    }

    let source = context.source_file();
    let parsed = ruby_prism::parse(context.source().as_bytes());
    let mut embedded = EmbeddedRuby::default();
    embedded.visit(&parsed.node());
    let mut literals = source.literal_ranges();
    let heredoc_literals = source.heredoc_ranges();
    for literal in &mut literals {
        if heredoc_literals.contains(literal) {
            literal.start = source.line_end(literal.start).saturating_add(1).min(literal.end);
        }
    }
    let mut comments = parsed
        .comments()
        .map(|comment| comment.location().start_offset()..comment.location().end_offset())
        .collect::<Vec<_>>();
    let heredocs = super::source_rules_layout::lexical_heredoc_body_ranges(context.source());
    for (offset, line) in source.lines() {
            let Some(hash) = line
                .match_indices('#')
                .map(|(hash, _)| hash)
                .find(|hash| outside_simple_quotes(line, *hash))
            else {
                continue;
            };
            if hash == 0
                || !line.as_bytes()[hash - 1].is_ascii_whitespace()
                || line.as_bytes().get(hash + 1) == Some(&b'{')
            {
                continue;
            }
            let start = offset + hash;
            if !comments
                .iter()
                .any(|range| range.start <= start && start < range.end)
                && !heredocs.iter().any(|range| range.start <= start && start < range.end)
            {
                comments.push(start..offset + line.len());
            }
    }
    comments.sort_by_key(|range| range.start);
    for location in comments {
        let comment = &context.source().as_bytes()[location.clone()];
        if comment.starts_with(b"=begin") {
            context.report("Avoid trailing inline comments.", location);
            continue;
        }
        if literals
            .iter()
            .any(|literal| literal.start <= location.start && location.start < literal.end)
            && !embedded
                .0
                .iter()
                .any(|ruby| ruby.start <= location.start && location.start < ruby.end)
        {
            continue;
        }
        let line_start = source.line_start(location.start);
        let prefix = &context.source()[line_start..location.start];
        if prefix.trim().is_empty()
            || comment.starts_with(b"# rubocop:disable")
            || comment.starts_with(b"# rubocop:enable")
        {
            continue;
        }
        context.report("Avoid trailing inline comments.", location);
    }
}

fn outside_simple_quotes(line: &str, end: usize) -> bool {
    let mut quote = None;
    let mut escaped = false;
    let mut previous_significant = None;
    for byte in line.as_bytes()[..end].iter().copied() {
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if quote == Some(byte) {
            quote = None;
        } else if quote.is_none()
            && (matches!(byte, b'\'' | b'"' | b'`')
                || byte == b'/'
                    && previous_significant.is_none_or(|previous| {
                        matches!(
                            previous,
                            b'(' | b'[' | b'{' | b',' | b'=' | b'!' | b'~' | b'?' | b':' | b';'
                        )
                    }))
        {
            quote = Some(byte);
        }
        if !byte.is_ascii_whitespace() {
            previous_significant = Some(byte);
        }
    }
    quote.is_none()
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
        let comment = line.trim_start();
        comment.starts_with('#') && !matches!(comment.trim_end(), "#" | "# ")
    };
    let column = lines[index].1.len() - lines[index].1.trim_start().len();
    let is_same_column_comment = |line: &str| {
        line.trim_start().starts_with('#') && line.len() - line.trim_start().len() == column
    };
    let mut before = index;
    while before > 0 && is_same_column_comment(lines[before - 1].1) {
        before -= 1;
    }
    let mut after = index + 1;
    while after < lines.len() && is_same_column_comment(lines[after].1) {
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
        if trimmed.is_empty() && last_magic.is_none() {
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
    let line = line.to_ascii_lowercase();
    line.starts_with("# frozen_string_literal:")
        || line.starts_with("# encoding:")
        || line.starts_with("# coding:")
        || (line.starts_with("# -*-")
            && (line.contains(" encoding:") || line.contains(" coding:")))
        || line.starts_with("# warn_indent:")
        || line.starts_with("# shareable_constant_value:")
        || line.starts_with("# typed:")
        || matches!(line.as_str(), "# rbs_inline: enabled" | "# rbs_inline: disabled")
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
