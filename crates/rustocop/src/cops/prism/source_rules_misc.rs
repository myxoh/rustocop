use std::collections::HashSet;

use super::*;

declare_source_cops! {
    Dir => "Style/Dir" => dir_method,
    DuplicateRescueException => "Lint/DuplicateRescueException" => duplicate_rescue,
    EmptyClass => "Lint/EmptyClass" => empty_class,
    ImplicitRuntimeError => "Style/ImplicitRuntimeError" => implicit_runtime_error,
    EnvHome => "Style/EnvHome" => env_home,
    ClassCheck => "Style/ClassCheck" => class_check,
    AsciiComments => "Style/AsciiComments" => ascii_comments,
}

fn dir_method(source: &str, context: &mut Reporter<'_>) {
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim();
        let normalized = trimmed.replace("::File", "File");
        if matches!(
            normalized.as_str(),
            "File.expand_path(File.dirname(__FILE__))" | "File.dirname(File.realpath(__FILE__))"
        ) {
            let start = offset + line.len() - line.trim_start().len();
            context.replace(
                "Use `__dir__` to get an absolute path to the current file's directory.",
                start..start + trimmed.len(),
                start..start + trimmed.len(),
                "__dir__",
            );
        }
    }
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

fn empty_class(source: &str, context: &mut Reporter<'_>) {
    let allow_comments = context.config_bool("AllowComments", true);
    let lines = source_lines(source).collect::<Vec<_>>();
    for (index, (offset, line)) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !trimmed.starts_with("class ") || trimmed.contains(" < ") {
            continue;
        }
        if let Some((end_offset, end_line)) = lines[index + 1..]
            .iter()
            .find(|(_, candidate)| candidate.trim() == "end")
        {
            let body = &source[offset + line.len()..*end_offset];
            let contains_comment = body.lines().any(|line| line.trim_start().starts_with('#'));
            let empty = body
                .lines()
                .all(|line| line.trim().is_empty() || line.trim_start().starts_with('#'));
            if empty && !(allow_comments && contains_comment) {
                let message = if trimmed.starts_with("class <<") {
                    "Empty metaclass detected."
                } else {
                    "Empty class detected."
                };
                let start = offset + line.len() - line.trim_start().len();
                context.report(message, start..end_offset + end_line.len());
            }
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
        let normalized = trimmed.strip_prefix("::").unwrap_or(trimmed);
        if matches!(
            normalized,
            "ENV['HOME']"
                | "ENV[\"HOME\"]"
                | "ENV.fetch('HOME')"
                | "ENV.fetch(\"HOME\")"
                | "ENV.fetch('HOME', nil)"
                | "ENV.fetch(\"HOME\", nil)"
        ) {
            let start = offset + line.len() - line.trim_start().len();
            context.replace(
                "Use `Dir.home` instead.",
                start..start + trimmed.len(),
                start..start + trimmed.len(),
                "Dir.home",
            );
        }
    }
}

fn class_check(source: &str, context: &mut Reporter<'_>) {
    let (bad, good) = if context.policy().enforced_style("is_a?") == "kind_of?" {
        ("is_a?", "kind_of?")
    } else {
        ("kind_of?", "is_a?")
    };
    for (start, _) in source.match_indices(bad) {
        context.replace(
            format!("Prefer `Object#{good}` over `Object#{bad}`."),
            start..start + bad.len(),
            start..start + bad.len(),
            good,
        );
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
