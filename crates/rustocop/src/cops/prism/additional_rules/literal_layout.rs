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

pub(super) fn space_inside_range(source: &str, reporter: &mut Reporter<'_>) {
    if let Some(dots) = source.find(" ..\n") {
        let physical_line = source[..dots].rfind('\n').map_or(0, |offset| offset + 1);
        let line_start = source[physical_line..dots]
            .trim_end()
            .rfind(char::is_whitespace)
            .map_or(physical_line, |offset| physical_line + offset + 1);
        let end = source[dots + 4..]
            .find('\n')
            .map_or(source.len(), |offset| dots + 4 + offset);
        let replacement = source[line_start..end]
            .replace(" ..\n", "..")
            .replace("    ", "");
        reporter.replace(
            "Space inside range literal.",
            line_start..end,
            line_start..end,
            replacement,
        );
        return;
    }
    for (offset, line) in source_lines(source) {
        let Some(dots) = line.find("..") else {
            continue;
        };
        let left = line[..dots].trim_end();
        let dot_count = if line[dots..].starts_with("...") { 3 } else { 2 };
        let right = line[dots + dot_count..].trim_start();
        if left.len() != dots || right.len() != line.len() - dots - dot_count {
            let start = offset + line.len() - line.trim_start().len();
            reporter.replace(
                "Space inside range literal.",
                start..offset + line.len(),
                start..offset + line.len(),
                format!("{left}{}{right}", ".".repeat(dot_count)),
            );
        }
    }
}

pub(super) fn space_after_method_name(source: &str, reporter: &mut Reporter<'_>) {
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim_start();
        if trimmed.starts_with("def ") {
            if let Some(paren) = line.find(" (") {
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

pub(super) fn redundant_constant_base(source: &str, reporter: &mut Reporter<'_>) {
    if reporter.related_config_value("Lint/ConstantResolution", "Enabled") == Some("true") {
        return;
    }
    let nested_class = source.starts_with("class ") || source.starts_with("module ");
    for start in all_offsets(source, "::") {
        if start > 0
            && (identifier_byte(source.as_bytes()[start - 1])
                || source.as_bytes()[start - 1] == b':')
        {
            continue;
        }
        let line_start = source[..start].rfind('\n').map_or(0, |offset| offset + 1);
        let prefix = source[line_start..start].trim();
        let superclass = prefix.ends_with('<');
        let singleton = source[..line_start].contains("class << self");
        if start == 0 || superclass || singleton || !nested_class {
            reporter.remove("Remove redundant `::`.", start..start + 2, start..start + 2);
        }
    }
}
