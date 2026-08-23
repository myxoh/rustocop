use std::collections::HashSet;

use super::*;

declare_source_cops! {
    DuplicateRescueException => "Lint/DuplicateRescueException" => duplicate_rescue,
    ImplicitRuntimeError => "Style/ImplicitRuntimeError" => implicit_runtime_error,
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

fn implicit_runtime_error(source: &str, context: &mut Reporter<'_>) {
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim_start();
        let method = if trimmed.starts_with("raise '") || trimmed.starts_with("raise \"") {
            "raise"
        } else if trimmed.starts_with("fail '") || trimmed.starts_with("fail \"") {
            "fail"
        } else {
            continue;
        };
        let start = offset + line.len() - trimmed.len();
        let end = if line.trim_end().ends_with('\\') {
            source[offset + line.len() + 1..]
                .find('\n')
                .map_or(source.len(), |next| offset + line.len() + 1 + next)
        } else {
            offset + line.len()
        };
        context.report(format!("Use `{method}` with an explicit exception class and message, rather than just a message."), start..end);
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
    for (offset, line) in source_lines(source) {
        let Some(hash) = line.find('#') else { continue };
        let comment = &line[hash + 1..];
        let Some((relative, _)) = comment.char_indices().find(|(_, character)| {
            !character.is_ascii()
                && *character != '©'
                && !allowed.iter().any(|item| item == &character.to_string())
        }) else {
            continue;
        };
        let start = offset + hash + 1 + relative;
        let mut end = start;
        for (relative, character) in source[start..offset + line.len()].char_indices() {
            if character.is_ascii()
                || character == '©'
                || allowed.iter().any(|item| item == &character.to_string())
            {
                break;
            }
            end = start + relative + character.len_utf8();
        }
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
