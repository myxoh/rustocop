use super::super::catalog_cop::custom;
use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        custom(
            "Layout/SpaceInsideArrayPercentLiteral",
            array_percent_literal_spacing,
        ),
        custom("Layout/RescueEnsureAlignment", end_alignment),
        custom("Layout/HashAlignment", hash_alignment),
        custom("Layout/SpaceAroundOperators", operator_spacing),
        custom("Layout/HeredocIndentation", heredoc_indentation),
        custom("Layout/SpaceAroundKeyword", space_around_keyword),
        custom(
            "Layout/MultilineMethodCallIndentation",
            continuation_indentation,
        ),
        custom(
            "Layout/SpaceInsidePercentLiteralDelimiters",
            percent_literal_delimiter_spacing,
        ),
    ]
}

fn array_percent_literal_spacing(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let bytes = source.as_bytes();
    let mut start = 0;
    while start + 2 < bytes.len() {
        if bytes[start] != b'%' || !matches!(bytes[start + 1], b'i' | b'I' | b'w' | b'W') {
            start += 1;
            continue;
        }
        let opening_at = start + 2;
        let opening = bytes[opening_at];
        if opening.is_ascii_alphanumeric() || opening.is_ascii_whitespace() || opening == b'=' {
            start += 1;
            continue;
        }
        let closing = match opening {
            b'(' => b')',
            b'[' => b']',
            b'{' => b'}',
            b'<' => b'>',
            byte => byte,
        };
        let paired = opening != closing;
        let mut depth = 1_usize;
        let mut escaped = false;
        let mut closing_at = opening_at + 1;
        while closing_at < bytes.len() {
            let byte = bytes[closing_at];
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if paired && byte == opening {
                depth += 1;
            } else if byte == closing {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            closing_at += 1;
        }
        if closing_at == bytes.len() {
            break;
        }

        let mut offset = opening_at + 1;
        while offset < closing_at {
            if bytes[offset] != b' ' {
                offset += 1;
                continue;
            }
            let run_start = offset;
            while offset < closing_at && bytes[offset] == b' ' {
                offset += 1;
            }
            if offset == closing_at || bytes[offset].is_ascii_whitespace() {
                continue;
            }
            let escaped_first = run_start > opening_at + 1
                && bytes[run_start - 1] == b'\\'
                && source[opening_at + 1..run_start - 1]
                    .bytes()
                    .rev()
                    .take_while(|byte| *byte == b'\\')
                    .count()
                    % 2
                    == 0;
            let offense_start = run_start + usize::from(escaped_first);
            if offset.saturating_sub(offense_start) >= 2
                && run_start > opening_at + 1
                && (escaped_first || !bytes[run_start - 1].is_ascii_whitespace())
            {
                context.replace(
                    "Use only a single space inside array percent literal.",
                    offense_start..offset,
                    offense_start..offset,
                    " ",
                );
            }
        }
        start = closing_at + 1;
    }
}

fn percent_literal_delimiter_spacing(context: &mut CopContext<'_, '_>) {
    const MESSAGE: &str = "Do not use spaces inside percent literal delimiters.";

    let source = context.source();
    let bytes = source.as_bytes();
    let mut start = 0;
    while start + 2 < bytes.len() {
        if bytes[start] != b'%' || !matches!(bytes[start + 1], b'i' | b'I' | b'w' | b'W' | b'x') {
            start += 1;
            continue;
        }

        let opening_at = start + 2;
        let opening = bytes[opening_at];
        if opening.is_ascii_alphanumeric() || opening.is_ascii_whitespace() || opening == b'=' {
            start += 1;
            continue;
        }
        let closing = match opening {
            b'(' => b')',
            b'[' => b']',
            b'{' => b'}',
            b'<' => b'>',
            byte => byte,
        };
        let paired = opening != closing;
        let mut depth = 1_usize;
        let mut escaped = false;
        let mut closing_at = opening_at + 1;
        while closing_at < bytes.len() {
            let byte = bytes[closing_at];
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if paired && byte == opening {
                depth += 1;
            } else if byte == closing {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            closing_at += 1;
        }
        if closing_at == bytes.len() {
            break;
        }

        let body = opening_at + 1..closing_at;
        let body_source = &source[body.clone()];
        if !body_source.is_empty() && body_source.trim().is_empty() {
            context.remove(MESSAGE, body.clone(), body);
        } else if !body_source.contains('\n') {
            let leading = body_source.bytes().take_while(|byte| *byte == b' ').count();
            if leading > 0 {
                let range = body.start..body.start + leading;
                context.remove(MESSAGE, range.clone(), range);
            }
            let trailing = body_source
                .bytes()
                .rev()
                .take_while(|byte| *byte == b' ')
                .count();
            if trailing > 0 {
                let trailing_start = body.end - trailing;
                let escaped_space = trailing_start > body.start
                    && bytes[trailing_start - 1] == b'\\'
                    && source[body.start..trailing_start - 1]
                        .bytes()
                        .rev()
                        .take_while(|byte| *byte == b'\\')
                        .count()
                        % 2
                        == 0;
                let range_start = if escaped_space {
                    trailing_start + 1
                } else {
                    trailing_start
                };
                if range_start < body.end {
                    let range = range_start..body.end;
                    context.remove(MESSAGE, range.clone(), range);
                }
            }
        }
        start = closing_at + 1;
    }
}

fn space_around_keyword(context: &mut CopContext<'_, '_>) {
    const KEYWORDS: &[&str] = &[
        "defined?", "BEGIN", "END", "and", "begin", "break", "case", "do", "else", "elsif", "end",
        "ensure", "for", "if", "in", "next", "not", "or", "rescue", "return", "super", "then",
        "unless", "until", "when", "while", "yield",
    ];
    let source = context.source();
    let file = context.source_file();
    let literal_ranges = file.literal_ranges();
    let comment_ranges = file.comment_ranges();
    let data_section_start = file.data_section_start();
    for keyword in KEYWORDS {
        for start in context.source_file().code_offsets(keyword) {
            if data_section_start.is_some_and(|data| data <= start)
                || comment_ranges
                    .iter()
                    .any(|range| range.start <= start && start < range.end)
                || literal_ranges
                    .iter()
                    .any(|range| range.start <= start && start < range.end)
            {
                continue;
            }
            let end = start + keyword.len();
            let before = source.as_bytes().get(start.wrapping_sub(1)).copied();
            let after = source.as_bytes().get(end).copied();
            let line = file.line(start).trim_start();
            if matches!(after, Some(b'?' | b'!'))
                || line.starts_with(&format!("def {keyword}"))
                || *keyword == "then" && line.starts_with("when ")
                || matches!(before, Some(b'.' | b':'))
                || before.is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
                || *keyword != "defined?"
                    && after.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            {
                continue;
            }

            let missing_before = before.is_some_and(|byte| {
                byte.is_ascii_digit() || matches!(byte, b'\'' | b'"' | b')' | b']' | b'}')
            });
            if missing_before {
                context.insert(
                    format!("Space before keyword `{keyword}` is missing."),
                    start..end,
                    start,
                    " ",
                );
                continue;
            }

            let missing_after = match after {
                Some(b'\'' | b'"') => true,
                Some(b'|') => *keyword == "do",
                Some(b'+') => *keyword == "begin",
                Some(b'{') => matches!(*keyword, "BEGIN" | "END" | "super"),
                Some(b'(') => matches!(*keyword, "and" | "or" | "return"),
                Some(byte) => {
                    !byte.is_ascii_whitespace()
                        && !matches!(
                            byte,
                            b'.' | b':'
                                | b'['
                                | b'#'
                                | b';'
                                | b'\\'
                                | b','
                                | b')'
                                | b']'
                                | b'}'
                                | b'&'
                        )
                }
                None => false,
            };
            if missing_after {
                context.insert(
                    format!("Space after keyword `{keyword}` is missing."),
                    start..end,
                    end,
                    " ",
                );
            }
        }
    }
}
