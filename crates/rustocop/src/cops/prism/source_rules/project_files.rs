use std::collections::HashMap;

use super::*;

pub(super) fn duplicated_gem(source: &str, context: &mut Reporter<'_>) {
    if std::path::Path::new(context.path())
        .file_name()
        .is_none_or(|name| name != "Gemfile")
    {
        return;
    }
    let mut first = HashMap::<String, (usize, usize)>::new();
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("gem ") && !trimmed.starts_with("gem(") {
            continue;
        }
        let Some(quote) = trimmed.find(['\'', '"']) else {
            continue;
        };
        let delimiter = trimmed.as_bytes()[quote] as char;
        let Some(end_quote) = trimmed[quote + 1..].find(delimiter) else {
            continue;
        };
        let name = &trimmed[quote + 1..quote + 1 + end_quote];
        let indent = line.len() - trimmed.len();
        if let Some((first_line, first_indent)) = first.get(name).copied() {
            if first_indent == 0 {
                let start = offset + indent;
                context.report(format!("Gem `{name}` requirements already given on line {first_line} of the Gemfile."), start..offset + line.len());
            }
        } else {
            first.insert(
                name.to_string(),
                (
                    source[..offset].bytes().filter(|b| *b == b'\n').count() + 1,
                    indent,
                ),
            );
        }
    }
}

pub(super) fn string_hash_keys(source: &str, context: &mut Reporter<'_>) {
    if source.contains("popen(")
        || source.contains("capture3(")
        || source.contains("pipeline(")
        || source.contains("gsub")
    {
        return;
    }
    for start in find_all(source, "'") {
        let Some(relative_end) = source[start + 1..].find('\'') else {
            continue;
        };
        let end = start + 1 + relative_end + 1;
        if !source[end..].trim_start().starts_with("=>") {
            continue;
        }
        let value = &source[start + 1..end - 1];
        let replacement = if value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            format!(":{value}")
        } else {
            format!(":\"{value}\"")
        };
        context.replace(
            "Prefer symbols instead of strings as hash keys.",
            start..end,
            start..end,
            replacement,
        );
    }
}
