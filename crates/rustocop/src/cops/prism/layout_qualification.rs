use super::*;

define_cops! {
    ArrayAlignment => "Layout/ArrayAlignment" => any_node(array_alignment),
    MultilineAssignmentLayout => "Layout/MultilineAssignmentLayout" => source(multiline_assignment_layout),
    EndAlignment => "Layout/EndAlignment" => any_node(end_alignment),
    ExtraSpacing => "Layout/ExtraSpacing" => source(extra_spacing),
    FirstHashElementIndentation => "Layout/FirstHashElementIndentation" => node(as_hash_node, first_hash_element_indentation),
}

fn multiline_assignment_layout(context: &mut CopContext<'_, '_>) {
    use crate::rubocop::ast::prism::convert as convert_rubocop_ast;
    use crate::rubocop::cop::mixin::check_assignment::extract_rhs;

    let source = context.source().to_string();
    let supported = context.config_values("SupportedTypes").to_vec();
    let style = context.policy().enforced_style("new_line").to_string();
    let parsed = ruby_prism::parse(source.as_bytes());
    let (ast, root) = convert_rubocop_ast(&source, &parsed.node());
    let Some(root) = root.map(|root| ast.node(root)) else {
        return;
    };
    let mut corrections = Vec::new();

    for assignment in root.each_node(&[
        "lvasgn", "ivasgn", "cvasgn", "gvasgn", "casgn", "masgn", "op_asgn", "or_asgn",
        "and_asgn", "send",
    ]) {
        let Some(rhs) = extract_rhs(assignment) else {
            continue;
        };
        if assignment.kind() == "send"
            && !assignment.method_name().is_some_and(|name| {
                name.ends_with('=')
                    && !matches!(name, "==" | "===" | "!=" | "=~" | "!~" | "<=" | ">=")
            })
        {
            continue;
        }
        let Some(assignment_chars) = assignment.source_range() else {
            continue;
        };
        let Some(rhs_chars) = rhs.source_range() else {
            continue;
        };
        let Some(operator_chars) = multiline_assignment_operator(
            &source,
            assignment_chars.clone(),
            rhs_chars.clone(),
            assignment.loc("operator").map(|(range, _)| range.clone()),
        ) else {
            continue;
        };
        let operator_source = source
            .chars()
            .skip(operator_chars.start)
            .take(operator_chars.end - operator_chars.start)
            .collect::<String>();
        if assignment.kind() == "send" && operator_source != "=" {
            continue;
        }

        let block_family = matches!(rhs.kind(), "block" | "numblock" | "itblock");
        let supported_rhs = if block_family {
            supported.iter().any(|kind| kind == "block")
        } else {
            supported.iter().any(|kind| kind == rhs.kind())
        };
        if !supported_rhs {
            continue;
        }

        let block_begins_on_assignment_line = rhs.loc("begin").is_some_and(|(begin, _)| {
            multiline_assignment_line_at(&source, begin.start) == assignment.first_line()
        });
        if rhs.single_line() && (rhs.kind() != "block" || block_begins_on_assignment_line) {
            continue;
        }

        let same_line =
            multiline_assignment_line_at(&source, operator_chars.start) == rhs.first_line();
        let (message, edit_chars, replacement) = if style == "new_line" && same_line {
            (
                "Right hand side of multi-line assignment is on the same line as the assignment operator `=`.",
                operator_chars.end..operator_chars.end,
                "\n",
            )
        } else if style == "same_line" && !same_line {
            (
                "Right hand side of multi-line assignment is not on the same line as the assignment operator `=`.",
                operator_chars.end..rhs_chars.start,
                " ",
            )
        } else {
            continue;
        };
        corrections.push((
            message,
            multiline_assignment_character_range_to_byte(&source, assignment_chars),
            multiline_assignment_character_range_to_byte(&source, edit_chars),
            replacement,
        ));
    }

    for (message, offense, edit, replacement) in corrections {
        context.replace(message, offense, edit, replacement);
    }
}

fn multiline_assignment_operator(
    source: &str,
    assignment: std::ops::Range<usize>,
    rhs: std::ops::Range<usize>,
    translated: Option<std::ops::Range<usize>>,
) -> Option<std::ops::Range<usize>> {
    if let Some(translated) = translated {
        let translated_source = source
            .chars()
            .skip(translated.start)
            .take(translated.end - translated.start)
            .collect::<String>();
        if translated_source.ends_with('=') {
            return Some(translated);
        }
    }
    let prefix = source
        .chars()
        .skip(assignment.start)
        .take(rhs.start.saturating_sub(assignment.start))
        .collect::<String>();
    let equal_byte = prefix.rfind('=')?;
    let equal = prefix[..equal_byte].chars().count();
    let mut start = equal;
    let characters = prefix.chars().collect::<Vec<_>>();
    while start > 0 && "&|+-*/%^<>".contains(characters[start - 1]) {
        start -= 1;
    }
    Some(assignment.start + start..assignment.start + equal + 1)
}

fn multiline_assignment_line_at(source: &str, character_offset: usize) -> usize {
    source
        .chars()
        .take(character_offset)
        .filter(|character| *character == '\n')
        .count()
        + 1
}

fn multiline_assignment_character_range_to_byte(
    source: &str,
    range: std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    let byte_at = |offset| {
        source
            .char_indices()
            .nth(offset)
            .map_or(source.len(), |(byte, _)| byte)
    };
    byte_at(range.start)..byte_at(range.end)
}

fn array_alignment(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(array) = node.as_array_node() {
        if context
            .parent()
            .is_some_and(|parent| parent.as_multi_write_node().is_some())
            || array.elements().len() < 2
        {
            return;
        }
        let elements = array.elements().iter().collect::<Vec<_>>();
        align_array_elements(
            &elements,
            array.opening_loc(),
            array.location().start_offset(),
            Some(&array),
            context,
        );
    } else if let Some(rescue) = node.as_rescue_node() {
        let elements = rescue.exceptions().iter().collect::<Vec<_>>();
        if elements.len() >= 2 {
            align_array_elements(
                &elements,
                None,
                rescue.location().start_offset(),
                None,
                context,
            );
        }
    }
}

#[allow(clippy::too_many_lines)]
fn align_array_elements(
    elements: &[Node<'_>],
    opening: Option<ruby_prism::Location<'_>>,
    container_offset: usize,
    array: Option<&ruby_prism::ArrayNode<'_>>,
    context: &mut CopContext<'_, '_>,
) {
    let first = &elements[0];
    let file = context.source_file();
    let style = context.policy().enforced_style("with_first_element");
    let base = if style == "with_fixed_indentation" {
        let container_start = opening.as_ref().map_or_else(
            || {
                context
                    .parent()
                    .map_or(container_offset, |parent| parent.location().start_offset())
            },
            |opening| opening.start_offset(),
        );
        let line_start = file.line_range(container_start).start;
        let indentation = context.source()[line_start..container_start]
            .chars()
            .take_while(|character| character.is_whitespace())
            .count();
        indentation
            + context
                .related_config_value("Layout/IndentationWidth", "Width")
                .and_then(|width| width.parse::<usize>().ok())
                .unwrap_or(2)
    } else {
        let line_start = file.line_range(first.location().start_offset()).start;
        unicode_width::UnicodeWidthStr::width(
            &context.source()[line_start..first.location().start_offset()],
        )
    };
    let message = if style == "with_fixed_indentation" {
        "Use one level of indentation for elements following the first line of a multi-line array."
    } else {
        "Align the elements of an array literal if they span more than one line."
    };
    let ancestor_array = context
        .ancestors()
        .iter()
        .rev()
        .find_map(Node::as_array_node);
    let nested_correction_conflict = array.is_some() && ancestor_array.is_some_and(|ancestor| {
        let Some(position) = ancestor.elements().iter().position(|element| {
            element.location().start_offset() == container_offset
        }) else {
            return false;
        };
        if position == 0 {
            return false;
        }
        let ancestor_elements = ancestor.elements().iter().collect::<Vec<_>>();
        let ancestor_first = &ancestor_elements[0];
        let expected = if style == "with_fixed_indentation" {
            let start = ancestor.opening_loc().map_or_else(
                || ancestor.location().start_offset(),
                |opening| opening.start_offset(),
            );
            let line_start = file.line_range(start).start;
            context.source()[line_start..start]
                .chars()
                .take_while(|character| character.is_whitespace())
                .count()
                + 2
        } else {
            let line_start = file
                .line_range(ancestor_first.location().start_offset())
                .start;
            unicode_width::UnicodeWidthStr::width(
                &context.source()[line_start..ancestor_first.location().start_offset()],
            )
        };
        let start = container_offset;
        let line_start = file.line_range(start).start;
        context.source()[line_start..start].chars().count() != expected
    });

    let bracketed = opening.is_some();
    let mut previous_line = if bracketed {
        file.line_start(first.location().end_offset().saturating_sub(1))
    } else {
        usize::MAX
    };
    for element in elements.iter().skip(usize::from(bracketed)) {
        let location = element.location();
        let line = file.line_range(location.start_offset());
        if line.start == previous_line {
            previous_line = file.line_start(location.end_offset().saturating_sub(1));
            continue;
        }
        previous_line = file.line_start(location.end_offset().saturating_sub(1));
        let actual = context.source()[line.start..location.start_offset()]
            .chars()
            .count();
        if actual == base {
            continue;
        }
        if nested_correction_conflict {
            context.report(message, &location);
        } else {
            let delta = base as isize - actual as isize;
            let mut edits = Vec::new();
            let first_line_start = line.start;
            let last_line_start = file.line_start(location.end_offset().saturating_sub(1));
            let mut heredoc_marker: Option<String> = None;
            for (line_start, content) in file
                .lines()
                .filter(|(start, _)| first_line_start <= *start && *start <= last_line_start)
            {
                if let Some(marker) = heredoc_marker.as_deref() {
                    if content.trim() == marker {
                        heredoc_marker = None;
                    }
                    continue;
                }
                let indentation = content.len() - content.trim_start().len();
                if !content.trim().is_empty() {
                    let adjusted = (indentation as isize + delta).max(0) as usize;
                    edits.push((line_start..line_start + indentation, " ".repeat(adjusted)));
                }
                if let Some((_, tail)) = content.split_once("<<") {
                    let marker = tail
                        .trim_start_matches(['~', '-', '`'])
                        .split(|character: char| {
                            !character.is_ascii_alphanumeric() && character != '_'
                        })
                        .next()
                        .unwrap_or_default();
                    if !marker.is_empty() {
                        heredoc_marker = Some(marker.to_string());
                    }
                }
            }
            context.replace_many(message, &location, edits);
        }
    }
}

fn end_alignment(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let candidate = if let Some(value) = node.as_class_node() {
        Some((value.class_keyword_loc(), value.end_keyword_loc(), false))
    } else if let Some(value) = node.as_singleton_class_node() {
        Some((value.class_keyword_loc(), value.end_keyword_loc(), true))
    } else if let Some(value) = node.as_module_node() {
        Some((value.module_keyword_loc(), value.end_keyword_loc(), false))
    } else if let Some(value) = node.as_if_node() {
        let (Some(keyword), Some(closing)) = (value.if_keyword_loc(), value.end_keyword_loc())
        else {
            return;
        };
        if source.get(keyword.start_offset()..keyword.end_offset()) != Some("if") {
            return;
        }
        Some((keyword, closing, true))
    } else if let Some(value) = node.as_unless_node() {
        let Some(closing) = value.end_keyword_loc() else {
            return;
        };
        Some((value.keyword_loc(), closing, true))
    } else if let Some(value) = node.as_while_node() {
        let Some(closing) = value.closing_loc() else {
            return;
        };
        Some((value.keyword_loc(), closing, true))
    } else if let Some(value) = node.as_until_node() {
        let Some(closing) = value.closing_loc() else {
            return;
        };
        Some((value.keyword_loc(), closing, true))
    } else if let Some(value) = node.as_case_node() {
        Some((value.case_keyword_loc(), value.end_keyword_loc(), true))
    } else { node.as_case_match_node().map(|value| (value.case_keyword_loc(), value.end_keyword_loc(), true)) };
    let Some((keyword, closing, variable_may_use_outer_expression)) = candidate else {
        return;
    };

    let file = context.source_file();
    let keyword_start = keyword.start_offset();
    let keyword_end = keyword.end_offset();
    let closing_start = closing.start_offset();
    let closing_end = closing.end_offset();
    if file.same_line(keyword_start, closing_start) {
        return;
    }
    let keyword_line_end = file.line_end(keyword_start);
    let bom = usize::from(source.starts_with('\u{feff}') && file.line_start(keyword_start) == 0);
    let first_code = if bom == 1 {
        '\u{feff}'.len_utf8()
    } else {
        file.indentation(keyword_start).end
    };
    let keyword_column = file.column(keyword_start).saturating_sub(bom);
    let actual_column = file.column(closing_start);
    let style = context
        .config_value("EnforcedStyleAlignWith")
        .unwrap_or("keyword");

    let (expected_column, reference) = match style {
        "start_of_line" => (
            file.column(first_code),
            source[first_code..keyword_line_end].trim_end().to_string(),
        ),
        "variable" if variable_may_use_outer_expression => {
            let prefix = &source[first_code..keyword_start];
            if !prefix.trim().is_empty() && !prefix.contains(';') {
                (
                    file.column(first_code),
                    source[first_code..keyword_end].trim_end().to_string(),
                )
            } else {
                (
                    keyword_column,
                    source[keyword_start..keyword_end].to_string(),
                )
            }
        }
        _ => (
            keyword_column,
            source[keyword_start..keyword_end].to_string(),
        ),
    };
    if actual_column == expected_column {
        return;
    }

    let keyword_line = source[..keyword_start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let closing_line = source[..closing_start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let closing_line_start = file.line_start(closing_start);
    let preceding = &source[closing_line_start..closing_start];
    let (edit, replacement) = if preceding.trim().is_empty() {
        (
            closing_line_start..closing_start,
            " ".repeat(expected_column),
        )
    } else {
        (
            closing_start..closing_start,
            format!("\n{}", " ".repeat(expected_column)),
        )
    };
    context.replace(
        format!(
            "`end` at {closing_line}, {actual_column} is not aligned with `{reference}` at {keyword_line}, {expected_column}."
        ),
        closing_start..closing_end,
        edit,
        replacement,
    );
}

#[derive(Clone, Debug)]
struct SpacingToken {
    start: usize,
    end: usize,
    line: usize,
    column: usize,
    text: String,
    kind: SpacingTokenKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SpacingTokenKind {
    Comment,
    String,
    Word,
    Operator,
    Punctuation,
}

#[allow(clippy::too_many_lines)]
fn extra_spacing(_context: &mut CopContext<'_, '_>) {
    let context = _context;
    let source = context.source();
    if source.trim().is_empty() {
        return;
    }
    let syntax_end = source
        .match_indices("__END__")
        .find(|(offset, _)| {
            (*offset == 0 || source.as_bytes().get(offset - 1) == Some(&b'\n'))
                && source.as_bytes().get(offset + 7).is_none_or(|byte| *byte == b'\n')
        })
        .map_or(source.len(), |(offset, _)| offset);
    let tokens = spacing_tokens(&source[..syntax_end]);
    let lines = context
        .source_file()
        .lines()
        .map(|(_, line)| line)
        .collect::<Vec<_>>();
    let ignored_hash_ranges = multiline_hash_pair_ranges(source);
    let literal_ranges = context
        .source_file()
        .literal_ranges()
        .into_iter()
        .collect::<Vec<_>>();
    let allow_alignment = context.config_bool("AllowForAlignment", true);
    let allow_trailing_comments = context.config_bool("AllowBeforeTrailingComments", false);
    let force_equals = context.config_explicit("ForceEqualSignAlignment")
        && context.config_bool("ForceEqualSignAlignment", false);
    let assignment_tokens = assignment_token_indices(&tokens, &lines);
    let assignment_starts = assignment_tokens
        .iter()
        .map(|index| tokens[*index].start)
        .collect::<std::collections::HashSet<_>>();

    let mut aligned_comment_lines = std::collections::HashSet::new();
    let mut full_line_comment_lines = std::collections::HashSet::new();
    let comments = context
        .source_file()
        .comment_ranges()
        .into_iter()
        .map(|range| {
            let line = source[..range.start].bytes().filter(|byte| *byte == b'\n').count();
            let line_start = source[..range.start].rfind('\n').map_or(0, |at| at + 1);
            (line, range.start - line_start)
        })
        .collect::<Vec<_>>();
    for pair in comments.windows(2) {
        if pair[0].1 == pair[1].1 {
            aligned_comment_lines.insert(pair[0].0);
            aligned_comment_lines.insert(pair[1].0);
        }
    }
    for range in context.source_file().comment_ranges() {
        let line = source[..range.start].bytes().filter(|byte| *byte == b'\n').count();
        let line_start = source[..range.start].rfind('\n').map_or(0, |at| at + 1);
        if source[line_start..range.start].trim().is_empty() {
            full_line_comment_lines.insert(line);
        }
    }

    let mut ordinary_offenses = Vec::new();
    for pair in tokens.windows(2) {
        let (left, right) = (&pair[0], &pair[1]);
        if literal_ranges
            .iter()
            .any(|range| range.start <= left.end && right.start <= range.end)
        {
            continue;
        }
        if left.line != right.line || right.start.saturating_sub(left.end) <= 1 {
            continue;
        }
        if right.kind == SpacingTokenKind::Comment {
            continue;
        }
        if force_equals && assignment_starts.contains(&right.start) {
            continue;
        }
        if allow_trailing_comments && right.kind == SpacingTokenKind::Comment {
            continue;
        }
        if ignored_hash_ranges
            .iter()
            .any(|range| range.start <= left.end && left.end < range.end)
        {
            continue;
        }
        if allow_alignment
            && aligned_spacing_token(&lines, &tokens, right, &full_line_comment_lines)
        {
            continue;
        }

        ordinary_offenses.push(left.end..right.start - 1);
    }
    if !allow_trailing_comments {
        for range in context.source_file().comment_ranges() {
            let line_number = source[..range.start]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count();
            if allow_alignment && aligned_comment_lines.contains(&line_number) {
                continue;
            }
            let line_start = source[..range.start].rfind('\n').map_or(0, |at| at + 1);
            let whitespace_start = source[line_start..range.start]
                .trim_end_matches([' ', '\t'])
                .len()
                + line_start;
            if source[line_start..whitespace_start].trim().is_empty()
                || range.start.saturating_sub(whitespace_start) <= 1
            {
                continue;
            }
            ordinary_offenses.push(whitespace_start..range.start - 1);
        }
    }
    let mut ordinary_edits = ordinary_offenses
        .iter()
        .cloned()
        .map(|range| (range, String::new()))
        .collect::<Vec<_>>();
    ordinary_edits.sort_by_key(|(range, _)| (range.start, range.end));
    ordinary_edits.dedup_by(|left, right| left.0 == right.0);
    for offense in ordinary_offenses {
        context.replace_many(
            "Unnecessary spacing detected.",
            offense,
            ordinary_edits.clone(),
        );
    }

    if force_equals {
        check_forced_equal_alignment(context, &tokens, &lines, &assignment_tokens);
    }
}

fn spacing_tokens(source: &str) -> Vec<SpacingToken> {
    const OPERATORS: &[&str] = &[
        "&&=", "||=", "<<=", ">>=", "**=", "===", "<=>", "==", "!=", "<=", ">=", "+=", "-=", "*=",
        "/=", "%=", "&=", "|=", "^=", "=>", "=~", "!~", "<<", ">>", "&&", "||", "&.", "::", "**",
        "..", "...", "=", "+", "-", "*", "/", "%", "<", ">", "&", "|", "^", "!", "~", "?", ":",
    ];
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut offset = 0;
    let mut line = 0;
    let mut line_start = 0;
    while offset < bytes.len() {
        match bytes[offset] {
            b' ' | b'\t' | b'\r' => {
                offset += 1;
                continue;
            }
            b'\n' => {
                offset += 1;
                line += 1;
                line_start = offset;
                continue;
            }
            _ => {}
        }
        let start = offset;
        let kind;
        if bytes[offset] == b'#' {
            offset = source[offset..]
                .find('\n')
                .map_or(source.len(), |relative| offset + relative);
            kind = SpacingTokenKind::Comment;
        } else if matches!(bytes[offset], b'\'' | b'"' | b'`') {
            let quote = bytes[offset];
            offset += 1;
            let mut escaped = false;
            while offset < bytes.len() {
                let byte = bytes[offset];
                offset += 1;
                if byte == b'\n' {
                    line += 1;
                    line_start = offset;
                }
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == quote {
                    break;
                }
            }
            kind = SpacingTokenKind::String;
        } else if bytes[offset] == b'%' && percent_literal_end(source, offset).is_some() {
            offset = percent_literal_end(source, offset).unwrap_or(offset + 1);
            let value = &source[start..offset];
            let newlines = value.bytes().filter(|byte| *byte == b'\n').count();
            if newlines > 0 {
                line += newlines;
                line_start = start + value.rfind('\n').unwrap_or(0) + 1;
            }
            kind = SpacingTokenKind::String;
        } else if bytes[offset].is_ascii_alphanumeric()
            || matches!(bytes[offset], b'_' | b'@' | b'$')
            || bytes[offset] >= 0x80
        {
            offset += 1;
            while offset < bytes.len()
                && (bytes[offset].is_ascii_alphanumeric()
                    || matches!(bytes[offset], b'_' | b'?' | b'!' | b'@' | b'$')
                    || bytes[offset] >= 0x80
                    || (bytes[offset] == b'.'
                        && bytes
                            .get(offset.wrapping_sub(1))
                            .is_some_and(u8::is_ascii_digit)
                        && bytes.get(offset + 1).is_some_and(u8::is_ascii_digit)))
            {
                offset += 1;
            }
            kind = SpacingTokenKind::Word;
        } else if let Some(operator) = OPERATORS
            .iter()
            .find(|operator| source[offset..].starts_with(**operator))
        {
            offset += operator.len();
            kind = SpacingTokenKind::Operator;
        } else {
            offset += source[offset..].chars().next().map_or(1, char::len_utf8);
            kind = SpacingTokenKind::Punctuation;
        }
        tokens.push(SpacingToken {
            start,
            end: offset,
            line,
            column: start.saturating_sub(line_start),
            text: source[start..offset].to_string(),
            kind,
        });
    }
    tokens
}

fn percent_literal_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut delimiter_at = start + 1;
    if matches!(
        bytes.get(delimiter_at),
        Some(b'q' | b'Q' | b'w' | b'W' | b'i' | b'I' | b'x' | b'r' | b's')
    ) {
        delimiter_at += 1;
    }
    let opening = *bytes.get(delimiter_at)?;
    if opening.is_ascii_alphanumeric() || opening.is_ascii_whitespace() || opening == b'=' {
        return None;
    }
    let closing = match opening {
        b'(' => b')',
        b'[' => b']',
        b'{' => b'}',
        b'<' => b'>',
        value => value,
    };
    let paired = closing != opening;
    let mut depth = 1_usize;
    let mut escaped = false;
    let mut offset = delimiter_at + 1;
    while offset < bytes.len() {
        let byte = bytes[offset];
        offset += 1;
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if paired && byte == opening {
            depth += 1;
        } else if byte == closing {
            depth -= 1;
            if depth == 0 {
                while bytes.get(offset).is_some_and(u8::is_ascii_alphabetic) {
                    offset += 1;
                }
                return Some(offset);
            }
        }
    }
    Some(bytes.len())
}

fn multiline_hash_pair_ranges(source: &str) -> Vec<std::ops::Range<usize>> {
    struct PairRanges<'source> {
        source: &'source str,
        ranges: Vec<std::ops::Range<usize>>,
    }
    impl PairRanges<'_> {
        fn collect(&mut self, elements: ruby_prism::NodeList<'_>, multiline: bool) {
            if !multiline {
                return;
            }
            for element in elements.iter() {
                let Some(pair) = element.as_assoc_node() else { continue };
                let key_end = pair.key().location().end_offset();
                let value_start = pair.value().location().start_offset();
                if key_end <= value_start {
                    self.ranges.push(key_end..value_start);
                }
            }
        }
    }
    impl<'pr> ruby_prism::Visit<'pr> for PairRanges<'_> {
        fn visit_hash_node(&mut self, node: &ruby_prism::HashNode<'pr>) {
            self.collect(
                node.elements(),
                self.source[node.location().start_offset()..node.location().end_offset()]
                    .contains('\n'),
            );
            ruby_prism::visit_hash_node(self, node);
        }
        fn visit_keyword_hash_node(&mut self, node: &ruby_prism::KeywordHashNode<'pr>) {
            self.collect(
                node.elements(),
                self.source[node.location().start_offset()..node.location().end_offset()]
                    .contains('\n'),
            );
            ruby_prism::visit_keyword_hash_node(self, node);
        }
    }
    let parsed = ruby_prism::parse(source.as_bytes());
    let mut ranges = PairRanges {
        source,
        ranges: Vec::new(),
    };
    ranges.visit(&parsed.node());
    ranges.ranges
}

fn aligned_spacing_token(
    lines: &[&str],
    tokens: &[SpacingToken],
    token: &SpacingToken,
    full_line_comment_lines: &std::collections::HashSet<usize>,
) -> bool {
    if token.kind == SpacingTokenKind::Comment {
        return false;
    }
    let indentation = lines
        .get(token.line)
        .map_or(0, |line| line.len() - line.trim_start().len());
    let preceding = (0..token.line).rev().collect::<Vec<_>>();
    let following = (token.line + 1..lines.len()).collect::<Vec<_>>();
    [None, Some(indentation)].into_iter().any(|required_indent| {
        [&preceding, &following].into_iter().any(|line_numbers| {
            aligned_spacing_in_lines(
                lines,
                tokens,
                token,
                line_numbers,
                required_indent,
                full_line_comment_lines,
            )
        })
    })
}

fn aligned_spacing_in_lines(
    lines: &[&str],
    tokens: &[SpacingToken],
    token: &SpacingToken,
    line_numbers: &[usize],
    required_indent: Option<usize>,
    full_line_comment_lines: &std::collections::HashSet<usize>,
) -> bool {
    for line_number in line_numbers {
        if full_line_comment_lines.contains(line_number) {
            continue;
        }
        let Some(line) = lines.get(*line_number) else { continue };
        let Some(indentation) = line.find(|character: char| !character.is_whitespace()) else {
            continue;
        };
        if required_indent.is_some_and(|required| required != indentation) {
            continue;
        }
        let word_aligned = token.column > 0
            && line
                .get(token.column - 1..token.column + 1)
                .is_some_and(|pair| {
                    pair.as_bytes()[0].is_ascii_whitespace()
                        && !pair.as_bytes()[1].is_ascii_whitespace()
                })
            || line
                .get(token.column..token.column + token.text.len())
                .is_some_and(|value| value == token.text);
        let operator_aligned = equality_or_comparison(&token.text)
            && tokens
                .iter()
                .find(|candidate| {
                    candidate.line == *line_number && equality_or_comparison(&candidate.text)
                })
                .is_some_and(|candidate| aligned_operators(token, candidate));
        // RuboCop intentionally decides on the nearest eligible line. It does
        // not keep searching farther through the file after a non-match.
        return word_aligned || operator_aligned;
    }
    false
}

fn equality_or_comparison(value: &str) -> bool {
    matches!(value, "=" | "==" | "===" | "!=" | "<=" | ">=" | "<<")
        || (value.ends_with('=') && !matches!(value, "=>" | "=~" | "!~"))
}

fn aligned_operators(left: &SpacingToken, right: &SpacingToken) -> bool {
    let left_end = left.column + left.text.len();
    let right_end = right.column + right.text.len();
    left_end == right_end
        && ((left.text.ends_with('=') && right.text.ends_with('='))
            || (left.text == "<<" && right.text.ends_with('='))
            || (left.text.ends_with('=') && right.text == "<<"))
}

fn assignment_token_indices(tokens: &[SpacingToken], lines: &[&str]) -> Vec<usize> {
    let mut seen = std::collections::HashSet::new();
    tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| {
            assignment_operator(&token.text)
                && !lines
                    .get(token.line)
                    .is_some_and(|line| line.trim_start().starts_with("def "))
                && seen.insert(token.line)
        })
        .map(|(index, _)| index)
        .collect()
}

fn assignment_operator(value: &str) -> bool {
    value.ends_with('=')
        && !matches!(
            value,
            "==" | "===" | "!=" | "<=" | ">=" | "=>" | "=~" | "!~"
        )
}

fn check_forced_equal_alignment(
    context: &mut CopContext<'_, '_>,
    tokens: &[SpacingToken],
    lines: &[&str],
    assignments: &[usize],
) {
    for token_index in assignments {
        let token = &tokens[*token_index];
        let preceding = relevant_assignment_indices(token.line, true, tokens, lines, assignments);
        let Some(previous_index) = preceding.get(1) else {
            continue;
        };
        if aligned_operators(token, &tokens[*previous_index]) {
            continue;
        }
        let mut group = relevant_assignment_indices(token.line, true, tokens, lines, assignments);
        group.extend(relevant_assignment_indices(
            token.line,
            false,
            tokens,
            lines,
            assignments,
        ));
        group.sort_unstable();
        group.dedup();
        let first_offender = group.windows(2).find_map(|pair| {
            (!aligned_operators(&tokens[pair[1]], &tokens[pair[0]])).then_some(pair[1])
        });
        if first_offender != Some(*token_index) {
            context.replace_indirectly(
                "`=` is not aligned with the preceding assignment.",
                token.start..token.end,
                token.end..token.end,
                "",
            );
            continue;
        }
        let align_to = group
            .iter()
            .map(|index| assignment_align_column(context.source(), &tokens[*index]))
            .max()
            .unwrap_or(token.column + token.text.len());
        let edits = group
            .iter()
            .filter_map(|index| {
                let assignment = &tokens[*index];
                let end_column = assignment.column + assignment.text.len();
                if align_to > end_column {
                    Some((
                        assignment.start..assignment.start,
                        " ".repeat(align_to - end_column),
                    ))
                } else if align_to < end_column {
                    let count = end_column - align_to;
                    Some((assignment.start - count..assignment.start, String::new()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        context.replace_many(
            "`=` is not aligned with the preceding assignment.",
            token.start..token.end,
            edits,
        );
    }
}

fn relevant_assignment_indices(
    origin: usize,
    upward: bool,
    tokens: &[SpacingToken],
    lines: &[&str],
    assignments: &[usize],
) -> Vec<usize> {
    let original_indent = line_indentation(lines.get(origin).copied().unwrap_or_default());
    let mut result = Vec::new();
    let mut at_level = true;
    let line_numbers: Box<dyn Iterator<Item = usize>> = if upward {
        Box::new((0..=origin).rev())
    } else {
        Box::new(origin..lines.len())
    };
    for line_number in line_numbers {
        let line = lines[line_number];
        let current_indent = line_indentation(line);
        let blank = line.trim().is_empty();
        if (current_indent < original_indent && !blank) || (at_level && blank) {
            break;
        }
        if current_indent == original_indent {
            if let Some(index) = assignments
                .iter()
                .find(|index| tokens[**index].line == line_number)
            {
                result.push(*index);
            }
        }
        if !blank {
            at_level = current_indent == original_indent;
        }
    }
    result
}

fn assignment_align_column(source: &str, token: &SpacingToken) -> usize {
    let line_start = source[..token.start]
        .rfind('\n')
        .map_or(0, |offset| offset + 1);
    let preceding = &source[line_start..token.start];
    let spaces = preceding
        .bytes()
        .rev()
        .take_while(|byte| *byte == b' ')
        .count();
    token.column + token.text.len() - spaces + 1
}

fn line_indentation(line: &str) -> usize {
    line.bytes()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count()
}

fn first_hash_element_indentation(
    node: &ruby_prism::HashNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let opening = node.opening_loc();
    let closing = node.closing_loc();
    let file = context.source_file();
    let pairs = hash_pair_infos(node, context.source());
    let first = pairs.first();
    let left_parenthesis = enclosing_call_parenthesis(context, node, opening.start_offset());
    let style = context
        .config_value("EnforcedStyle")
        .unwrap_or("special_inside_parentheses")
        .to_string();
    let width = configured_hash_indentation_width(context);

    if let Some(pair) = first {
        if !file.same_line(opening.start_offset(), pair.start) {
            let (base, description) =
                hash_indentation_base(node, context, pair, left_parenthesis, &style);
            let separator_offset = separator_alignment_offset(context, &pairs, pair);
            let expected = base + width + separator_offset;
            let actual = file.column(pair.start);
            if actual != expected {
                let message = format!(
                    "Use {width} spaces for indentation in a hash, relative to {description}."
                );
                let edits = hash_pair_indentation_edits(context.source(), pair, actual, expected);
                context.replace_many(message, pair.start..pair.end, edits);
            }
        } else {
            return;
        }
    }

    let closing_start = closing.start_offset();
    let closing_line_start = file.line_start(closing_start);
    if !context.source()[closing_line_start..closing_start]
        .trim()
        .is_empty()
    {
        return;
    }
    let (expected, base_kind) = hash_indentation_base(
        node,
        context,
        first.unwrap_or(&HashPairInfo::empty(opening.end_offset())),
        left_parenthesis,
        &style,
    );
    let actual = file.column(closing_start);
    if actual == expected {
        return;
    }
    let message = match base_kind {
        "the position of the opening brace" => {
            "Indent the right brace the same as the left brace."
        }
        "the first position after the preceding left parenthesis" => {
            "Indent the right brace the same as the first position after the preceding left parenthesis."
        }
        "the parent hash key" => "Indent the right brace the same as the parent hash key.",
        _ => "Indent the right brace the same as the start of the line where the left brace is.",
    };
    context.replace(
        message,
        closing_start..closing.end_offset(),
        closing_line_start..closing_start,
        " ".repeat(expected),
    );
}

#[derive(Clone, Debug)]
struct HashPairInfo {
    start: usize,
    end: usize,
    key_start: usize,
    key_end: usize,
    value_start: usize,
    operator: String,
}

impl HashPairInfo {
    fn empty(offset: usize) -> Self {
        Self {
            start: offset,
            end: offset,
            key_start: offset,
            key_end: offset,
            value_start: offset,
            operator: String::new(),
        }
    }
}

fn hash_pair_infos(node: &ruby_prism::HashNode<'_>, source: &str) -> Vec<HashPairInfo> {
    node.elements()
        .iter()
        .filter_map(|element| {
            let pair = element.as_assoc_node()?;
            let location = pair.location();
            let key = pair.key().location();
            let value = pair.value().location();
            // Prism represents shorthand pairs such as `{ data: }` with the value
            // location aliased to the key, so the value may start before the key
            // location ends. There is no source gap to slice in that form.
            let between = source
                .get(key.end_offset()..value.start_offset())
                .unwrap_or_default();
            let operator = if between.contains("=>") { "=>" } else { ":" };
            Some(HashPairInfo {
                start: location.start_offset(),
                end: location.end_offset(),
                key_start: key.start_offset(),
                key_end: key.end_offset(),
                value_start: value.start_offset(),
                operator: operator.to_string(),
            })
        })
        .collect()
}

fn configured_hash_indentation_width(context: &CopContext<'_, '_>) -> usize {
    context
        .config_value("IndentationWidth")
        .and_then(|value| value.parse().ok())
        .or_else(|| {
            context
                .related_config_value("Layout/IndentationWidth", "Width")
                .and_then(|value| value.parse().ok())
        })
        .unwrap_or(2)
}

fn enclosing_call_parenthesis(
    context: &CopContext<'_, '_>,
    _hash: &ruby_prism::HashNode<'_>,
    opening_brace: usize,
) -> Option<usize> {
    if context.related_config_value("Layout/ArgumentAlignment", "EnforcedStyle")
        == Some("with_fixed_indentation")
    {
        return None;
    }
    let file = context.source_file();
    for ancestor in context.ancestors().iter().rev() {
        if ancestor.as_block_node().is_some() || ancestor.as_lambda_node().is_some() {
            return None;
        }
        if let Some(call) = ancestor.as_call_node() {
            let opening = call.opening_loc()?;
            return (opening.as_slice() == b"("
                && file.same_line(opening.start_offset(), opening_brace))
            .then_some(opening.start_offset());
        }
    }
    None
}

fn hash_indentation_base(
    node: &ruby_prism::HashNode<'_>,
    context: &CopContext<'_, '_>,
    first: &HashPairInfo,
    left_parenthesis: Option<usize>,
    style: &str,
) -> (usize, &'static str) {
    let file = context.source_file();
    let opening = node.opening_loc().start_offset();
    if style == "align_braces" {
        return (file.column(opening), "the position of the opening brace");
    }
    if first.start != first.end {
        if let Some(column) = parent_hash_key_column(context, opening, first) {
            return (column, "the parent hash key");
        }
    }
    if style == "special_inside_parentheses" {
        if let Some(parenthesis) = left_parenthesis {
            return (
                file.column(parenthesis) + 1,
                "the first position after the preceding left parenthesis",
            );
        }
    }
    (
        file.column(file.indentation(opening).end),
        "the start of the line where the left curly brace is",
    )
}

fn parent_hash_key_column(
    context: &CopContext<'_, '_>,
    opening: usize,
    _first: &HashPairInfo,
) -> Option<usize> {
    let file = context.source_file();
    let parent = context.parent()?.as_assoc_node()?;
    let parent_location = parent.location();
    let key = parent.key().location();
    let value = parent.value().location();
    if value.start_offset() != opening || !file.same_line(key.start_offset(), value.start_offset())
    {
        return None;
    }
    let siblings = context.ancestors().iter().rev().find_map(|ancestor| {
        if let Some(hash) = ancestor.as_hash_node() {
            return Some(hash.elements().iter().collect::<Vec<_>>());
        }
        ancestor
            .as_keyword_hash_node()
            .map(|hash| hash.elements().iter().collect::<Vec<_>>())
    })?;
    let position = siblings.iter().position(|candidate| {
        let location = candidate.location();
        location.start_offset() == parent_location.start_offset()
            && location.end_offset() == parent_location.end_offset()
    })?;
    let sibling = siblings.get(position + 1)?;
    if file.same_line(
        parent_location.end_offset(),
        sibling.location().start_offset(),
    ) {
        return None;
    }
    Some(file.column(parent_location.start_offset()))
}

fn separator_alignment_offset(
    context: &CopContext<'_, '_>,
    pairs: &[HashPairInfo],
    first: &HashPairInfo,
) -> usize {
    let key = if first.operator == ":" {
        "EnforcedColonStyle"
    } else {
        "EnforcedHashRocketStyle"
    };
    if context.related_config_value("Layout/HashAlignment", key) != Some("separator") {
        return 0;
    }
    let longest = pairs
        .iter()
        .map(|pair| pair.key_end - pair.key_start)
        .max()
        .unwrap_or(first.key_end - first.key_start);
    longest.saturating_sub(first.key_end - first.key_start)
}

fn hash_pair_indentation_edits(
    source: &str,
    pair: &HashPairInfo,
    actual: usize,
    expected: usize,
) -> Vec<(std::ops::Range<usize>, String)> {
    let file = SourceFile::new(source);
    let first_line = file.line_start(pair.start);
    let mut edits = vec![(first_line..pair.start, " ".repeat(expected))];
    if !file.same_line(pair.key_start, pair.value_start) {
        return edits;
    }
    let delta = expected as isize - actual as isize;
    let mut line_start = source[pair.start..pair.end]
        .find('\n')
        .map(|relative| pair.start + relative + 1);
    while let Some(start) = line_start {
        if start >= pair.end {
            break;
        }
        let indentation = file.indentation(start);
        let adjusted = (indentation.len() as isize + delta).max(0) as usize;
        edits.push((indentation, " ".repeat(adjusted)));
        line_start = source[start..pair.end]
            .find('\n')
            .map(|relative| start + relative + 1);
    }
    edits
}
