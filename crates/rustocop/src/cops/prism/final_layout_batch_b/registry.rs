use super::super::catalog_cop::{custom, replace};
use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        replace(
            "Layout/SpaceInsideArrayPercentLiteral",
            "%w[ ",
            "%w[",
            "Space inside percent array detected.",
        ),
        custom("Layout/RescueEnsureAlignment", end_alignment),
        custom("Layout/HashAlignment", hash_alignment),
        custom("Layout/SpaceAroundOperators", operator_spacing),
        custom(
            "Layout/EmptyLinesAroundAccessModifier",
            empty_around_access_modifier,
        ),
        custom("Layout/HeredocIndentation", heredoc_indentation),
        custom("Layout/SpaceAroundKeyword", space_around_keyword),
        custom("Layout/FirstArgumentIndentation", continuation_indentation),
        custom(
            "Layout/MultilineMethodCallIndentation",
            continuation_indentation,
        ),
        replace(
            "Layout/SpaceInsidePercentLiteralDelimiters",
            "%w( ",
            "%w(",
            "Space inside percent literal delimiters detected.",
        ),
    ]
}

fn space_around_keyword(context: &mut CopContext<'_, '_>) {
    const KEYWORDS: &[&str] = &[
        "defined?", "BEGIN", "END", "and", "begin", "break", "case", "do", "else",
        "elsif", "end", "ensure", "for", "if", "in", "next", "not", "or", "rescue",
        "return", "super", "then", "unless", "until", "when", "while", "yield",
    ];
    let source = context.source();
    let literal_ranges = context.source_file().literal_ranges();
    for keyword in KEYWORDS {
        for start in context.source_file().code_offsets(keyword) {
            if literal_ranges
                .iter()
                .any(|range| range.start <= start && start < range.end)
            {
                continue;
            }
            let end = start + keyword.len();
            let before = source.as_bytes().get(start.wrapping_sub(1)).copied();
            let after = source.as_bytes().get(end).copied();
            if matches!(before, Some(b'.' | b':'))
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
                Some(byte) => !byte.is_ascii_whitespace()
                    && !matches!(byte, b'.' | b':' | b'[' | b'#' | b';' | b'\\' | b',' | b')' | b']' | b'}' | b'&'),
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
