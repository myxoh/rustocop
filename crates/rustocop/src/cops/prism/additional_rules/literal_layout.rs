use super::*;

pub(super) fn empty_heredoc(source: &str, reporter: &mut Reporter<'_>) {
    for (offset, line) in source_lines(source) {
        let Some(marker) = line.find("<<") else {
            continue;
        };
        let token = line[marker + 2..].trim_start_matches(['~', '-']);
        let identifier = token
            .bytes()
            .take_while(|byte| identifier_byte(*byte))
            .count();
        if identifier == 0 {
            continue;
        }
        let label = &token[..identifier];
        let header_end = marker
            + 2
            + usize::from(
                line.as_bytes()
                    .get(marker + 2)
                    .is_some_and(|b| matches!(b, b'~' | b'-')),
            )
            + identifier;
        let body_start = offset + line.len() + 1;
        let closing_line = source[body_start..].lines().next().unwrap_or_default();
        if closing_line.trim() == label {
            let full_end = body_start
                + closing_line.len()
                + usize::from(
                    source.as_bytes().get(body_start + closing_line.len()) == Some(&b'\n'),
                );
            let quotes = if reporter.related_config_value("Style/StringLiterals", "EnforcedStyle")
                == Some("double_quotes")
            {
                "\"\""
            } else {
                "''"
            };
            let replacement = format!("{quotes}{}\n", &line[header_end..]);
            reporter.replace(
                "Use an empty string literal instead of heredoc.",
                offset + marker..offset + header_end,
                offset + marker..full_end,
                replacement,
            );
        }
    }
}

pub(super) fn space_after_method_name(source: &str, reporter: &mut Reporter<'_>) {
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim_start();
        if trimmed.starts_with("def ") {
            if let Some(paren) = line.find(" (") {
                let def_start = line.len() - trimmed.len();
                let identity_start = def_start + "def ".len();
                if paren < identity_start {
                    continue;
                }
                let identity = line[identity_start..paren].trim();
                if identity.is_empty()
                    || identity.chars().any(char::is_whitespace)
                    || identity.contains(['(', ')'])
                {
                    continue;
                }
                let start = offset + paren;
                reporter.remove(
                    "Do not put a space between a method name and the opening parenthesis.",
                    start..start + 1,
                    start..start + 1,
                );
            }
        }
    }
}
