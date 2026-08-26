use super::*;
use regex::RegexBuilder;

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
    let mut comments = context.source_file().comment_ranges().into_iter().fold(
        String::new(),
        |mut text, range| {
            if range.start >= first_code {
                return text;
            }
            let comment = &context.source()[range];
            text.push_str(comment.strip_prefix('#').unwrap_or(comment).trim_start());
            text.push('\n');
            text
        },
    );
    if context.source().starts_with("=begin") {
        if let Some(end) = context.source().find("\n=end") {
            comments.push_str(&context.source()["=begin".len()..end]);
        }
    }
    let pattern = copyright_notice_pattern(&visible_notice);
    if RegexBuilder::new(pattern)
        .multi_line(true)
        .build()
        .is_ok_and(|pattern| pattern.is_match(&comments))
    {
        return;
    }
    let message =
        format!("Include a copyright notice matching /{visible_notice}/ before any code.");
    let offense = if context.source().starts_with('\n') && !context.source().trim().is_empty() {
        0..1
    } else {
        0..0
    };
    if context.source().is_empty() {
        context.report(message, offense);
        return;
    }
    let correction = context
        .config_value("AutocorrectNotice")
        .map(unescape_config)
        .unwrap_or_default();
    if correction.is_empty() {
        context.report(message, offense);
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
        offense,
        insertion..insertion,
        format!("{}\n{suffix}", correction.trim_end_matches('\n')),
    );
}

fn copyright_notice_pattern(notice: &str) -> &str {
    let anchored = notice
        .strip_prefix(r"\A")
        .or_else(|| notice.strip_prefix('^'))
        .unwrap_or(notice);
    let Some(without_comment) = anchored.strip_prefix('#') else {
        return notice;
    };
    let mut pattern = without_comment.trim_start();
    for whitespace in [r"\s+", r"\s*", r"\s?", r"\s"] {
        if let Some(rest) = pattern.strip_prefix(whitespace) {
            pattern = rest.trim_start();
            break;
        }
    }
    pattern
}

fn unescape_config(value: &str) -> String {
    value
        .replace("\\n", "\n")
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

fn commented_keyword(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for comment_range in context.source_file().comment_ranges() {
        let offset = context.source_file().line_start(comment_range.start);
        let line_end = source[offset..]
            .find('\n')
            .map_or(source.len(), |relative| offset + relative);
        let line = &source[offset..line_end];
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
        let comment_at = comment_range.start - offset;
        let comment = &source[comment_range.clone()];
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
        let offense = comment_range;
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

fn comment_annotation(context: &mut CopContext<'_, '_>) {
    let keywords = context.config_values("Keywords").to_vec();
    let require_colon = context.config_bool("RequireColon", true);
    let mut previous_comment_line = None;
    for comment_range in context.source_file().comment_ranges() {
        if context.source().as_bytes().get(comment_range.start) != Some(&b'#') {
            continue;
        }
        let line_number = context.line_index(comment_range.start);
        let offset = context.line_start_at(line_number);
        let line = context.line_at(line_number);
        let hash = comment_range.start - offset;
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
            .filter(|keyword| {
                text.get(..keyword.len())
                    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(keyword))
                    && text
                        .as_bytes()
                        .get(keyword.len())
                        .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
            })
            .max_by_key(|keyword| keyword.len())
        else {
            continue;
        };
        let raw_keyword = configured_keyword.len();
        let keyword = &text[..raw_keyword];
        let remainder = &text[raw_keyword..];
        let colon = remainder.find(':').filter(|colon| {
            remainder[..*colon]
                .bytes()
                .all(|byte| matches!(byte, b' ' | b'\t'))
        });
        let colon_len = colon.map_or(0, |colon| colon + 1);
        let space_len = remainder[colon_len..]
            .bytes()
            .take_while(|byte| matches!(byte, b' ' | b'\t'))
            .count();
        let has_colon = colon.is_some();
        let has_space = space_len > 0;
        if !has_colon && !has_space {
            continue;
        }
        let note = &remainder[colon_len + space_len..];
        let has_note = note.bytes().next().is_some_and(|byte| !byte.is_ascii_whitespace());
        if keyword_is_capitalized(keyword) && !has_colon && has_space && has_note {
            continue;
        }
        if keyword == keyword.to_uppercase()
            && has_space
            && has_note
            && has_colon == require_colon
        {
            continue;
        }
        let correct_prefix = if require_colon { ": " } else { " " };
        let consumed = raw_keyword + colon_len + space_len;
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
    embedded_comment_annotation(context, &keywords, require_colon);
}

fn keyword_is_capitalized(keyword: &str) -> bool {
    let mut bytes = keyword.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_uppercase())
        && bytes.all(|byte| !byte.is_ascii_uppercase())
}

fn embedded_comment_annotation(
    context: &mut CopContext<'_, '_>,
    keywords: &[String],
    require_colon: bool,
) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut index = 0;
    while index < lines.len() {
        let (block_start, opening) = lines[index];
        if opening.trim_end_matches('\r') != "=begin" {
            index += 1;
            continue;
        }
        let Some(relative_end) = lines[index + 1..]
            .iter()
            .position(|(_, line)| line.trim_end_matches('\r') == "=end")
        else {
            break;
        };
        let block_end = index + 1 + relative_end;
        for (_, line) in &lines[index + 1..block_end] {
            let Some(after_hash) = line.strip_prefix('#') else {
                continue;
            };
            let (margin, text) = after_hash
                .strip_prefix(' ')
                .map_or((1, after_hash), |text| (2, text));
            let Some(configured) = keywords.iter().find(|keyword| {
                text.get(..keyword.len())
                    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(keyword))
                    && text
                        .as_bytes()
                        .get(keyword.len())
                        .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
            }) else {
                continue;
            };
            let keyword = &text[..configured.len()];
            let remainder = &text[configured.len()..];
            let colon = remainder.find(':').filter(|colon| {
                remainder[..*colon]
                    .bytes()
                    .all(|byte| matches!(byte, b' ' | b'\t'))
            });
            let colon_len = colon.map_or(0, |colon| colon + 1);
            let space_len = remainder[colon_len..]
                .bytes()
                .take_while(|byte| matches!(byte, b' ' | b'\t'))
                .count();
            let has_colon = colon.is_some();
            let has_space = space_len > 0;
            if !has_colon && !has_space {
                continue;
            }
            let note = &remainder[colon_len + space_len..];
            let has_note = note.bytes().next().is_some_and(|byte| !byte.is_ascii_whitespace());
            if keyword_is_capitalized(keyword) && !has_colon && has_space && has_note {
                continue;
            }
            if keyword == keyword.to_uppercase()
                && has_space
                && has_note
                && has_colon == require_colon
            {
                break;
            }
            let range = block_start + margin
                ..block_start + margin + configured.len() + colon_len + space_len;
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
                let prefix = if require_colon { ": " } else { " " };
                context.replace(
                    message,
                    range.clone(),
                    range,
                    format!("{}{prefix}", keyword.to_uppercase()),
                );
            } else {
                context.report(message, range);
            }
            break;
        }
        index = block_end + 1;
    }
}
