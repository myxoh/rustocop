use super::*;

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

fn numeric_literals(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if node.as_integer_node().is_none() && node.as_float_node().is_none() {
        return;
    }
    let location = node.location();
    let line_start = context.source_file().line_start(location.start_offset());
    let line_end = context.source()[line_start..]
        .find('\n')
        .map_or(context.source().len(), |at| line_start + at);
    if context.source()[line_start..line_end].contains("rubocop:disable Style/NumericLiterals") {
        return;
    }
    let source = context.source_file().at(&location);
    let unsigned = source.strip_prefix('-').unwrap_or(source);
    let integer = unsigned.split(['.', 'e', 'E']).next().unwrap_or(unsigned);
    if integer.starts_with('0') {
        return;
    }
    if context
        .config_values("AllowedPatterns")
        .iter()
        .any(|pattern| {
            let pattern = pattern.replace("\\\\", "\\");
            regex::Regex::new(&format!("^(?:{pattern})$"))
                .is_ok_and(|pattern| pattern.is_match(integer))
        })
    {
        return;
    }
    let digits = integer.replace('_', "");
    let minimum = context.config_usize("MinDigits", 5);
    if digits.len() < minimum || context.config_values("AllowedNumbers").contains(&digits) {
        return;
    }
    let groups = integer.split('_').collect::<Vec<_>>();
    let strict = context.config_bool("Strict", false);
    let valid = groups.len() > 1
        && groups
            .first()
            .is_some_and(|group| (1..=3).contains(&group.len()))
        && groups.iter().skip(1).all(|group| group.len() == 3);
    let tolerated = !strict
        && groups.len() > 2
        && groups[1..groups.len() - 1]
            .iter()
            .all(|group| group.len() == 3);
    if valid || tolerated {
        return;
    }
    let mut formatted = String::new();
    for (index, byte) in digits.bytes().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            formatted.push('_');
        }
        formatted.push(byte as char);
    }
    let formatted = formatted.chars().rev().collect::<String>();
    let replacement = source.replacen(integer, &formatted, 1);
    let extended_start = context.source()[..location.start_offset()]
        .rfind('-')
        .filter(|minus| {
            context.source()[*minus + 1..location.start_offset()]
                .bytes()
                .all(|byte| byte.is_ascii_whitespace())
        })
        .unwrap_or(location.start_offset());
    context.replace(
        "Use underscores(_) as thousands separator and separate every 3 digits with them.",
        extended_start..location.end_offset(),
        extended_start..location.end_offset(),
        if extended_start < location.start_offset() {
            format!("-{replacement}")
        } else {
            replacement
        },
    );
}

fn command_literal(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if node.as_x_string_node().is_none() && node.as_interpolated_x_string_node().is_none() {
        return;
    }
    let location = node.location();
    let source = context.source_file().at(&location);
    if source.starts_with("<<") {
        return;
    }
    let backticks = source.starts_with('`');
    let contains_backtick =
        source[usize::from(backticks)..source.len().saturating_sub(1)].contains('`');
    let allow_inner_backticks = context.config_bool("AllowInnerBackticks", false);
    let style = context.policy().enforced_style("backticks");
    let allowed = match style {
        "backticks" => {
            backticks && (!contains_backtick || allow_inner_backticks)
                || !backticks && contains_backtick && !allow_inner_backticks
        }
        "percent_x" => !backticks,
        "mixed" => {
            backticks && !source.contains('\n') && (!contains_backtick || allow_inner_backticks)
                || !backticks
                    && (source.contains('\n') || contains_backtick && !allow_inner_backticks)
        }
        _ => true,
    };
    if allowed {
        return;
    }
    let (message, replacement) = if backticks {
        let body = source.trim_matches('`');
        let replacement = if contains_backtick {
            None
        } else {
            let delimiters = context
                .related_config_map("Style/PercentLiteralDelimiters", "PreferredDelimiters")
                .and_then(|values| values.get("%x").or_else(|| values.get("default")))
                .map(String::as_str)
                .unwrap_or("()");
            let (open, close) = delimiters.split_at(1);
            Some(format!("%x{open}{body}{close}"))
        };
        ("Use `%x` around command string.", replacement)
    } else {
        let body_start = 2;
        let body = &source[body_start + 1..source.len() - 1];
        let replacement = (!contains_backtick).then(|| format!("`{body}`"));
        ("Use backticks around command string.", replacement)
    };
    if let Some(replacement) = replacement {
        context.replace(message, &location, &location, replacement);
    } else {
        context.report(message, &location);
    }
}
