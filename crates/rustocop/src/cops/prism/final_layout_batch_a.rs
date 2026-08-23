use super::catalog_cop::custom;
use super::*;

mod registry;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    let mut cops = vec![
        custom("Layout/LineContinuationSpacing", line_continuation_spacing),
        Box::new(MultilineMethodDefinitionBraceLayout),
        custom("Layout/SpaceInsideParens", space_inside_parens),
        Box::new(ClosingParenthesisIndentation),
        custom("Layout/IndentationStyle", indentation_style),
        custom("Layout/LeadingCommentSpace", leading_comment_space),
        custom("Layout/CommentIndentation", comment_indentation),
        Box::new(ElseAlignment),
        Box::new(AccessModifierIndentation),
        custom("Layout/MultilineHashBraceLayout", hash_brace_layout),
        Box::new(CaseIndentation),
    ];
    cops.extend(registry::cops());
    cops
}

fn leading_comment_space(context: &mut CopContext<'_, '_>) {
    let shebang_file = context
        .source()
        .lines()
        .next()
        .is_some_and(|line| line.starts_with("#!"));
    let config_ru = context.path().rsplit('/').next() == Some("config.ru");
    let gemfile = context.path().rsplit('/').next() == Some("Gemfile");
    let parsed = parse(context.source().as_bytes());
    for parsed_comment in parsed.comments() {
        if parsed_comment.type_() != ruby_prism::CommentType::InlineComment {
            continue;
        }
        let location = parsed_comment.location();
        let comment = context.source_file().at(&location);
        let content = &comment[1..];
        let hashes = comment.bytes().take_while(|byte| *byte == b'#').count();
        let multiple_hash_comment = hashes > 1
            && (hashes == comment.len() || comment[hashes..].starts_with(char::is_whitespace));
        let first_line = context.source_file().line_start(location.start_offset()) == 0;
        if content.is_empty()
            || content.starts_with([' ', '\t'])
            || multiple_hash_comment
            || content.starts_with('=')
            || content.starts_with("++")
            || content.starts_with("--")
            || shebang_file && content.starts_with('!')
            || first_line && config_ru && content.starts_with('\\')
            || context.config_bool("AllowDoxygenCommentStyle", false) && content.starts_with('*')
            || context.config_bool("AllowGemfileRubyComment", false)
                && gemfile
                && (content.starts_with("ruby=") || content.starts_with("ruby-gemset="))
            || context.config_bool("AllowRBSInlineAnnotation", false)
                && content.starts_with(['[', ':', '|'])
            || context.config_bool("AllowSteepAnnotation", false) && content.starts_with(['$', ':'])
        {
            continue;
        }
        context.insert(
            "Missing space after `#`.",
            location.start_offset()..location.end_offset(),
            location.start_offset() + 1,
            " ",
        );
    }
}

fn array_brace_layout(context: &mut CopContext<'_, '_>) {
    brace_layout(context, '[', ']');
}

fn hash_brace_layout(context: &mut CopContext<'_, '_>) {
    brace_layout(context, '{', '}');
}

fn method_call_brace_layout(context: &mut CopContext<'_, '_>) {
    brace_layout(context, '(', ')');
}

struct MultilineMethodDefinitionBraceLayout;

impl Cop for MultilineMethodDefinitionBraceLayout {
    fn name(&self) -> &'static str {
        "Layout/MultilineMethodDefinitionBraceLayout"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(definition) = node.as_def_node() else {
            return;
        };
        let mut context = context.cop_context(self.name(), source, ancestors);
        check_method_definition_brace_layout(&definition, &mut context);
    }
}

fn check_method_definition_brace_layout(
    definition: &ruby_prism::DefNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let (Some(opening), Some(closing), Some(parameters)) = (
        definition.lparen_loc(),
        definition.rparen_loc(),
        definition.parameters(),
    ) else {
        return;
    };
    let file = context.source_file();
    if file.same_line(opening.start_offset(), closing.start_offset()) {
        return;
    }
    let parameter_location = parameters.location();
    let first_parameter = parameter_location.start_offset();
    let last_parameter = parameter_location.end_offset();
    if first_parameter == last_parameter {
        return;
    }
    let opening_with_first = file.same_line(opening.start_offset(), first_parameter);
    let closing_with_last =
        file.same_line(last_parameter.saturating_sub(1), closing.start_offset());
    let style = context.policy().enforced_style("symmetrical");
    let wants_same_line = match style {
        "same_line" => true,
        "new_line" => false,
        _ => opening_with_first,
    };
    if wants_same_line == closing_with_last {
        return;
    }
    if wants_same_line && closes_immediately_after_heredoc(context.source(), closing.start_offset())
    {
        return;
    }

    let message = match (style, wants_same_line) {
        ("same_line", _) => {
            "Closing method definition brace must be on the same line as the last parameter."
        }
        ("new_line", _) => {
            "Closing method definition brace must be on the line after the last parameter."
        }
        (_, true) => "Closing method definition brace must be on the same line as the last parameter when opening brace is on the same line as the first parameter.",
        (_, false) => "Closing method definition brace must be on the line after the last parameter when opening brace is on a separate line from the first parameter.",
    };
    if wants_same_line {
        let closing_line_start = file.line_start(closing.start_offset());
        let removal_start = closing_line_start.saturating_sub(1);
        context.add_offense(&closing, message, |corrector| {
            corrector.remove(removal_start..closing.end_offset());
            corrector.replace(last_parameter..last_parameter, ")");
        });
    } else {
        let indentation = file.indentation_text(opening.start_offset());
        context.insert(
            message,
            &closing,
            closing.start_offset(),
            format!("\n{indentation}"),
        );
    }
}

fn closes_immediately_after_heredoc(source: &str, closing: usize) -> bool {
    let closing_line = source[..closing]
        .rfind('\n')
        .map_or(0, |newline| newline + 1);
    let preceding = source[..closing_line.saturating_sub(1)]
        .lines()
        .next_back()
        .map(str::trim)
        .unwrap_or_default();
    if preceding.is_empty() {
        return false;
    }
    source[..closing_line]
        .lines()
        .any(|line| line.contains("<<") && line.contains(preceding))
}

fn brace_layout(context: &mut CopContext<'_, '_>, open: char, close: char) {
    let style = context.policy().enforced_style("symmetrical").to_string();
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if !window[0].1.trim_end().ends_with(open) || window[1].1.trim() == close.to_string() {
            continue;
        }
        if matches!(style.as_str(), "symmetrical" | "new_line")
            && lines.iter().any(|(_, line)| {
                line.trim_end().ends_with(close) && line.trim() != close.to_string()
            })
        {
            context.report(
                "Closing brace must be on a new line.",
                window[0].0..window[1].0 + window[1].1.len(),
            );
        }
    }
}

fn align_continuation(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if window[0].1.trim_end().ends_with([',', '+', '\\'])
            && !window[1].1.trim().is_empty()
            && window[1].1.len() - window[1].1.trim_start().len() == 0
        {
            context.insert(
                "Use configured indentation for a continuation line.",
                window[1].0..window[1].0,
                window[1].0,
                "  ",
            );
        }
    }
}

fn line_continuation_spacing(context: &mut CopContext<'_, '_>) {
    let trimmed_source = context.source().trim_start();
    if ["%i", "%I", "%q", "%Q", "%r", "%x", "%W", "%w", "/", "`"]
        .iter()
        .any(|prefix| trimmed_source.starts_with(prefix))
    {
        return;
    }
    let space_style = context.policy().enforced_style("space") == "space";
    let heredoc_ranges = context.source_file().heredoc_ranges();
    for (offset, line) in context.source_file().lines() {
        if line.trim() == "__END__" {
            break;
        }
        if !line.trim_end().ends_with('\\') || line.trim_start().starts_with('#') {
            continue;
        }
        let slash = line.rfind('\\').unwrap_or(0);
        if heredoc_ranges
            .iter()
            .any(|range| range.start <= offset + slash && offset + slash < range.end)
        {
            continue;
        }
        if !context
            .source_file()
            .code_offsets("\\")
            .contains(&(offset + slash))
        {
            continue;
        }
        let spaces = line[..slash].len() - line[..slash].trim_end().len();
        if space_style && spaces != 1 {
            context.replace(
                "Use one space in front of backslash.",
                offset + slash - spaces..offset + slash + 1,
                offset + slash - spaces..offset + slash,
                " ",
            );
        } else if !space_style && spaces > 0 {
            context.remove(
                "Use zero spaces in front of backslash.",
                offset + slash - spaces..offset + slash + 1,
                offset + slash - spaces..offset + slash,
            );
        }
    }
}

fn space_inside_parens(context: &mut CopContext<'_, '_>) {
    let style = context.policy().enforced_style("no_space");
    let no_space = style == "no_space";
    let compact = style == "compact";
    let source = context.source();
    let file = context.source_file();
    let literal_ranges = file.literal_ranges();
    let heredoc_ranges = file.heredoc_ranges();
    let comment_ranges = file.comment_ranges();
    let data_section_start = file.data_section_start();
    let inside_literal = |offset| {
        data_section_start.is_some_and(|start| start <= offset)
            || comment_ranges
                .iter()
                .any(|range| range.start <= offset && offset < range.end)
            || literal_ranges
                .iter()
                .any(|range| range.start <= offset && offset < range.end)
                && !heredoc_ranges.iter().any(|range| {
                    range.start <= offset
                        && offset < range.end
                        && file.same_line(offset, range.start)
                })
    };
    for opening in file.code_offsets("(") {
        if inside_literal(opening) {
            continue;
        }
        let whitespace_end = opening
            + 1
            + source[opening + 1..]
                .bytes()
                .take_while(|byte| matches!(byte, b' ' | b'\t'))
                .count();
        let next = source.as_bytes().get(whitespace_end).copied();
        if source.as_bytes().get(opening + 1) == Some(&b'\n') || next == Some(b'#') {
            continue;
        }
        let whitespace = opening + 1..whitespace_end;
        let unwanted = !whitespace.is_empty()
            && (no_space || next == Some(b')') || compact && next == Some(b'('));
        if unwanted {
            context.remove(
                "Space inside parentheses detected.",
                whitespace.clone(),
                whitespace,
            );
        } else if !no_space
            && whitespace.is_empty()
            && !matches!(next, Some(b')'))
            && !(compact && next == Some(b'('))
        {
            context.insert(
                "No space inside parentheses detected.",
                opening + 1..opening + 2,
                opening + 1,
                " ",
            );
        }
    }
    for closing in file.code_offsets(")") {
        if inside_literal(closing) {
            continue;
        }
        let line_start = file.line_start(closing);
        let whitespace_start = source[line_start..closing]
            .trim_end_matches([' ', '\t'])
            .len()
            + line_start;
        let previous = source
            .as_bytes()
            .get(whitespace_start.wrapping_sub(1))
            .copied();
        if whitespace_start == line_start
            || source.as_bytes().get(closing.wrapping_sub(1)) == Some(&b'\n')
            || previous == Some(b'(')
        {
            continue;
        }
        let whitespace = whitespace_start..closing;
        let unwanted = !whitespace.is_empty() && (no_space || compact && previous == Some(b')'));
        if unwanted {
            context.remove(
                "Space inside parentheses detected.",
                whitespace.clone(),
                whitespace,
            );
        } else if !no_space && whitespace.is_empty() && !(compact && previous == Some(b')')) {
            context.insert(
                "No space inside parentheses detected.",
                closing..closing + 1,
                closing,
                " ",
            );
        }
    }
}

pub(super) struct SpaceInsideBlockBraces;

impl Cop for SpaceInsideBlockBraces {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideBlockBraces"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(block) = node.as_block_node() else {
            return;
        };
        let mut context = context.cop_context(self.name(), source, ancestors);
        check_space_inside_block_braces(&block, &mut context);
    }
}

fn check_space_inside_block_braces(
    block: &ruby_prism::BlockNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let file = context.source_file();
    let opening = block.opening_loc();
    let closing = block.closing_loc();
    if file.at(&opening) != "{" || file.at(&closing) != "}" {
        return;
    }

    let opening_end = opening.end_offset();
    let closing_start = closing.start_offset();
    let Some(contents) = file.slice(opening_end..closing_start) else {
        return;
    };

    if !contents.contains('\n') && contents.trim().is_empty() {
        match context
            .config_value("EnforcedStyleForEmptyBraces")
            .unwrap_or("no_space")
        {
            "no_space" if !contents.is_empty() => context.remove(
                "Space inside empty braces detected.",
                opening_end..closing_start,
                opening_end..closing_start,
            ),
            "space" if contents.is_empty() => context.insert(
                "Space missing inside empty braces.",
                opening.start_offset()..closing.end_offset(),
                opening_end,
                " ",
            ),
            _ => {}
        }
        return;
    }

    let style = context.policy().enforced_style("space");
    let wants_space = style != "no_space";
    let leading_length = contents
        .bytes()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count();
    let trailing_length = contents
        .bytes()
        .rev()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count();
    let first_content = opening_end + leading_length;
    let has_parameters = context.source().as_bytes().get(first_content) == Some(&b'|');

    if has_parameters {
        let space_before_parameters = context.config_bool("SpaceBeforeBlockParameters", true);
        if space_before_parameters && leading_length == 0 {
            context.insert(
                "Space between { and | missing.",
                opening.start_offset()..first_content + 1,
                opening_end,
                " ",
            );
        } else if !space_before_parameters && leading_length > 0 {
            context.remove(
                "Space between { and | detected.",
                opening_end..first_content,
                opening_end..first_content,
            );
        }
    } else if context.source().as_bytes().get(first_content) != Some(&b'\n')
        && context.source().as_bytes().get(first_content) != Some(&b'\r')
        && file.same_line(opening.start_offset(), first_content)
    {
        if wants_space && leading_length == 0 {
            context.insert(
                "Space missing inside {.",
                first_content..first_content + 1,
                opening_end,
                " ",
            );
        } else if !wants_space && leading_length > 0 {
            context.remove(
                "Space inside { detected.",
                opening_end..first_content,
                opening_end..first_content,
            );
        }
    }

    if !file.same_line(opening.start_offset(), closing_start) {
        if !wants_space {
            let opening_indent = file.indentation(opening.start_offset());
            let closing_indent = file.indentation(closing_start);
            if closing_indent.end > closing_indent.start + opening_indent.len() {
                let excess = closing_indent.start + opening_indent.len()..closing_indent.end;
                context.remove("Space inside } detected.", excess.clone(), excess);
            }
        }
        return;
    }

    let last_content = closing_start.saturating_sub(trailing_length);
    let block_delimiters_enabled = context
        .related_config_value("Style/BlockDelimiters", "Enabled")
        .is_some_and(|enabled| enabled == "true");
    if wants_space && (!has_parameters || block_delimiters_enabled) && trailing_length == 0 {
        context.insert(
            "Space missing inside }.",
            closing_start..closing.end_offset(),
            closing_start,
            " ",
        );
    } else if !wants_space && trailing_length > 0 {
        context.remove(
            "Space inside } detected.",
            last_content..closing_start,
            last_content..closing_start,
        );
    }
}

struct ClosingParenthesisIndentation;

impl Cop for ClosingParenthesisIndentation {
    fn name(&self) -> &'static str {
        "Layout/ClosingParenthesisIndentation"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let inspected = if let Some(call) = node.as_call_node() {
            call.opening_loc()
                .zip(call.closing_loc())
                .map(|(left, right)| {
                    let elements = call
                        .arguments()
                        .map(|arguments| arguments.arguments().iter().collect())
                        .unwrap_or_default();
                    (left, right, elements, call.location())
                })
        } else if let Some(parentheses) = node.as_parentheses_node() {
            let elements = parentheses.body().map_or_else(Vec::new, |body| {
                body.as_statements_node().map_or_else(
                    || vec![body],
                    |statements| statements.body().iter().collect(),
                )
            });
            Some((
                parentheses.opening_loc(),
                parentheses.closing_loc(),
                elements,
                parentheses.location(),
            ))
        } else if let Some(definition) = node.as_def_node() {
            definition
                .lparen_loc()
                .zip(definition.rparen_loc())
                .map(|(left, right)| {
                    let elements = definition
                        .parameters()
                        .map(parameter_nodes)
                        .unwrap_or_default();
                    (left, right, elements, definition.location())
                })
        } else {
            None
        };
        let Some((left, right, elements, node_location)) = inspected else {
            return;
        };
        let file = SourceFile::new(source);
        let right_indentation = file.indentation(right.start_offset());
        if right_indentation.end != right.start_offset() {
            return;
        }
        let actual = right_indentation.len();
        let left_column = left.start_offset() - file.line_start(left.start_offset());
        let node_column =
            node_location.start_offset() - file.line_start(node_location.start_offset());
        let mut reporter = context.cop_context(self.name(), source, ancestors);
        let expected = if elements.is_empty() {
            let line_indentation = file.indentation(left.start_offset()).len();
            let candidates = [line_indentation, left_column, node_column];
            if candidates.contains(&actual) {
                return;
            }
            candidates[0]
        } else {
            let first = elements[0].location();
            let first_indentation = file.indentation(first.start_offset()).len();
            if !file.same_line(left.start_offset(), first.start_offset()) {
                first_indentation.saturating_sub(configured_indentation_width(&reporter))
            } else {
                let columns = if let Some(hash) = elements[0].as_keyword_hash_node() {
                    hash.elements()
                        .iter()
                        .map(|element| {
                            let location = element.location();
                            location.start_offset() - file.line_start(location.start_offset())
                        })
                        .collect::<std::collections::HashSet<_>>()
                } else {
                    elements
                        .iter()
                        .map(|element| {
                            let location = element.location();
                            location.start_offset() - file.line_start(location.start_offset())
                        })
                        .collect::<std::collections::HashSet<_>>()
                };
                if columns.len() == 1 {
                    left_column
                } else {
                    first_indentation
                }
            }
        };
        if actual == expected {
            return;
        }
        let message = if expected == left_column {
            "Align `)` with `(`.".to_string()
        } else {
            format!("Indent `)` to column {expected} (not {actual})")
        };
        reporter.replace(message, &right, right_indentation, " ".repeat(expected));
    }
}

fn parameter_nodes<'pr>(parameters: ruby_prism::ParametersNode<'pr>) -> Vec<Node<'pr>> {
    let mut nodes = parameters.requireds().iter().collect::<Vec<_>>();
    nodes.extend(parameters.optionals().iter());
    if let Some(rest) = parameters.rest() {
        nodes.push(rest);
    }
    nodes.extend(parameters.posts().iter());
    nodes.extend(parameters.keywords().iter());
    if let Some(keyword_rest) = parameters.keyword_rest() {
        nodes.push(keyword_rest);
    }
    if let Some(block) = parameters.block() {
        nodes.push(block.as_node());
    }
    nodes.sort_by_key(|node| node.location().start_offset());
    nodes
}

fn configured_indentation_width(context: &CopContext<'_, '_>) -> usize {
    context
        .config_value("IndentationWidth")
        .and_then(|value| value.parse::<usize>().ok())
        .or_else(|| {
            context
                .related_config_value("Layout/IndentationWidth", "Width")
                .and_then(|value| value.parse::<usize>().ok())
        })
        .unwrap_or(2)
}

fn indentation_style(context: &mut CopContext<'_, '_>) {
    if context.source().contains("<<") {
        return;
    }
    let spaces_style = context.policy().enforced_style("spaces") == "spaces";
    let width = context.config_usize("IndentationWidth", 2);
    for (offset, line) in context.source_file().lines() {
        if line.trim() == "__END__" {
            break;
        }
        let indentation = line.len() - line.trim_start_matches([' ', '\t']).len();
        let trimmed = &line[indentation..];
        if indentation == 0
            || trimmed.is_empty()
            || !context
                .source_file()
                .code_offsets(trimmed)
                .contains(&(offset + indentation))
        {
            continue;
        }
        let leading = &line[..indentation];
        if spaces_style && leading.contains('\t') {
            let offense_end = leading.rfind('\t').map_or(indentation, |tab| tab + 1);
            context.replace(
                "Tab detected in indentation.",
                offset..offset + offense_end,
                offset..offset + indentation,
                leading.replace('\t', &" ".repeat(width)),
            );
        } else if !spaces_style && leading.contains(' ') {
            let spaces = leading.bytes().take_while(|byte| *byte == b' ').count();
            let offense_end = if spaces > 0 { spaces } else { indentation };
            context.replace(
                "Space detected in indentation.",
                offset..offset + offense_end,
                offset..offset + indentation,
                "\t".repeat(indentation / width),
            );
        }
    }
}

fn comment_indentation(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let data_index = lines
        .iter()
        .position(|(_, line)| line.trim() == "__END__")
        .unwrap_or(lines.len());
    let width = context.config_usize("IndentationWidth", 2);
    let allow_alignment = context.config_bool("AllowForAlignment", false);
    let outdent_modifiers = context
        .related_config_value("Layout/AccessModifierIndentation", "EnforcedStyle")
        == Some("outdent");

    let comments = lines[..data_index]
        .iter()
        .enumerate()
        .filter_map(|(line_index, (offset, line))| {
            let marker = line.find('#')?;
            Some((
                line_index,
                *offset,
                *line,
                marker,
                line[..marker].trim().is_empty(),
            ))
        })
        .collect::<Vec<_>>();

    for (comment_index, &(line_index, offset, line, column, own_line)) in
        comments.iter().enumerate()
    {
        if !own_line {
            continue;
        }
        let next = lines[line_index + 1..data_index]
            .iter()
            .map(|(_, line)| *line)
            .find(|line| !line.trim().is_empty());
        let expected = next.map_or(0, |line| {
            expected_comment_indentation(line, width, outdent_modifiers)
        });
        if column == expected {
            continue;
        }
        let two_alternatives = next.is_some_and(|next_line| {
            ["else", "elsif", "when", "in", "rescue", "ensure"]
                .iter()
                .any(|keyword| {
                    let trimmed = next_line.trim_start();
                    trimmed == *keyword || trimmed.starts_with(&format!("{keyword} "))
                })
        });
        let message_expected = if two_alternatives {
            expected + width
        } else {
            expected
        };
        if two_alternatives && column == message_expected {
            continue;
        }
        if allow_alignment
            && comments[..comment_index]
                .iter()
                .rev()
                .find(|(_, _, _, _, preceding_own_line)| !preceding_own_line)
                .is_some_and(|(_, _, _, preceding, _)| *preceding == column)
        {
            continue;
        }

        let message = format!(
            "Incorrect indentation detected (column {column} instead of {message_expected})."
        );
        let correction_expected = lines[line_index + 1..data_index]
            .iter()
            .map(|(_, line)| *line)
            .find(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
            .map_or(0, |line| {
                expected_comment_indentation(line, width, outdent_modifiers)
            });
        let offense = offset + column..offset + line.len();
        let mut edits = vec![(offset..offset + column, " ".repeat(correction_expected))];
        for &(_, preceding_offset, _, preceding_column, preceding_own_line) in
            comments[..comment_index].iter().rev()
        {
            if !preceding_own_line
                || preceding_column != column
                || context.source()[preceding_offset..offset]
                    .matches('\n')
                    .count()
                    != edits.len()
            {
                break;
            }
            edits.push((
                preceding_offset..preceding_offset + preceding_column,
                " ".repeat(correction_expected),
            ));
        }
        context.replace_many(message, offense, edits);
    }
}

fn expected_comment_indentation(line: &str, width: usize, outdent_modifiers: bool) -> usize {
    let indentation = line.len() - line.trim_start().len();
    let trimmed = line.trim_start();
    let less_indented = trimmed.starts_with("end")
        || trimmed.starts_with([')', '}', ']'])
        || (outdent_modifiers
            && ["private", "protected", "public"].iter().any(|modifier| {
                trimmed == *modifier || trimmed.starts_with(&format!("{modifier} "))
            }));
    indentation + usize::from(less_indented) * width
}

struct ElseAlignment;

impl Cop for ElseAlignment {
    fn name(&self) -> &'static str {
        "Layout/ElseAlignment"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let file = SourceFile::new(source);
        let mut reporter = context.cop_context(self.name(), source, ancestors);
        if let Some(condition) = node.as_if_node() {
            let Some(base) = condition.if_keyword_loc() else {
                return;
            };
            if file.at(&base) == "elsif" {
                return;
            }
            let mut subsequent = condition.subsequent();
            while let Some(branch) = subsequent {
                if let Some(elsif) = branch.as_if_node() {
                    let Some(keyword) = elsif.if_keyword_loc() else {
                        break;
                    };
                    check_else_alignment(&base, &keyword, &file, &mut reporter);
                    subsequent = elsif.subsequent();
                } else if let Some(else_node) = branch.as_else_node() {
                    check_else_alignment(
                        &base,
                        &else_node.else_keyword_loc(),
                        &file,
                        &mut reporter,
                    );
                    break;
                } else {
                    break;
                }
            }
        } else if let Some(condition) = node.as_unless_node() {
            if let Some(else_node) = condition.else_clause() {
                check_else_alignment(
                    &condition.keyword_loc(),
                    &else_node.else_keyword_loc(),
                    &file,
                    &mut reporter,
                );
            }
        } else if let Some(case_node) = node.as_case_node() {
            if let (Some(branch), Some(else_node)) =
                (case_node.conditions().last(), case_node.else_clause())
            {
                if let Some(when_node) = branch.as_when_node() {
                    check_else_alignment(
                        &when_node.keyword_loc(),
                        &else_node.else_keyword_loc(),
                        &file,
                        &mut reporter,
                    );
                }
            }
        } else if let Some(case_node) = node.as_case_match_node() {
            if let (Some(branch), Some(else_node)) =
                (case_node.conditions().last(), case_node.else_clause())
            {
                if let Some(in_node) = branch.as_in_node() {
                    check_else_alignment(
                        &in_node.in_loc(),
                        &else_node.else_keyword_loc(),
                        &file,
                        &mut reporter,
                    );
                }
            }
        } else if let Some(begin_node) = node.as_begin_node() {
            let (Some(else_node), Some(rescue)) =
                (begin_node.else_clause(), begin_node.rescue_clause())
            else {
                return;
            };
            let base = begin_node.begin_keyword_loc().unwrap_or_else(|| {
                ancestors
                    .iter()
                    .rev()
                    .find_map(|ancestor| {
                        let assignment = ancestor.as_local_variable_write_node().is_some()
                            || ancestor.as_instance_variable_write_node().is_some()
                            || ancestor.as_class_variable_write_node().is_some()
                            || ancestor.as_global_variable_write_node().is_some()
                            || ancestor.as_constant_write_node().is_some();
                        assignment
                            .then(|| ancestor.location())
                            .or_else(|| ancestor.as_call_node().map(|call| call.location()))
                            .or_else(|| ancestor.as_def_node().map(|def| def.def_keyword_loc()))
                    })
                    .unwrap_or_else(|| rescue.keyword_loc())
            });
            check_else_alignment(&base, &else_node.else_keyword_loc(), &file, &mut reporter);
        }
    }
}

fn check_else_alignment(
    base: &ruby_prism::Location<'_>,
    branch: &ruby_prism::Location<'_>,
    file: &SourceFile<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let indentation = file.indentation(branch.start_offset());
    if indentation.end != branch.start_offset() {
        return;
    }
    let base_line = file.line_start(base.start_offset());
    let bom_width = usize::from(base_line == 0 && file.as_str().starts_with('\u{feff}')) * 3;
    let base_prefix = &file.as_str()[base_line..base.start_offset()];
    let variable_alignment = context
        .related_config_value("Layout/EndAlignment", "EnforcedStyleAlignWith")
        == Some("variable")
        && base_prefix.contains('=');
    let modifier_definition = file.at(base) == "def" && !base_prefix.trim().is_empty();
    let expected = if variable_alignment || modifier_definition {
        file.indentation(base.start_offset()).len()
    } else {
        (base.start_offset() - base_line).saturating_sub(bom_width)
    };
    if indentation.len() == expected {
        return;
    }
    let base_name = if variable_alignment {
        base_prefix
            .split_once('=')
            .map_or(base_prefix, |(variable, _)| variable)
            .trim()
    } else if modifier_definition {
        base_prefix.split_whitespace().next().unwrap_or_default()
    } else {
        file.at(base).split_whitespace().next().unwrap_or_default()
    };
    context.replace(
        format!("Align `{}` with `{base_name}`.", file.at(branch)),
        branch,
        indentation,
        " ".repeat(expected),
    );
}

struct AccessModifierIndentation;

impl Cop for AccessModifierIndentation {
    fn name(&self) -> &'static str {
        "Layout/AccessModifierIndentation"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        let name = call.name().as_slice();
        if !matches!(
            name,
            b"private" | b"protected" | b"public" | b"module_function"
        ) || call.receiver().is_some()
            || call.arguments().is_some()
        {
            return;
        }
        let Some(container) = ancestors.iter().rev().find(|ancestor| {
            ancestor.as_class_node().is_some()
                || ancestor.as_module_node().is_some()
                || ancestor.as_singleton_class_node().is_some()
                || ancestor.as_block_node().is_some()
        }) else {
            return;
        };
        let location = call.location();
        let file = SourceFile::new(source);
        if file.same_line(container.location().start_offset(), location.start_offset()) {
            return;
        }
        let base = file.indentation(container.location().start_offset()).len();
        let mut reporter = context.cop_context(self.name(), source, ancestors);
        let style = reporter.policy().enforced_style("indent").to_string();
        let configured_width = reporter
            .config_value("IndentationWidth")
            .and_then(|value| value.parse::<usize>().ok())
            .or_else(|| {
                reporter
                    .related_config_value("Layout/IndentationWidth", "Width")
                    .and_then(|value| value.parse::<usize>().ok())
            })
            .unwrap_or(2);
        let expected = base
            + if style == "outdent" {
                0
            } else {
                configured_width
            };
        let indentation = file.indentation(location.start_offset());
        let actual = indentation.len();
        if actual == expected {
            return;
        }
        let display = String::from_utf8_lossy(name);
        reporter.replace(
            format!(
                "{} access modifiers like `{display}`.",
                if style == "outdent" {
                    "Outdent"
                } else {
                    "Indent"
                }
            ),
            &location,
            indentation,
            " ".repeat(expected),
        );
    }
}

struct CaseIndentation;

impl Cop for CaseIndentation {
    fn name(&self) -> &'static str {
        "Layout/CaseIndentation"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let (case_keyword, end_keyword, conditions) = if let Some(case_node) = node.as_case_node() {
            (
                case_node.case_keyword_loc(),
                case_node.end_keyword_loc(),
                case_node.conditions().iter().collect::<Vec<_>>(),
            )
        } else if let Some(case_node) = node.as_case_match_node() {
            (
                case_node.case_keyword_loc(),
                case_node.end_keyword_loc(),
                case_node.conditions().iter().collect::<Vec<_>>(),
            )
        } else {
            return;
        };
        let file = SourceFile::new(source);
        if file.same_line(case_keyword.start_offset(), end_keyword.start_offset()) {
            return;
        }
        let mut reporter = context.cop_context(self.name(), source, ancestors);
        let style = reporter.policy().enforced_style("case").to_string();
        let width = if reporter.config_bool("IndentOneStep", false) {
            reporter.config_usize("IndentationWidth", 2)
        } else {
            0
        };
        let base = if style == "end" {
            file.indentation(end_keyword.start_offset()).len()
        } else {
            case_keyword.start_offset() - file.line_start(case_keyword.start_offset())
        };
        for condition in conditions {
            let (keyword, branch) = if let Some(when_node) = condition.as_when_node() {
                (when_node.keyword_loc(), "when")
            } else if let Some(in_node) = condition.as_in_node() {
                (in_node.in_loc(), "in")
            } else {
                continue;
            };
            let actual = keyword.start_offset() - file.line_start(keyword.start_offset());
            let expected = base + width;
            if actual == expected {
                continue;
            }
            let depth = if width == 0 {
                "as deep as"
            } else {
                "one step more than"
            };
            let message = format!("Indent `{branch}` {depth} `{style}`.");
            if file.same_line(case_keyword.start_offset(), keyword.start_offset()) {
                reporter.report(message, &keyword);
            } else {
                reporter.replace(
                    message,
                    &keyword,
                    file.indentation(keyword.start_offset()),
                    " ".repeat(expected),
                );
            }
        }
    }
}

struct EmptyLineAfterGuardClause;

impl Cop for EmptyLineAfterGuardClause {
    fn name(&self) -> &'static str {
        "Layout/EmptyLineAfterGuardClause"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(guard) = guard_conditional(node) else {
            return;
        };
        let Some(next) = right_sibling(node, ancestors) else {
            return;
        };
        if guard_conditional(&next).is_some() {
            return;
        }
        let file = SourceFile::new(source);
        if file.same_line(node.location().end_offset(), next.location().start_offset()) {
            return;
        }

        let heredoc = guard_heredoc_terminator(node, source);
        let effective_end = heredoc.as_ref().map_or_else(
            || {
                file.line_range(node.location().end_offset().saturating_sub(1))
                    .end
            },
            |(_, line_end)| *line_end,
        );
        if effective_end >= source.len() {
            return;
        }
        let next_line = file.line(effective_end);
        if next_line.trim().is_empty() {
            return;
        }
        let insertion = if allowed_guard_directive(next_line) {
            let directive_end = file.line_range(effective_end).end;
            if directive_end >= source.len() || file.line(directive_end).trim().is_empty() {
                return;
            }
            directive_end
        } else {
            effective_end
        };
        let offense = heredoc.map(|(range, _)| range).unwrap_or_else(|| {
            guard.1.map_or_else(
                || node.location().start_offset()..node.location().end_offset(),
                |location| location.start_offset()..location.end_offset(),
            )
        });
        let mut reporter = context.cop_context(self.name(), source, ancestors);
        reporter.insert(
            "Add empty line after guard clause.",
            offense,
            insertion,
            "\n",
        );
    }
}

fn guard_conditional<'pr>(
    node: &Node<'pr>,
) -> Option<(Node<'pr>, Option<ruby_prism::Location<'pr>>)> {
    if let Some(condition) = node.as_if_node() {
        let statements = condition.statements()?;
        if statements.body().len() != 1 {
            return None;
        }
        let branch = statements.body().first()?;
        if is_guard_statement(&branch) {
            return Some((branch, condition.end_keyword_loc()));
        }
    } else if let Some(condition) = node.as_unless_node() {
        let statements = condition.statements()?;
        if statements.body().len() != 1 {
            return None;
        }
        let branch = statements.body().first()?;
        if is_guard_statement(&branch) {
            return Some((branch, condition.end_keyword_loc()));
        }
    }
    None
}

fn is_guard_statement(node: &Node<'_>) -> bool {
    node.as_return_node().is_some()
        || node.as_break_node().is_some()
        || node.as_next_node().is_some()
        || node
            .as_and_node()
            .is_some_and(|conjunction| is_guard_statement(&conjunction.right()))
        || node
            .as_or_node()
            .is_some_and(|conjunction| is_guard_statement(&conjunction.right()))
        || node.as_call_node().is_some_and(|call| {
            call.receiver().is_none() && matches!(call.name().as_slice(), b"raise" | b"fail")
        })
}

fn right_sibling<'pr>(node: &Node<'pr>, ancestors: &[Node<'pr>]) -> Option<Node<'pr>> {
    ancestors.iter().rev().find_map(|ancestor| {
        let statements = if let Some(program) = ancestor.as_program_node() {
            Some(program.statements())
        } else if let Some(definition) = ancestor.as_def_node() {
            definition.body().and_then(|body| body.as_statements_node())
        } else if let Some(class) = ancestor.as_class_node() {
            class.body().and_then(|body| body.as_statements_node())
        } else if let Some(module) = ancestor.as_module_node() {
            module.body().and_then(|body| body.as_statements_node())
        } else if let Some(singleton) = ancestor.as_singleton_class_node() {
            singleton.body().and_then(|body| body.as_statements_node())
        } else if let Some(block) = ancestor.as_block_node() {
            block.body().and_then(|body| body.as_statements_node())
        } else if let Some(begin) = ancestor.as_begin_node() {
            begin.statements()
        } else if let Some(rescue) = ancestor.as_rescue_node() {
            rescue.statements()
        } else {
            ancestor.as_statements_node()
        }?;
        let mut found = false;
        for sibling in statements.body().iter() {
            if found {
                return Some(sibling);
            }
            found = sibling.location().start_offset() == node.location().start_offset()
                && sibling.location().end_offset() == node.location().end_offset();
        }
        None
    })
}

fn guard_heredoc_terminator(
    node: &Node<'_>,
    source: &str,
) -> Option<(std::ops::Range<usize>, usize)> {
    let node_source = source.get(node.location().start_offset()..node.location().end_offset())?;
    let marker = guard_heredoc_marker(node_source)?;
    let file = SourceFile::new(source);
    file.lines()
        .find(|(offset, line)| *offset >= node.location().end_offset() && line.trim() == marker)
        .map(|(offset, line)| {
            let content_end = offset + line.len();
            (offset..content_end, file.line_range(offset).end)
        })
}

fn guard_heredoc_marker(source: &str) -> Option<&str> {
    source
        .match_indices("<<")
        .filter_map(|(offset, _)| {
            let mut rest = &source[offset + 2..];
            rest = rest.strip_prefix(['~', '-']).unwrap_or(rest);
            if rest.starts_with(['\'', '"', '`']) {
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

fn allowed_guard_directive(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.starts_with("# rubocop:enable") {
        return true;
    }
    let Some(comment) = trimmed.strip_prefix('#') else {
        return false;
    };
    let comment = comment.trim_start();
    if comment.starts_with(":nocov:") {
        return true;
    }
    let Some(rest) = comment.strip_prefix("simplecov") else {
        return false;
    };
    let rest = rest.trim_start();
    let Some(rest) = rest.strip_prefix(':') else {
        return false;
    };
    matches!(
        rest.trim_start().split_whitespace().next(),
        Some("disable" | "enable")
    )
}
