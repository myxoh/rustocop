use super::helpers::*;
use super::{push_offense, CorrectionStatus, Offense, SourceLine};
use crate::config::InspectionConfig;

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
            let corrected = options.autocorrect && correctable;
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

fn check_line_length(
    lines: &mut [SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Layout/LineLength";
    if !options.cop_enabled(cop) {
        return;
    }

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
    let allow_qualified = options
        .cop_config
        .explicitly_contains(cop, "AllowQualifiedName")
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
    let heredoc_configured = options.cop_config.explicitly_contains(cop, "AllowHeredoc");
    let allow_heredoc = heredoc_configured
        .then(|| options.cop_config.value(cop, "AllowHeredoc"))
        .flatten();
    let allowed_heredocs = if heredoc_configured {
        options.cop_config.values(cop, "AllowHeredoc")
    } else {
        &[]
    };
    let split_strings = options
        .cop_config
        .bool(cop, "SplitStrings")
        .unwrap_or(false);
    let mut heredoc_queue = std::collections::VecDeque::<(String, bool)>::new();
    let mut heredoc_stack = Vec::<(String, bool)>::new();
    let mut heredoc: Option<(String, bool)> = None;
    let mut nesting = 0isize;
    for (index, line) in lines.iter_mut().enumerate() {
        if line.body == "__END__" && heredoc.is_none() {
            break;
        }
        let closes_heredoc = heredoc
            .as_ref()
            .is_some_and(|(delimiter, _)| line.body.trim() == delimiter);
        let in_allowed_heredoc = heredoc.as_ref().is_some_and(|(_, allowed)| *allowed);
        let length = visual_length(&line.body, tab_width);
        let directive_at = line.body.find("rubocop:");
        let length_without_directive = directive_at
            .and_then(|at| line.body[..at].rfind('#'))
            .map_or(length, |at| line.body[..at].trim_end().chars().count());
        let effective_length = if allow_directives && directive_at.is_some() {
            length_without_directive
        } else {
            length
        };
        let exempt = index == 0 && line.body.starts_with("#!")
            || in_allowed_heredoc
            || allowed_patterns
                .iter()
                .any(|pattern| pattern.is_match(&line.body))
            || allow_rbs && (line.body.contains("#:") || line.body.contains("# @rbs"))
            || allow_uri
                && allowed_excess_token(&line.body, max, ExcessToken::Uri, uri_schemes, tab_width)
            || allow_qualified
                && allowed_excess_token(
                    &line.body,
                    max,
                    ExcessToken::QualifiedName,
                    &[],
                    tab_width,
                );
        if effective_length > max && !exempt {
            let breakable = (heredoc.is_none() || line.body.contains("#{"))
                && line_length_breakable(&line.body, max, split_strings, nesting);
            let indentation_difference = line
                .body
                .chars()
                .take_while(|character| *character == '\t')
                .count()
                * tab_width.saturating_sub(1);
            let raw_limit = max.saturating_sub(indentation_difference);
            let token_end = allow_uri
                .then(|| excessive_token_end(&line.body, raw_limit, ExcessToken::Uri, uri_schemes))
                .flatten()
                .or_else(|| {
                    allow_qualified
                        .then(|| {
                            excessive_token_end(
                                &line.body,
                                raw_limit,
                                ExcessToken::QualifiedName,
                                &[],
                            )
                        })
                        .flatten()
                });
            let column = token_end
                .map(|end| end + 1)
                .unwrap_or_else(|| max.saturating_sub(indentation_difference) + 1);
            let message = format!("Line is too long. [{}/{}]", effective_length, max);
            let length = effective_length.saturating_sub(column - 1).max(1);
            let correctable = breakable;
            let corrected = options.autocorrect && breakable;
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
        let openings = heredoc_delimiters(&line.body)
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
    }
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
            if let Some(at) = commas
                .iter()
                .map(|(at, _)| *at)
                .filter(|at| *at < heredoc)
                .next_back()
            {
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
            || line[..open].contains("let(")
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

fn excessive_token_end(
    line: &str,
    max: usize,
    kind: ExcessToken,
    uri_schemes: &[String],
) -> Option<usize> {
    match kind {
        ExcessToken::Uri if line.matches("://").count() > 1 => None,
        ExcessToken::Uri => uri_schemes.iter().find_map(|scheme| {
            let start = line.find(&format!("{scheme}://"))?;
            (start < max)
                .then(|| {
                    let end = line[start..]
                        .find(|character: char| {
                            character.is_ascii_whitespace()
                                || matches!(character, '\'' | '"' | ')' | ']' | '}')
                        })
                        .map_or(line.len(), |at| start + at);
                    let wrapper = line[end..].chars().next();
                    (end > max).then_some(
                        (end + usize::from(wrapper.is_some_and(|c| !c.is_whitespace())))
                            .min(line.len()),
                    )
                })
                .flatten()
        }),
        ExcessToken::QualifiedName => {
            let mut offset = 0usize;
            line.split_inclusive(char::is_whitespace).find_map(|piece| {
                let leading = piece.len() - piece.trim_start().len();
                let raw = piece.trim();
                let token = raw.trim_matches(['\'', '"', '(', ')', '[', ']', '{', '}', ',']);
                let start = offset + leading + raw.find(token).unwrap_or(0);
                offset += piece.len();
                let end = start + token.len();
                let wrapper = line[end..].chars().next();
                (token.contains("::") && start < max && end > max).then_some(
                    (end + usize::from(wrapper.is_some_and(|c| !c.is_whitespace())))
                        .min(line.len()),
                )
            })
        }
    }
}

#[derive(Clone, Copy)]
enum ExcessToken {
    Uri,
    QualifiedName,
}

fn allowed_excess_token(
    line: &str,
    max: usize,
    kind: ExcessToken,
    uri_schemes: &[String],
    tab_width: usize,
) -> bool {
    if matches!(kind, ExcessToken::Uri) {
        if let Some(open) = line.find('{') {
            let close = line[open + 1..]
                .rfind('}')
                .map(|relative| open + 1 + relative);
            if let Some(close) = close {
                let candidate = &line[open + 1..close];
                if uri_schemes
                    .iter()
                    .any(|scheme| candidate.starts_with(&format!("{scheme}://")))
                    && line
                        .chars()
                        .count()
                        .saturating_sub(candidate.chars().count())
                        <= max
                    && line[close + 1..].trim().is_empty()
                {
                    return true;
                }
            }
        }
        for scheme in uri_schemes {
            let needle = format!("{scheme}://");
            let Some(start) = line.find(&needle) else {
                continue;
            };
            let end = line[start..]
                .find(|character: char| {
                    character.is_ascii_whitespace()
                        || matches!(character, '\'' | '"' | ')' | ']' | '}')
                })
                .map_or(line.len(), |at| start + at);
            if end < line.len() && line[end..].chars().next().is_some_and(char::is_whitespace) {
                continue;
            }
            if !line[end..]
                .trim()
                .chars()
                .all(|character| matches!(character, '\'' | '"' | ')' | ']' | '}'))
            {
                continue;
            }
            let non_uri_length =
                visual_length(&line[..start], tab_width) + visual_length(&line[end..], tab_width);
            let visual_start = visual_length(&line[..start], tab_width);
            let visual_end = visual_length(&line[..end], tab_width);
            if visual_start <= max && visual_end >= max && non_uri_length <= max {
                return true;
            }
        }
        return false;
    }
    let mut column = 0usize;
    line.split_inclusive(char::is_whitespace).any(|piece| {
        let start = column;
        column += piece.chars().count();
        let token = piece
            .trim()
            .trim_matches(['\'', '"', '(', ')', '[', ']', '{', '}', '<', '>', ',']);
        let applicable = match kind {
            ExcessToken::Uri => uri_schemes
                .iter()
                .any(|scheme| token.starts_with(&format!("{scheme}://"))),
            ExcessToken::QualifiedName => token.split("::").count() >= 2,
        };
        let token_length = token.chars().count();
        applicable
            && start <= max
            && column >= max
            && column == line.chars().count()
            && piece == piece.trim_end()
            && line.chars().count().saturating_sub(token_length) <= max
    })
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
    let breakable_block = line.find('{').is_some_and(|at| at > 0) || line.contains(" do");
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

fn heredoc_delimiters(line: &str) -> Vec<String> {
    let mut delimiters = Vec::new();
    let mut rest = line;
    while let Some(marker) = rest.find("<<") {
        rest = &rest[marker + 2..];
        rest = rest.strip_prefix(['-', '~']).unwrap_or(rest);
        let quote = rest
            .chars()
            .next()
            .filter(|character| matches!(character, '\'' | '"' | '`'));
        if quote.is_some() {
            rest = &rest[1..];
        }
        let delimiter = rest
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
            .collect::<String>();
        if delimiter.is_empty() {
            continue;
        }
        rest = &rest[delimiter.len()..];
        delimiters.push(delimiter);
    }
    delimiters
}
