use super::helpers::*;
use super::{push_offense, CorrectionStatus, Offense, SourceLine};
use crate::config::InspectionConfig;
use ruby_prism::Visit;

const TRAILING_WHITESPACE_COP: &str = "Layout/TrailingWhitespace";

pub(super) fn before_prism(
    lines: &mut [SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    check_line_length(lines, options, offenses);
    check_trailing_whitespace(lines, options, offenses);
}

pub(super) fn after_prism(
    _lines: &[SourceLine],
    _options: &InspectionConfig,
    _offenses: &mut Vec<Offense>,
) {
}

fn check_trailing_whitespace(
    lines: &mut [SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = TRAILING_WHITESPACE_COP;
    if !options.cop_enabled(cop) {
        return;
    }
    let autocorrect = options.autocorrect_for(cop);

    let allow_in_heredoc = options
        .cop_config
        .bool(cop, "AllowInHeredoc")
        .unwrap_or(false);
    let mut openings = heredoc_openings(lines);
    let mut heredoc: Option<(String, bool, Option<usize>)> = None;
    let mut in_documentation_comment = false;

    for (index, line) in lines.iter_mut().enumerate() {
        let in_heredoc = heredoc.is_some();
        let heredoc_is_interpolated = heredoc
            .as_ref()
            .is_none_or(|(_, interpolated, _)| *interpolated);
        let heredoc_indentation = heredoc
            .as_ref()
            .and_then(|(_, _, indentation)| *indentation);
        let closes_heredoc = heredoc
            .as_ref()
            .is_some_and(|(terminator, _, _)| line.body.trim() == terminator);
        if !in_heredoc && !in_documentation_comment && line.body == "__END__" {
            break;
        }

        if !in_heredoc && line.body.starts_with("=begin") {
            in_documentation_comment = true;
        }

        let length = trailing_whitespace_len(&line.body);
        if length != 0 && !(allow_in_heredoc && in_heredoc) {
            let correctable = !in_heredoc || heredoc_is_interpolated;
            let corrected = autocorrect && correctable;
            let column = line.body.chars().count() - length + 1;
            push_offense(
                offenses,
                cop,
                "Trailing whitespace detected.",
                index + 1,
                column,
                length,
                CorrectionStatus::from_flags(correctable, corrected),
            );

            if corrected {
                if in_heredoc && line.body.chars().count() > length {
                    escape_heredoc_trailing_whitespace(&mut line.body, length);
                } else if in_heredoc
                    && heredoc_indentation.is_some_and(|indentation| length > indentation)
                {
                    let indentation = heredoc_indentation.expect("checked above");
                    escape_whitespace_beyond_indentation(&mut line.body, indentation);
                } else {
                    trim_trailing_spaces(&mut line.body);
                }
            }
        }

        if in_heredoc {
            if closes_heredoc {
                heredoc = None;
            }
        } else if let Some(opening) = openings[index].take() {
            heredoc = Some(opening);
        }

        if !in_heredoc && line.body.starts_with("=end") {
            in_documentation_comment = false;
        }
    }
}

fn heredoc_opening(line: &str) -> Option<(String, bool, bool)> {
    let marker = line.find("<<")?;
    let mut rest = &line[marker + 2..];
    let squiggly = rest.starts_with('~');
    rest = rest.strip_prefix(['-', '~']).unwrap_or(rest);
    let quote = rest
        .chars()
        .next()
        .filter(|character| matches!(character, '\'' | '"' | '`'));
    if quote.is_some() {
        rest = &rest[1..];
    }
    let name = rest
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect::<String>();
    (!name.is_empty()).then_some((name, quote != Some('\''), squiggly))
}

fn heredoc_openings(lines: &[SourceLine]) -> Vec<Option<(String, bool, Option<usize>)>> {
    lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            let (terminator, interpolated, squiggly) = heredoc_opening(&line.body)?;
            let indentation = squiggly.then(|| {
                lines[index + 1..]
                    .iter()
                    .take_while(|line| line.body.trim() != terminator)
                    .filter(|line| !line.body.trim().is_empty())
                    .map(|line| {
                        line.body
                            .chars()
                            .take_while(|character| matches!(character, ' ' | '\t'))
                            .count()
                    })
                    .min()
                    .unwrap_or(0)
            });
            Some((terminator, interpolated, indentation))
        })
        .collect()
}

fn escape_heredoc_trailing_whitespace(line: &mut String, length: usize) {
    let split = line
        .char_indices()
        .nth(line.chars().count() - length)
        .map_or(line.len(), |(offset, _)| offset);
    let whitespace = line[split..].to_string();
    line.replace_range(split.., &format!("#{{'{whitespace}'}}"));
}

fn escape_whitespace_beyond_indentation(line: &mut String, indentation: usize) {
    let split = line
        .char_indices()
        .nth(indentation)
        .map_or(line.len(), |(offset, _)| offset);
    let whitespace = line[split..].to_string();
    line.replace_range(split.., &format!("#{{'{whitespace}'}}"));
}

#[allow(clippy::too_many_lines)]
fn check_line_length(
    lines: &mut [SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Layout/LineLength";
    if !options.cop_enabled(cop) {
        return;
    }
    // RuboCop does not invoke this AST cop when the configured parser cannot
    // build a valid syntax tree. This matters for generator templates that
    // intentionally contain ERB/placeholders but still have a `.rb` suffix.
    let parsed_source = lines
        .iter()
        .map(|line| format!("{}{}", line.body, line.ending))
        .collect::<String>();
    let parsed = ruby_prism::parse(parsed_source.as_bytes());
    if parsed.errors().next().is_some() {
        return;
    }
    let autocorrect = options.autocorrect_for(cop);

    let max = options
        .cop_config
        .value(cop, "Max")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(120);
    let allowed_patterns = options.cop_config.patterns(cop, "AllowedPatterns");
    let allow_uri = options.cop_config.bool(cop, "AllowURI").unwrap_or(true);
    let uri_schemes = options.cop_config.values(cop, "URISchemes");
    let tab_width = options
        .cop_config
        .value("Layout/IndentationStyle", "IndentationWidth")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(2);
    let complete_default_config = options
        .cop_config
        .bool("AllCops", "DisabledByDefault")
        .unwrap_or(false);
    let allow_qualified = (options
        .cop_config
        .explicitly_contains(cop, "AllowQualifiedName")
        || complete_default_config)
        && options
            .cop_config
            .bool(cop, "AllowQualifiedName")
            .unwrap_or(false);
    let allow_directives = options
        .cop_config
        .bool(cop, "AllowCopDirectives")
        .unwrap_or(true);
    let allow_rbs = options
        .cop_config
        .bool(cop, "AllowRBSInlineAnnotation")
        .unwrap_or(false);
    let heredoc_configured =
        options.cop_config.explicitly_contains(cop, "AllowHeredoc") || complete_default_config;
    let allow_heredoc = if options.cop_config.explicitly_contains(cop, "AllowHeredoc") {
        options.cop_config.value(cop, "AllowHeredoc")
    } else {
        complete_default_config.then_some("true")
    };
    let allowed_heredocs = if heredoc_configured {
        options.cop_config.values(cop, "AllowHeredoc")
    } else {
        &[]
    };
    let split_strings = options
        .cop_config
        .bool(cop, "SplitStrings")
        .unwrap_or(false);
    let allow_all_heredocs = allow_heredoc == Some("true");
    let allowed_heredoc_lines = allowed_line_length_heredoc_lines(
        &parsed_source,
        &parsed,
        allow_all_heredocs,
        allowed_heredocs,
    );
    let comment_columns = prism_comment_columns(lines, &parsed_source, &parsed);
    let mut heredoc_queue = std::collections::VecDeque::<(String, bool)>::new();
    let mut heredoc_stack = Vec::<(String, bool)>::new();
    let mut heredoc: Option<(String, bool)> = None;
    let mut nesting = 0isize;
    let mut directive_disabled = false;
    let mut follows_autocorrect_split = false;
    for (index, line) in lines.iter_mut().enumerate() {
        if line.body == "__END__" && heredoc.is_none() {
            break;
        }
        let closes_heredoc = heredoc
            .as_ref()
            .is_some_and(|(delimiter, _)| line.body.trim() == delimiter);
        let in_allowed_heredoc = if allow_all_heredocs {
            allowed_heredoc_lines.contains(&index)
        } else {
            heredoc.as_ref().is_some_and(|(_, allowed)| *allowed)
        };
        let comment_column = comment_columns.get(index).copied().flatten();
        let directive_marker =
            comment_column.and_then(|comment| line_length_directive_marker(&line.body, comment));
        let line_disabled = update_line_length_directive(
            &line.body,
            comment_column,
            directive_marker,
            &mut directive_disabled,
        );
        let length = visual_length(&line.body, tab_width);
        let directive_at = directive_marker.map(|(marker, _)| marker);
        let length_without_directive =
            directive_at.map_or(length, |at| line.body[..at].trim_end().chars().count());
        let effective_length = if allow_directives && directive_at.is_some() {
            length_without_directive
        } else {
            length
        };
        let directive_length_only = allow_directives && directive_at.is_some();
        let indentation_difference = line
            .body
            .chars()
            .take_while(|character| *character == '\t')
            .count()
            * tab_width.saturating_sub(1);
        let applicable_token_range = |range: (usize, usize)| {
            let adjusted = (
                range.0 + indentation_difference,
                range.1 + indentation_difference,
            );
            (!(adjusted.0 < max && adjusted.1 < max)).then_some(adjusted)
        };
        let uri_range = (!directive_length_only && allow_uri)
            .then(|| last_excess_token_range(&line.body, ExcessToken::Uri, uri_schemes))
            .flatten()
            .and_then(applicable_token_range);
        let qualified_range = (!directive_length_only && allow_qualified)
            .then(|| last_excess_token_range(&line.body, ExcessToken::QualifiedName, &[]))
            .flatten()
            .and_then(applicable_token_range);
        let token_position_allowed = |range: (usize, usize)| range.0 < max && range.1 == length;
        let excess_tokens_allowed = match (uri_range, qualified_range) {
            (Some(uri), Some(qualified)) => {
                token_position_allowed(uri) && token_position_allowed(qualified)
            }
            (Some(uri), None) => token_position_allowed(uri),
            (None, Some(qualified)) => token_position_allowed(qualified),
            (None, None) => false,
        };
        let exempt = index == 0 && line.body.starts_with("#!")
            || in_allowed_heredoc
            || allowed_patterns
                .iter()
                .any(|pattern| pattern.is_match(&line.body))
            || allow_rbs && (line.body.contains("#:") || line.body.contains("# @rbs"))
            || excess_tokens_allowed;
        if effective_length > max && !exempt && !line_disabled {
            let breakable = !follows_autocorrect_split
                && (heredoc.is_none() || line.body.contains("#{"))
                && line_length_breakable(&line.body, max, split_strings, nesting);
            let raw_limit = max.saturating_sub(indentation_difference);
            let token_end = uri_range
                .or(qualified_range)
                .and_then(|(start, end)| (start < raw_limit).then_some(end));
            let column = token_end
                .map(|end| end + 1)
                .unwrap_or_else(|| max.saturating_sub(indentation_difference) + 1);
            let message = format!("Line is too long. [{}/{}]", effective_length, max);
            let length = effective_length.saturating_sub(column - 1).max(1);
            let correctable = breakable;
            let corrected = autocorrect && breakable;
            let raw_length = line.body.chars().count();
            let (last_line, last_column) = if effective_length > raw_length {
                (
                    index + 2,
                    effective_length
                        .saturating_sub(raw_length + usize::from(!line.ending.is_empty())),
                )
            } else {
                (index + 1, effective_length)
            };
            offenses.push(Offense {
                cop_name: cop.to_string(),
                message,
                corrected,
                correctable,
                line: index + 1,
                column,
                last_line,
                last_column,
                length,
            });
            if corrected {
                line.body = correct_line_length(&line.body, max, split_strings);
            }
        }
        if closes_heredoc {
            heredoc = heredoc_stack.pop().or_else(|| heredoc_queue.pop_front());
            continue;
        }
        let parent_allowed = heredoc.as_ref().is_some_and(|(_, allowed)| *allowed);
        let openings = heredoc_delimiters(&line.body, heredoc.is_some())
            .into_iter()
            .map(|delimiter| {
                let allowed = parent_allowed
                    || allow_heredoc == Some("true")
                    || allowed_heredocs.iter().any(|value| value == &delimiter);
                (delimiter, allowed)
            })
            .collect::<Vec<_>>();
        if heredoc.is_some() && !openings.is_empty() {
            heredoc_stack.push(heredoc.take().unwrap());
            heredoc = openings.first().cloned();
            heredoc_queue.extend(openings.into_iter().skip(1));
        } else if heredoc.is_none() && !openings.is_empty() {
            heredoc = openings.first().cloned();
            heredoc_queue.extend(openings.into_iter().skip(1));
        }
        if heredoc.is_none() {
            nesting += delimiter_delta(&line.body);
            nesting = nesting.max(0);
        }
        follows_autocorrect_split = line.body.ends_with(", ")
            || line.body.trim_end().ends_with('|') && line.body.contains('{');
    }
}

fn line_length_directive_marker(line: &str, comment: usize) -> Option<(usize, usize)> {
    for (relative, _) in line[comment..].match_indices('#') {
        let marker = comment + relative;
        let mut cursor = marker + 1;
        while line
            .as_bytes()
            .get(cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            cursor += 1;
        }
        if !line[cursor..].starts_with("rubocop") {
            continue;
        }
        cursor += "rubocop".len();
        while line
            .as_bytes()
            .get(cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            cursor += 1;
        }
        if line.as_bytes().get(cursor) != Some(&b':') {
            continue;
        }
        // DirectiveComment deliberately rejects `# # rubocop:...`, while
        // accepting a directive later in a substantive comment.
        if marker > comment && line[comment + 1..marker].trim().is_empty() {
            return None;
        }
        return Some((marker, cursor + 1));
    }
    None
}

fn update_line_length_directive(
    line: &str,
    comment: Option<usize>,
    marker: Option<(usize, usize)>,
    disabled: &mut bool,
) -> bool {
    let (Some(comment), Some((_marker, directive_start))) = (comment, marker) else {
        return *disabled;
    };
    let directive = line[directive_start..].trim_start();
    let (turn_off, rest) = if let Some(rest) = directive.strip_prefix("disable") {
        (true, rest)
    } else if let Some(rest) = directive.strip_prefix("todo") {
        (true, rest)
    } else if let Some(rest) = directive.strip_prefix("enable") {
        (false, rest)
    } else {
        return *disabled;
    };
    let applies = rest
        .split_once(" --")
        .map_or(rest, |(names, _)| names)
        .split(',')
        .map(str::trim)
        .any(|name| matches!(name, "all" | "Layout" | "Layout/LineLength"));
    if !applies {
        return *disabled;
    }
    if line[..comment].trim().is_empty() {
        *disabled = turn_off;
        *disabled
    } else {
        turn_off || *disabled
    }
}

fn prism_comment_columns(
    lines: &[SourceLine],
    source: &str,
    parsed: &ruby_prism::ParseResult<'_>,
) -> Vec<Option<usize>> {
    let mut columns = vec![None; lines.len()];
    for comment in parsed.comments() {
        let start = comment.location().start_offset();
        let line_start = source[..start].rfind('\n').map_or(0, |newline| newline + 1);
        let line = source[..line_start]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count();
        if let Some(column) = columns.get_mut(line) {
            *column = Some(start - line_start);
        }
    }
    columns
}

fn correct_line_length(line: &str, max: usize, split_strings: bool) -> String {
    if let Some(at) = first_semicolon_outside_string(line) {
        let mut end = at + 1;
        while line.as_bytes().get(end) == Some(&b';') {
            end += 1;
        }
        return format!("{}\n{}", &line[..end], &line[end..]);
    }

    if let Some(at) = block_break_position(line) {
        return format!("{}\n{}", &line[..at], &line[at..]);
    }

    if line.contains(" = ") && line.contains(',') {
        let assignment = line.find(" = ").unwrap() + 3;
        let with_assignment_break = format!("{}\n{}", &line[..assignment], &line[assignment..]);
        let second_line = &with_assignment_break[assignment + 1..];
        if let Some(comma) = commas_outside_strings(second_line)
            .into_iter()
            .find(|(_, depth)| *depth == 0)
            .map(|(at, _)| at)
        {
            let split = assignment + 1 + comma + 1;
            let split = consume_spaces(&with_assignment_break, split);
            return format!(
                "{}\n{}",
                &with_assignment_break[..split],
                &with_assignment_break[split..]
            );
        }
        return with_assignment_break;
    }

    let commas = commas_outside_strings(line);
    if !commas.is_empty() {
        if let Some(heredoc) = line.find("<<") {
            if let Some(at) = commas.iter().map(|(at, _)| *at).rfind(|at| *at < heredoc) {
                let split = consume_spaces(line, at + 1);
                return format!("{}\n{}", &line[..split], &line[split..]);
            }
        }
        let minimum_depth = commas
            .iter()
            .filter(|(at, _)| *at < max)
            .map(|(_, depth)| *depth)
            .min()
            .unwrap_or_else(|| commas.iter().map(|(_, depth)| *depth).min().unwrap_or(0));
        if let Some(at) = commas
            .iter()
            .filter(|(at, depth)| *depth == minimum_depth && *at < max)
            .map(|(at, _)| *at)
            .next_back()
        {
            let split = consume_spaces(line, at + 1);
            return format!("{}\n{}", &line[..split], &line[split..]);
        }
        if let Some(open) = preferred_opening_delimiter(line, max) {
            let split = consume_spaces(line, open + 1);
            return format!("{}\n{}", &line[..split], &line[split..]);
        }
        let split = consume_spaces(line, commas[0].0 + 1);
        return format!("{}\n{}", &line[..split], &line[split..]);
    }

    if split_strings {
        if let Some(corrected) = split_long_string(line, max) {
            return corrected;
        }
    }
    line.to_string()
}

fn consume_spaces(line: &str, mut at: usize) -> usize {
    while line.as_bytes().get(at) == Some(&b' ') {
        at += 1;
    }
    at
}

fn first_semicolon_outside_string(line: &str) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;
    for (at, character) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && quote.is_some() {
            escaped = true;
            continue;
        }
        if let Some(delimiter) = quote {
            if character == delimiter {
                quote = None;
            }
        } else if matches!(character, '\'' | '"') {
            quote = Some(character);
        } else if character == ';' {
            return Some(at);
        }
    }
    None
}

fn block_break_position(line: &str) -> Option<usize> {
    if let Some(open) = line.find('{') {
        let block_like = line[open..].starts_with("{|")
            || line[open..].starts_with("{ |")
            || line[..open].contains("select")
            || line[..open].trim_start().starts_with("->");
        if !block_like || open > 0 && line.as_bytes().get(open - 1) == Some(&b'#') {
            return None;
        }
        if let Some(first_pipe) = line[open + 1..].find('|').map(|at| open + 1 + at) {
            if let Some(second_pipe) = line[first_pipe + 1..]
                .find('|')
                .map(|at| first_pipe + 1 + at)
            {
                return Some(second_pipe + 1);
            }
        }
        return Some(open + 1);
    }
    if let Some(open) = line.find(" do") {
        let after_do = open + 3;
        if let Some(first_pipe) = line[after_do..].find('|').map(|at| after_do + at) {
            if let Some(second_pipe) = line[first_pipe + 1..]
                .find('|')
                .map(|at| first_pipe + 1 + at)
            {
                return Some(second_pipe + 1);
            }
        }
        return Some(after_do);
    }
    None
}

fn commas_outside_strings(line: &str) -> Vec<(usize, usize)> {
    let mut commas = Vec::new();
    let mut quote = None;
    let mut escaped = false;
    let mut depth = 0usize;
    for (at, character) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && quote.is_some() {
            escaped = true;
            continue;
        }
        if let Some(delimiter) = quote {
            if character == delimiter {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            ',' => commas.push((at, depth)),
            _ => {}
        }
    }
    commas
}

fn preferred_opening_delimiter(line: &str, max: usize) -> Option<usize> {
    line.char_indices()
        .filter(|(at, character)| *at < max && matches!(character, '(' | '[' | '{'))
        .map(|(at, _)| at)
        .next_back()
}

fn split_long_string(line: &str, max: usize) -> Option<String> {
    let (quote, delimiter, closing) = quoted_span_crossing(line, max)?;
    if closing < max || closing <= quote + 1 {
        return None;
    }
    let prefix = &line[..quote];
    let content = &line[quote + 1..closing];
    let suffix = &line[closing + delimiter.len_utf8()..];
    let capacity = max.saturating_sub(prefix.chars().count() + 4).max(1);
    let mut split = content
        .char_indices()
        .nth(capacity)
        .map_or(content.len(), |(at, _)| at);
    if split > 0
        && content.as_bytes().get(split - 1) == Some(&b'#')
        && content.as_bytes().get(split) == Some(&b'{')
    {
        split -= 1;
    }
    for interpolation in content.match_indices("#{").map(|(at, _)| at) {
        if interpolation >= split {
            break;
        }
        if interpolation_end(content, interpolation).is_none_or(|end| end >= split) {
            split = interpolation;
            break;
        }
    }
    while split > 0 && content.as_bytes().get(split - 1) == Some(&b'\\') {
        split -= 1;
    }
    if let Some(escape) = content[..split].rfind('\\') {
        let escape_length = match content.as_bytes().get(escape + 1) {
            Some(b'u') => 6,
            Some(b'x') => 4,
            Some(_) => 2,
            None => 1,
        };
        if escape + escape_length >= split {
            split = escape;
        }
    }
    if let Some(space) = content[..split].rfind(char::is_whitespace) {
        split = space + content[space..].chars().next().unwrap().len_utf8();
    }
    if split == 0 || split >= content.len() {
        return None;
    }
    let left = &content[..split];
    let right = &content[split..];
    let first = format!("{prefix}{delimiter}{left}{delimiter} \\");
    let remainder = format!("{delimiter}{right}{delimiter}{suffix}");
    let remainder = if remainder.chars().count() > max {
        split_long_string(&remainder, max).unwrap_or(remainder)
    } else {
        remainder
    };
    Some(format!("{first}\n{remainder}"))
}

fn quoted_span_crossing(line: &str, max: usize) -> Option<(usize, char, usize)> {
    if let Some(delimiter) = line.chars().next().filter(|c| matches!(c, '\'' | '"')) {
        if let Some(closing) = line.rfind(delimiter).filter(|at| *at >= max) {
            return Some((0, delimiter, closing));
        }
    }
    let mut opening: Option<(usize, char)> = None;
    let mut escaped = false;
    for (at, character) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && opening.is_some() {
            escaped = true;
            continue;
        }
        if let Some((start, delimiter)) = opening {
            if character == delimiter {
                if at >= max {
                    return Some((start, delimiter, at));
                }
                opening = None;
            }
        } else if matches!(character, '\'' | '"') {
            opening = Some((at, character));
        }
    }
    None
}

fn interpolation_end(content: &str, start: usize) -> Option<usize> {
    let mut depth = 1usize;
    let mut at = start + 2;
    while at < content.len() {
        if content[at..].starts_with("#{") {
            depth += 1;
            at += 2;
            continue;
        }
        match content.as_bytes()[at] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(at);
                }
            }
            _ => {}
        }
        at += 1;
    }
    None
}

fn last_excess_token_range(
    line: &str,
    kind: ExcessToken,
    uri_schemes: &[String],
) -> Option<(usize, usize)> {
    match kind {
        ExcessToken::Uri => {
            let mut candidates = Vec::new();
            for scheme in uri_schemes {
                let scheme_prefix = format!("{scheme}:");
                for (start, _) in line.match_indices(&scheme_prefix) {
                    let after_scheme = start + scheme_prefix.len();
                    let uri_end = if line[after_scheme..].starts_with('\\') {
                        // URI's regexp still recognizes `http:` in a regexp
                        // literal such as `/http:\/\/example/`.
                        after_scheme
                    } else {
                        rfc2396_uri_end(line, start)
                    };
                    // URI's regexp consumes a closing bracket in YARD-style
                    // `[https://...]` links; URI.parse then rejects the match.
                    let candidate = &line[start..uri_end];
                    // URI.parse rejects an unmatched closing bracket in a
                    // fragment. This is what keeps Markdown links whose first
                    // URL has a fragment from being treated as one giant URI.
                    let valid = !line[after_scheme..uri_end].starts_with(':')
                        && !candidate.split_once('#').is_some_and(|(_, fragment)| {
                            fragment.matches(']').count() > fragment.matches('[').count()
                        });
                    candidates.push((start, uri_end, valid));
                }
            }
            candidates.sort_unstable();
            // URI.make_regexp consumes nested `http://` text as part of the
            // surrounding URI. Scheme-by-scheme scanning must therefore not
            // promote a nested URL (commonly a query parameter value) to a
            // separate, later match.
            let mut matches = Vec::new();
            let mut covered_until = 0;
            for (start, end, valid) in candidates {
                if start < covered_until {
                    continue;
                }
                covered_until = end;
                if valid {
                    matches.push((start, end));
                }
            }
            let (start, uri_end) = matches.last().copied()?;
            let extended_end = extend_non_whitespace(line, uri_end);
            Some((
                line[..start].chars().count(),
                line[..extended_end].chars().count(),
            ))
        }
        ExcessToken::QualifiedName => {
            static QUALIFIED_NAME: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
            let pattern = QUALIFIED_NAME.get_or_init(|| {
                regex::Regex::new(r"\b(?:[A-Z][A-Za-z0-9_]*::)+[A-Za-z_][A-Za-z0-9_]*\b").unwrap()
            });
            let found = pattern.find_iter(line).last()?;
            let end = extend_non_whitespace(line, found.end());
            Some((
                line[..found.start()].chars().count(),
                line[..end].chars().count(),
            ))
        }
    }
}

fn rfc2396_uri_end(line: &str, start: usize) -> usize {
    let mut end = start;
    let mut query_or_fragment = false;
    while end < line.len() {
        let byte = line.as_bytes()[end];
        if matches!(byte, b'?' | b'#') {
            query_or_fragment = true;
        }
        let allowed = byte.is_ascii_alphanumeric()
            || matches!(
                byte,
                b'-' | b'_'
                    | b'.'
                    | b'!'
                    | b'~'
                    | b'*'
                    | b'\''
                    | b'('
                    | b')'
                    | b';'
                    | b'/'
                    | b'?'
                    | b':'
                    | b'@'
                    | b'&'
                    | b'='
                    | b'+'
                    | b'$'
                    | b','
                    | b'#'
                    | b'%'
            )
            || query_or_fragment && matches!(byte, b'[' | b']');
        if !allowed {
            break;
        }
        end += 1;
    }
    end
}

fn extend_non_whitespace(line: &str, start: usize) -> usize {
    // LineLengthHelp extends a URI/qualified-name match through a trailing
    // brace expression (originally intended for YARD links). RuboCop applies
    // that rule to ordinary Ruby blocks too.
    if line.contains('{') && line.trim_end().ends_with('}') && line[start..].contains('}') {
        return line.trim_end().len();
    }
    start
        + line[start..]
            .chars()
            .take_while(|character| !character.is_whitespace())
            .map(char::len_utf8)
            .sum::<usize>()
}

#[derive(Clone, Copy)]
enum ExcessToken {
    Uri,
    QualifiedName,
}

fn visual_length(source: &str, tab_width: usize) -> usize {
    let leading_tabs = source
        .chars()
        .take_while(|character| *character == '\t')
        .count();
    source.chars().count() + leading_tabs * tab_width.saturating_sub(1)
}

fn line_length_breakable(line: &str, max: usize, split_strings: bool, _nesting: isize) -> bool {
    let trimmed = line.trim_start();
    if trimmed == "#" || trimmed.starts_with("# ") || line.contains('%') && line.contains('{') {
        return false;
    }
    if line.matches("#{").count() > 1 {
        return true;
    }
    if line.contains("<<") {
        return line
            .find(',')
            .zip(line.find("<<"))
            .is_some_and(|(comma, heredoc)| comma < heredoc && comma < max);
    }
    let mut words = trimmed.split_whitespace();
    let receiver = words.next().unwrap_or_default();
    let unparenthesized_hash_call =
        !receiver.ends_with(':') && words.next().is_some_and(|word| word.ends_with(':'));
    if unparenthesized_hash_call && (line.matches(": ").count() <= 1 || trimmed.ends_with(',')) {
        return false;
    }
    if _nesting > 0
        && (!line.contains('{')
            || trimmed
                .find('(')
                .is_some_and(|at| at > 0 && !trimmed[..at].contains(char::is_whitespace)))
    {
        return false;
    }
    if split_strings {
        if let Some(quote) = line.find(['\'', '"']) {
            let delimiter = line.as_bytes()[quote];
            let closing = line[quote + 1..]
                .rfind(delimiter as char)
                .map(|at| quote + 1 + at);
            let interpolation_only = closing.is_some_and(|closing| {
                let content = &line[quote + 1..closing];
                content.starts_with("#{")
                    && content.ends_with('}')
                    && content.matches("#{").count() == 1
            });
            if interpolation_only {
                return false;
            }
            let comment_before = line[..quote]
                .rfind('#')
                .is_some_and(|at| line.as_bytes().get(at + 1) != Some(&b'{'));
            if quote < max
                && quote + 3 < max
                && closing.is_some_and(|closing| closing >= max)
                && !comment_before
            {
                return true;
            }
        }
    }
    let comma_before_limit = line.match_indices(',').any(|(at, _)| at < max);
    let plain_let_block = line
        .find('{')
        .is_some_and(|open| line[..open].contains("let("));
    let breakable_block =
        plain_let_block || block_break_position(line).is_some_and(|at| at < line.trim_end().len());
    let breakable_semicolon = line
        .find(';')
        .is_some_and(|at| at < line.trim_end_matches(';').len());
    let parenthesized_call = line.find('(').is_some_and(|at| at < max) && line.contains(',');
    let unparenthesized_call = line.find(char::is_whitespace).is_some_and(|at| {
        at > 0
            && line[..at]
                .chars()
                .all(|c| c.is_alphanumeric() || "_.!?".contains(c))
    }) && line.contains(',')
        && !line.trim_end().ends_with(',');
    comma_before_limit
        || breakable_block
        || breakable_semicolon
        || parenthesized_call
        || unparenthesized_call
        || line.contains(" = ") && line.contains(',')
}

fn delimiter_delta(line: &str) -> isize {
    let mut quote = None;
    let mut delta = 0isize;
    for byte in line.bytes() {
        if let Some(delimiter) = quote {
            if byte == delimiter {
                quote = None;
            }
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' | b'[' | b'{' => delta += 1,
            b')' | b']' | b'}' => delta -= 1,
            _ => {}
        }
    }
    delta
}

fn heredoc_delimiters(line: &str, in_heredoc: bool) -> Vec<String> {
    // A heredoc body can only introduce a nested heredoc from interpolated
    // Ruby. Its other text is not tokenized as Ruby source.
    let mut cursor = if in_heredoc {
        let Some(interpolation) = line.find("#{") else {
            return Vec::new();
        };
        interpolation + 2
    } else {
        0
    };
    let mut delimiters = Vec::new();
    let bytes = line.as_bytes();
    let mut quote = None;
    let mut escaped = false;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == delimiter {
                quote = None;
            }
            cursor += 1;
            continue;
        }
        if byte == b'#' {
            // `#{` is Ruby only while scanning an interpolated heredoc body;
            // elsewhere a hash begins a comment.
            if in_heredoc && bytes.get(cursor + 1) == Some(&b'{') {
                cursor += 2;
                continue;
            }
            break;
        }
        if matches!(byte, b'\'' | b'"' | b'`') {
            quote = Some(byte);
            cursor += 1;
            continue;
        }
        if bytes.get(cursor..cursor + 2) != Some(b"<<") {
            cursor += 1;
            continue;
        }
        let mut end = cursor + 2;
        if matches!(bytes.get(end), Some(b'-' | b'~')) {
            end += 1;
        }
        let delimiter_quote = bytes
            .get(end)
            .copied()
            .filter(|byte| matches!(byte, b'\'' | b'"' | b'`'));
        if delimiter_quote.is_some() {
            end += 1;
        }
        let name_start = end;
        while bytes
            .get(end)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            end += 1;
        }
        if end > name_start {
            delimiters.push(line[name_start..end].to_string());
        }
        if delimiter_quote.is_some() && bytes.get(end) == delimiter_quote.as_ref() {
            end += 1;
        }
        cursor = end.max(cursor + 2);
    }
    delimiters
}

fn allowed_line_length_heredoc_lines(
    source: &str,
    parsed: &ruby_prism::ParseResult<'_>,
    allow_all: bool,
    allowed_delimiters: &[String],
) -> std::collections::HashSet<usize> {
    if !allow_all {
        return std::collections::HashSet::new();
    }
    let mut collector = LineLengthHeredocCollector {
        source,
        allow_all,
        allowed_delimiters,
        lines: std::collections::HashSet::new(),
    };
    collector.visit(&parsed.node());
    collector.lines
}

struct LineLengthHeredocCollector<'source, 'config> {
    source: &'source str,
    allow_all: bool,
    allowed_delimiters: &'config [String],
    lines: std::collections::HashSet<usize>,
}

impl LineLengthHeredocCollector<'_, '_> {
    fn record(
        &mut self,
        opening: Option<ruby_prism::Location<'_>>,
        closing: Option<ruby_prism::Location<'_>>,
    ) {
        let (Some(opening), Some(closing)) = (opening, closing) else {
            return;
        };
        let opening_source = &self.source[opening.start_offset()..opening.end_offset()];
        if !opening_source.starts_with("<<") {
            return;
        }
        let delimiter = self.source[closing.start_offset()..closing.end_offset()].trim();
        if !self.allow_all
            && !self
                .allowed_delimiters
                .iter()
                .any(|allowed| allowed == delimiter)
        {
            return;
        }
        let opening_line = self.source[..opening.start_offset()]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count();
        let closing_line = self.source[..closing.start_offset()]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count();
        self.lines.extend(opening_line + 1..closing_line);
    }
}

impl<'pr> Visit<'pr> for LineLengthHeredocCollector<'_, '_> {
    fn visit_string_node(&mut self, node: &ruby_prism::StringNode<'pr>) {
        self.record(node.opening_loc(), node.closing_loc());
    }

    fn visit_interpolated_string_node(&mut self, node: &ruby_prism::InterpolatedStringNode<'pr>) {
        self.record(node.opening_loc(), node.closing_loc());
        ruby_prism::visit_interpolated_string_node(self, node);
    }
}
