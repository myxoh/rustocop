use super::*;

mod literals;
use literals::*;

define_cops! {
    Copyright => "Style/Copyright" => source(copyright),
    CommentedKeyword => "Style/CommentedKeyword" => source(commented_keyword),
    CommentAnnotation => "Style/CommentAnnotation" => source(comment_annotation),
    NumericLiterals => "Style/NumericLiterals" => any_node(numeric_literals),
    CommandLiteral => "Style/CommandLiteral" => any_node(command_literal),
}

fn copyright(context: &mut CopContext<'_, '_>) {
    let notice = context.config_value("Notice").unwrap_or("Copyright");
    let visible_notice = unescape_config(notice);
    let first_code = context
        .source_file()
        .lines()
        .find(|(_, line)| {
            let line = line.trim();
            !line.is_empty() && !line.starts_with('#')
        })
        .map_or(context.source().len(), |(offset, _)| offset);
    if context.source()[..first_code].contains("Copyright")
        || context.source().starts_with("=begin") && context.source().contains("Copyright")
    {
        return;
    }
    let message =
        format!("Include a copyright notice matching /{visible_notice}/ before any code.");
    if context.source().is_empty() {
        context.report(message, 0..0);
        return;
    }
    let correction = context
        .config_value("AutocorrectNotice")
        .map(unescape_config)
        .unwrap_or_default();
    if correction.is_empty() {
        context.report(message, 0..1);
        return;
    }
    let correction = if correction.starts_with('#') {
        correction
    } else {
        correction
            .lines()
            .map(|line| {
                if line.is_empty() {
                    "#".to_string()
                } else {
                    format!("# {line}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let mut insertion = 0;
    for (offset, line) in context.source_file().lines() {
        let magic = offset == 0 && line.starts_with("#!")
            || line.contains("coding:")
            || line.contains("encoding:");
        if !magic {
            break;
        }
        insertion = offset
            + line.len()
            + usize::from(context.source().as_bytes().get(offset + line.len()) == Some(&b'\n'));
    }
    let suffix = if insertion == 0 && correction.contains('\n') {
        "\n"
    } else {
        ""
    };
    context.replace(
        message,
        0..1,
        insertion..insertion,
        format!("{}\n{suffix}", correction.trim_end_matches('\n')),
    );
}

fn unescape_config(value: &str) -> String {
    value
        .replace("\\n", "\n")
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

fn commented_keyword(context: &mut CopContext<'_, '_>) {
    let mut heredoc_end: Option<String> = None;
    for (offset, line) in context.source_file().lines() {
        if let Some(marker) = &heredoc_end {
            if line.trim() == marker {
                heredoc_end = None;
            }
            continue;
        }
        if let Some(at) = line.find("<<-").or_else(|| line.find("<<~")) {
            let marker = line[at + 3..].trim().trim_matches(['\'', '"']).to_string();
            if !marker.is_empty() {
                heredoc_end = Some(marker);
                continue;
            }
        }
        let trimmed = line.trim_start();
        let Some(keyword) =
            ["begin", "class", "def", "end", "module"]
                .into_iter()
                .find(|keyword| {
                    trimmed.starts_with(keyword)
                        && trimmed
                            .as_bytes()
                            .get(keyword.len())
                            .is_some_and(u8::is_ascii_whitespace)
                })
        else {
            continue;
        };
        let Some(comment_at) = ruby_comment_offset(line) else {
            continue;
        };
        let comment = &line[comment_at..];
        let compact_comment = comment.replace([' ', '\t'], "");
        let steep_annotation = comment
            .strip_prefix("# ")
            .map(str::trim_start)
            .unwrap_or_default();
        if compact_comment.contains("rubocop:")
            || comment.contains(":nodoc:")
            || comment.contains(":yields:")
            || steep_annotation == "steep:ignore"
            || steep_annotation.starts_with("steep:ignore ")
            || (comment.starts_with("#:") && matches!(keyword, "def" | "end"))
            || (comment.starts_with("#[")
                && comment.trim_end().ends_with(']')
                && line[..comment_at].contains('<'))
        {
            continue;
        }
        let offense = offset + comment_at..offset + line.len();
        let (edit, replacement) = if keyword == "end" {
            (
                offset + line[..comment_at].trim_end().len()..offset + line.len(),
                String::new(),
            )
        } else {
            (
                offset..offset + line.len(),
                format!("{comment}\n{}", line[..comment_at].trim_end()),
            )
        };
        context.replace(
            format!("Do not place comments on the same line as the `{keyword}` keyword."),
            offense,
            edit,
            replacement,
        );
    }
}

fn ruby_comment_offset(line: &str) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;
    for (index, byte) in line.bytes().enumerate() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == delimiter {
                quote = None;
            }
        } else if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
        } else if byte == b'#' {
            return Some(index);
        }
    }
    None
}

fn comment_annotation(context: &mut CopContext<'_, '_>) {
    let keywords = context.config_values("Keywords").to_vec();
    let require_colon = context.config_bool("RequireColon", true);
    let mut previous_comment_line = None;
    for (line_number, (offset, line)) in context.source_file().lines().enumerate() {
        let Some(hash) = line.find('#') else {
            previous_comment_line = None;
            continue;
        };
        let comment_only = line[..hash].trim().is_empty();
        if comment_only && previous_comment_line == Some(line_number.saturating_sub(1)) {
            previous_comment_line = Some(line_number);
            continue;
        }
        previous_comment_line = comment_only.then_some(line_number);
        let after_hash = &line[hash + 1..];
        let leading = after_hash.len() - after_hash.trim_start().len();
        let text = after_hash.trim_start();
        let Some(configured_keyword) = keywords
            .iter()
            .filter(|keyword| text.len() >= keyword.len())
            .filter(|keyword| text[..keyword.len()].eq_ignore_ascii_case(keyword))
            .max_by_key(|keyword| keyword.len())
        else {
            continue;
        };
        let raw_keyword = configured_keyword.len();
        let keyword = &text[..raw_keyword];
        let remainder = &text[raw_keyword..];
        if remainder.is_empty() || remainder.starts_with(['.', '(']) {
            continue;
        }
        if keyword
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_uppercase())
            && keyword != keyword.to_uppercase()
            && !remainder.starts_with(':')
        {
            continue;
        }
        let has_note = !remainder.trim_matches([' ', ':']).is_empty();
        let correct_prefix = if require_colon { ": " } else { " " };
        if has_note && keyword == keyword.to_uppercase() && remainder.starts_with(correct_prefix) {
            continue;
        }
        let consumed = raw_keyword
            + remainder
                .bytes()
                .take_while(|byte| matches!(byte, b':' | b' ' | b'\t'))
                .count();
        let start = offset + hash + 1 + leading;
        let range = start..start + consumed.max(raw_keyword);
        let message = if has_note {
            if require_colon {
                format!("Annotation keywords like `{keyword}` should be all upper case, followed by a colon, and a space, then a note describing the problem.")
            } else {
                format!("Annotation keywords like `{keyword}` should be all upper case, followed by a space, then a note describing the problem.")
            }
        } else {
            format!("Annotation comment, with keyword `{keyword}`, is missing a note.")
        };
        if has_note {
            context.replace(
                message,
                range.clone(),
                range,
                format!("{}{correct_prefix}", keyword.to_uppercase()),
            );
        } else {
            context.report(message, range);
        }
    }
}
