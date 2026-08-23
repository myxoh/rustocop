use super::*;

define_cops! {
    ClosingHeredocIndentation => "Layout/ClosingHeredocIndentation" => source(closing_heredoc_indentation),
    Encoding => "Style/Encoding" => source(encoding),
    DisableCopsWithinSourceCodeDirective => "Style/DisableCopsWithinSourceCodeDirective" => source(disable_cops_within_source_code_directive),
    RedundantHeredocDelimiterQuotes => "Style/RedundantHeredocDelimiterQuotes" => source(redundant_heredoc_delimiter_quotes),
}

fn redundant_heredoc_delimiter_quotes(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let heredoc_starts = context
        .source_file()
        .literal_ranges()
        .into_iter()
        .filter_map(|range| source[range.clone()].starts_with("<<").then_some(range.start))
        .collect::<std::collections::HashSet<_>>();
    let mut search_from = 0;
    while let Some(relative) = source[search_from..].find("<<") {
        let start = search_from + relative;
        if !heredoc_starts.contains(&start) {
            search_from = start + 2;
            continue;
        }
        let bytes = source.as_bytes();
        let mut quote_offset = start + 2;
        if matches!(bytes.get(quote_offset), Some(b'~' | b'-')) {
            quote_offset += 1;
        }
        let Some(quote @ (b'\'' | b'"')) = bytes.get(quote_offset).copied() else {
            search_from = start + 2;
            continue;
        };
        let Some(close_relative) = source[quote_offset + 1..].find(char::from(quote)) else {
            break;
        };
        let close = quote_offset + 1 + close_relative;
        let identifier = &source[quote_offset + 1..close];
        if identifier.is_empty()
            || !identifier
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            search_from = close + 1;
            continue;
        }
        let body_start = source[close + 1..]
            .find('\n')
            .map_or(close + 1, |newline| close + 2 + newline);
        let closing_start = source[body_start..]
            .lines()
            .scan(body_start, |offset, line| {
                let current = *offset;
                *offset += line.len() + 1;
                Some((current, line))
            })
            .find(|(_, line)| line.trim() == identifier)
            .map(|(offset, _)| offset)
            .unwrap_or(source.len());
        let body = source.get(body_start..closing_start).unwrap_or_default();
        if body.contains("#{")
            || body.contains("#@")
            || body.contains("#$")
            || body.contains('\\')
        {
            search_from = close + 1;
            continue;
        }
        let token_end = close + 1;
        let replacement = format!("{}{}", &source[start..quote_offset], identifier);
        context.replace(
            format!("Remove the redundant heredoc delimiter quotes, use `{replacement}` instead."),
            start..token_end,
            start..token_end,
            replacement,
        );
        search_from = token_end;
    }
}

fn closing_heredoc_indentation(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for (index, (_, line)) in lines.iter().enumerate() {
        let Some(marker) = line.find("<<-").or_else(|| line.find("<<~")) else {
            continue;
        };
        let marker_tail = line[marker + 3..].trim_start();
        let identifier_tail = marker_tail
            .strip_prefix(['\'', '"', '`'])
            .unwrap_or(marker_tail);
        let identifier = identifier_tail
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
            .collect::<String>();
        if identifier.is_empty() {
            continue;
        }
        let opening_indent = line.len() - line.trim_start().len();
        let expression_indent = if index > 0 && lines[index - 1].1.trim_end().ends_with(',') {
            lines[index - 1].1.len() - lines[index - 1].1.trim_start().len()
        } else {
            opening_indent
        };
        let Some((closing_offset, closing_line)) = lines[index + 1..]
            .iter()
            .find(|(_, candidate)| candidate.trim() == identifier)
        else {
            continue;
        };
        let closing_indent = closing_line.len() - closing_line.trim_start().len();
        if closing_indent == opening_indent || closing_indent == expression_indent {
            continue;
        }
        let offense = *closing_offset..closing_offset + closing_indent + identifier.len();
        context.replace(
            format!(
                "`{identifier}` is not aligned with `{}`.",
                line.trim()
            ),
            offense,
            *closing_offset..closing_offset + closing_indent,
            " ".repeat(opening_indent),
        );
    }
}

fn encoding(context: &mut CopContext<'_, '_>) {
    let mut accepts_magic_comment = true;
    for (offset, line) in context.source_file().lines() {
        let text = line.trim_end_matches('\r');
        if offset == 0 && text.starts_with("#!") {
            continue;
        }
        if !accepts_magic_comment || text.is_empty() || !text.starts_with('#') {
            break;
        }
        let lower = text.to_ascii_lowercase();
        if !utf8_encoding_comment(&lower) {
            accepts_magic_comment = magic_comment(&lower);
            continue;
        }
        let replacement = encoding_comment_without_encoding(text, &lower);
        let offense = offset..offset + text.len();
        let edit_end = if replacement.is_empty()
            && context.source().as_bytes().get(offset + text.len()) == Some(&b'\n')
        {
            offset + text.len() + 1
        } else {
            offset + text.len()
        };
        context.replace(
            "Unnecessary utf-8 encoding comment.",
            offense,
            offset..edit_end,
            replacement,
        );
    }
}

fn utf8_encoding_comment(lower: &str) -> bool {
    let compact = lower.replace(' ', "");
    (compact.contains("encoding:utf-8")
        || compact.contains("coding:utf-8")
        || compact.contains("fileencoding=utf-8"))
        && lower.starts_with('#')
}

fn magic_comment(lower: &str) -> bool {
    lower.contains("frozen_string_literal")
        || lower.contains("coding")
        || lower.contains("encoding")
        || lower.starts_with("# vim:")
        || lower.starts_with("# -*-")
}

fn encoding_comment_without_encoding(text: &str, lower: &str) -> String {
    if lower.starts_with("# vim:") && lower.contains("filetype=ruby") {
        return "# vim: filetype=ruby".to_string();
    }
    if lower.starts_with("# -*-") && lower.contains("mode:") {
        let mode_part = text
            .split(';')
            .find(|part| part.to_ascii_lowercase().contains("mode:"))
            .unwrap_or("mode: ruby");
        let mode = mode_part
            .split_once(':')
            .map(|(_, value)| value.trim().trim_end_matches(&['-', '*'][..]).trim())
            .unwrap_or("ruby");
        return format!("# -*- mode: {mode} -*-");
    }
    String::new()
}

fn disable_cops_within_source_code_directive(context: &mut CopContext<'_, '_>) {
    let allowed = context.config_values("AllowedCops").to_vec();
    let source = context.source();
    let mut all_disabled = false;
    for range in context.source_file().comment_ranges() {
        let comment = source.get(range.clone()).unwrap_or_default();
        let Some((command, list)) = directive(comment) else {
            continue;
        };
        let cops = list.split(',').map(str::trim).collect::<Vec<_>>();
        if command == "enable" && cops.contains(&"all") && all_disabled {
            all_disabled = false;
            continue;
        }
        if command == "disable" && cops.contains(&"all") {
            all_disabled = true;
        }
        let disallowed = cops
            .iter()
            .copied()
            .filter(|cop| !allowed.iter().any(|allowed| allowed == cop))
            .collect::<Vec<_>>();
        if disallowed.is_empty() {
            continue;
        }
        let message = if allowed.is_empty() {
            "RuboCop disable/enable directives are not permitted.".to_string()
        } else {
            format!(
                "RuboCop disable/enable directives for `{}` are not permitted.",
                disallowed.join("`, `")
            )
        };
        let retained = cops
            .iter()
            .copied()
            .filter(|cop| allowed.iter().any(|allowed| allowed == cop))
            .collect::<Vec<_>>();
        let replacement = if retained.is_empty() {
            String::new()
        } else {
            format!("# rubocop:{command} {}", retained.join(", "))
        };
        context.replace(message, range.clone(), range, replacement);
    }
}

fn directive(comment: &str) -> Option<(&str, &str)> {
    let marker = comment.find("rubocop")?;
    let before_marker = &comment[..marker];
    let directive_hash = before_marker.trim_end().strip_suffix('#')?;
    if directive_hash.starts_with('#')
        && directive_hash[1..].chars().all(char::is_whitespace)
    {
        return None;
    }
    let after_marker = comment[marker + "rubocop".len()..].trim_start();
    let body = after_marker.strip_prefix(':')?;
    let (command, cops) = body.trim().split_once(' ')?;
    let cops = cops.split_once(" -- ").map_or(cops, |(cops, _)| cops).trim();
    matches!(command, "disable" | "enable" | "todo").then_some((command, cops))
}
