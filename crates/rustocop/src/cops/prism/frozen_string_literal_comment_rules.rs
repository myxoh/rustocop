use super::*;

define_cops! {
    FrozenStringLiteralComment => "Style/FrozenStringLiteralComment" => compatibility_source(check_frozen_string_literal_comment),
}

fn check_frozen_string_literal_comment(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    let source = context.source();
    return_if!(source.trim().is_empty() || !context.target_ruby_version().at_least(2, 3));
    let style = context.policy().enforced_style("always");
    let comment = magic_comment(source);

    match style {
        "never" => {
            let Some(comment) = comment.filter(|comment| matches!(comment.value.as_str(), "true" | "false")) else { return };
            let remove_end = if source[comment.end..].starts_with("\n\n") {
                comment.end + 2
            } else if source[comment.end..].starts_with('\n') {
                comment.end + 1
            } else {
                comment.end
            };
            context.replace(
                "Unnecessary frozen string literal comment.",
                comment.start..comment.end,
                comment.start..remove_end,
                "",
            );
        }
        "always_true" => match comment {
            Some(comment) if comment.value == "true" => {}
            Some(comment) => {
                let replacement = enabled_comment(&source[comment.start..comment.end]);
                context.replace(
                    "Frozen string literal comment must be set to `true`.",
                    comment.start..comment.end,
                    comment.start..comment.end,
                    replacement,
                )
            }
            None => insert_missing(context, "Missing magic comment `# frozen_string_literal: true`."),
        },
        _ => {
            if comment.is_none_or(|comment| !matches!(comment.value.as_str(), "true" | "false")) {
                insert_missing(context, "Missing frozen string literal comment.");
            }
        }
    }
}

struct MagicComment {
    start: usize,
    end: usize,
    value: String,
}

fn magic_comment(source: &str) -> Option<MagicComment> {
    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        let body = line.trim_end_matches('\n');
        let trimmed = body.trim_start();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            break;
        }
        let normalized = trimmed.to_ascii_lowercase().replace('-', "_");
        if let Some(key) = normalized.find("frozen_string_literal") {
            let tail = &normalized[key + "frozen_string_literal".len()..];
            if let Some(colon) = tail.find(':') {
                let value = tail[colon + 1..]
                    .trim_start()
                    .split(|character: char| character.is_whitespace() || matches!(character, ';' | '-' | '*'))
                    .next()
                    .unwrap_or_default()
                    .to_string();
                let start = offset + body.len() - trimmed.len();
                return Some(MagicComment { start, end: offset + body.len(), value });
            }
        }
        offset += line.len();
    }
    None
}

fn insert_missing(context: &mut CompatibilityCopContext<'_, '_, '_>, message: &str) {
    let source = context.source();
    let mut insert = 0;
    let mut lines = source.split_inclusive('\n');
    if let Some(first) = lines.next() {
        if first.starts_with("#!") {
            insert = first.len();
            if let Some(second) = lines.next() {
                if encoding_comment(second) {
                    insert += second.len();
                }
            }
        } else if encoding_comment(first) {
            insert = first.len();
        }
    }
    let correction = "# frozen_string_literal: true\n".to_string();
    context.replace(message, 0..1.min(source.len()), insert..insert, correction);
}

fn encoding_comment(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.starts_with("# encoding:") || lower.starts_with("# coding:") || lower.contains("-*- coding:") || lower.contains("-*- encoding:")
}

fn enabled_comment(line: &str) -> String {
    if !line.contains("-*-" ) {
        return "# frozen_string_literal: true".to_string();
    }
    let normalized = line.to_ascii_lowercase().replace('-', "_");
    let Some(key) = normalized.find("frozen_string_literal") else {
        return "# frozen_string_literal: true".to_string();
    };
    let Some(colon) = normalized[key..].find(':').map(|offset| key + offset) else {
        return "# frozen_string_literal: true".to_string();
    };
    let value_start = colon + 1 + line[colon + 1..].len() - line[colon + 1..].trim_start().len();
    let value_end = line[value_start..]
        .find(char::is_whitespace)
        .map_or(line.len(), |offset| value_start + offset);
    format!("{}true{}", &line[..value_start], &line[value_end..])
}
