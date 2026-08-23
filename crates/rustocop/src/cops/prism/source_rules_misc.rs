use std::collections::HashSet;

use super::*;

declare_source_cops! {
    DuplicateRescueException => "Lint/DuplicateRescueException" => duplicate_rescue,
    EnvHome => "Style/EnvHome" => env_home,
    AsciiComments => "Style/AsciiComments" => ascii_comments,
}

fn duplicate_rescue(source: &str, context: &mut Reporter<'_>) {
    let mut seen = HashSet::<String>::new();
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim_start();
        let Some(list) = trimmed.strip_prefix("rescue ") else {
            continue;
        };
        let list_start = offset + line.len() - trimmed.len() + 7;
        let mut cursor = 0;
        for item in list.split(',') {
            let name = item.trim();
            let relative = list[cursor..].find(name).unwrap_or(0) + cursor;
            if !seen.insert(name.to_string()) {
                context.report(
                    "Duplicate `rescue` exception detected.",
                    list_start + relative..list_start + relative + name.len(),
                );
            }
            cursor = relative + name.len();
        }
    }
}

fn env_home(source: &str, context: &mut Reporter<'_>) {
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim();
        let expression = trimmed
            .rsplit_once('=')
            .map_or(trimmed, |(_, right)| right.trim());
        let normalized = expression.strip_prefix("::").unwrap_or(expression);
        if matches!(
            normalized,
            "ENV['HOME']"
                | "ENV[\"HOME\"]"
                | "ENV.fetch('HOME')"
                | "ENV.fetch(\"HOME\")"
                | "ENV.fetch('HOME', nil)"
                | "ENV.fetch(\"HOME\", nil)"
        ) {
            let start = offset + line.find(expression).unwrap_or(0);
            context.replace(
                "Use `Dir.home` instead.",
                start..start + expression.len(),
                start..start + expression.len(),
                "Dir.home",
            );
        }
    }
}

fn ascii_comments(source: &str, context: &mut Reporter<'_>) {
    let allowed = context.config_values("AllowedChars").to_vec();
    let parsed = ruby_prism::parse(source.as_bytes());
    for comment in parsed.comments() {
        let location = comment.location();
        let text = source
            .get(location.start_offset()..location.end_offset())
            .unwrap_or_default();
        let disallowed = |character: char| {
            !character.is_ascii()
                && character != '©'
                && !allowed.iter().any(|item| item == &character.to_string())
        };
        if !text.chars().any(disallowed) {
            continue;
        }
        let Some((relative, _)) = text.char_indices().find(|(_, character)| !character.is_ascii())
        else {
            continue;
        };
        let start = location.start_offset() + relative;
        let end = source[start..location.end_offset()]
            .char_indices()
            .take_while(|(_, character)| !character.is_ascii())
            .last()
            .map_or(start, |(relative, character)| {
                start + relative + character.len_utf8()
            });
        context.report("Use only ascii symbols in comments.", start..end);
    }
}

fn source_lines(source: &str) -> impl Iterator<Item = (usize, &str)> {
    source.split_inclusive('\n').scan(0, |offset, line| {
        let start = *offset;
        *offset += line.len();
        Some((start, line.strip_suffix('\n').unwrap_or(line)))
    })
}
