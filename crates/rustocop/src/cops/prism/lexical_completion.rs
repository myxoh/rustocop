use super::*;

define_cops! {
    ClosingHeredocIndentation => "Layout/ClosingHeredocIndentation" => any_node(closing_heredoc_indentation),
    DisableCopsWithinSourceCodeDirective => "Style/DisableCopsWithinSourceCodeDirective" => compatibility_source(disable_cops_within_source_code_directive),
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

fn closing_heredoc_indentation(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (opening, closing) = if let Some(string) = node.as_string_node() {
        (string.opening_loc(), string.closing_loc())
    } else if let Some(string) = node.as_interpolated_string_node() {
        (string.opening_loc(), string.closing_loc())
    } else {
        return;
    };
    let (Some(opening), Some(closing)) = (opening, closing) else {
        return;
    };
    if !matches!(opening.as_slice(), [b'<', b'<', b'-' | b'~', ..]) {
        return;
    }

    let file = context.source_file();
    let opening_line = file.line(opening.start_offset());
    let closing_line = file.line(closing.start_offset());
    let opening_indent = opening_line.len() - opening_line.trim_start().len();
    let closing_indent = closing_line.len() - closing_line.trim_start().len();
    if opening_indent == closing_indent {
        return;
    }

    let heredoc_start = opening.start_offset();
    let mut argument = false;
    let mut chained = false;
    let mut outer_call = None;
    let ancestors = context.ancestors();
    let mut direct_call = None;
    for (index, ancestor) in ancestors.iter().enumerate().rev() {
        if ancestor.as_arguments_node().is_some() {
            continue;
        }
        direct_call = ancestor.as_call_node().map(|call| (index, call));
        break;
    }
    if let Some((call_index, call)) = direct_call {
        let contains = |location: ruby_prism::Location<'_>| {
            location.start_offset() <= heredoc_start && heredoc_start < location.end_offset()
        };
        argument = call
            .arguments()
            .is_some_and(|arguments| contains(arguments.location()));
        chained = call
            .receiver()
            .is_some_and(|receiver| contains(receiver.location()));
        if argument || chained {
            outer_call = Some(call);
            for ancestor in ancestors[..call_index].iter().rev() {
                if ancestor.as_arguments_node().is_some() {
                    continue;
                }
                let Some(parent_call) = ancestor.as_call_node() else {
                    break;
                };
                outer_call = Some(parent_call);
            }
        }
    }
    if let Some(call) = outer_call {
        let call_indent = file.indentation(call.location().start_offset()).len();
        if (argument || chained) && closing_indent == call_indent {
            return;
        }
    }

    let identifier = closing_line.trim();
    let opening_text = opening_line.trim();
    let suffix = if argument {
        " or beginning of method definition"
    } else {
        ""
    };
    let closing_start = file.line_start(closing.start_offset());
    context.replace(
        format!("`{identifier}` is not aligned with `{opening_text}`{suffix}."),
        closing_start..closing_start + closing_indent + identifier.len(),
        closing_start..closing_start + closing_indent,
        " ".repeat(opening_indent),
    );
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

fn disable_cops_within_source_code_directive(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    let allowed = context.config_values("AllowedCops").to_vec();
    let source = context.source();
    let mut all_disabled = false;
    for range in context.source_file().comment_ranges() {
        let comment = source.get(range.clone()).unwrap_or_default();
        let Some((command, list)) = directive(comment) else {
            continue;
        };
        let cops = list.split(',').map(str::trim).collect::<Vec<_>>();
        let disables_this_cop = matches!(command, "disable" | "todo")
            && cops.iter().any(|cop| {
                cop.split_whitespace()
                    .next()
                    .map(|name| name.trim_matches(|character: char| {
                        !character.is_alphanumeric() && !matches!(character, '_' | '/')
                    }))
                    == Some("Style/DisableCopsWithinSourceCodeDirective")
            });
        if disables_this_cop
            && !context.related_config_explicit(
                "Style/DisableCopsWithinSourceCodeDirective",
                "Enabled",
            )
        {
            continue;
        }
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
            .filter(|cop| !cop.is_empty() && !allowed.iter().any(|allowed| allowed == cop))
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
    if cops.starts_with('.') {
        return None;
    }
    matches!(command, "disable" | "enable" | "todo").then_some((command, cops))
}
