use crate::model::SourceLine;

pub(super) fn trailing_whitespace_len(value: &str) -> usize {
    value
        .chars()
        .rev()
        .take_while(|character| matches!(character, ' ' | '\t' | '\u{3000}'))
        .count()
}

pub(super) fn trim_trailing_spaces(value: &mut String) {
    while value.ends_with(' ') || value.ends_with('\t') || value.ends_with('\u{3000}') {
        value.pop();
    }
}

pub(super) fn strip_comment(line: &str) -> &str {
    line.split('#').next().unwrap_or(line)
}

pub(super) fn find_extra_spacing(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    for index in 1..bytes.len().saturating_sub(1) {
        if bytes[index] == b' ' && bytes[index + 1] == b' ' {
            let previous = bytes[index - 1] as char;
            let next = bytes.get(index + 2).copied().unwrap_or_default() as char;
            if previous.is_ascii_alphanumeric() && (next.is_ascii_alphanumeric() || next == '=') {
                return Some(index + 1);
            }
        }
    }
    None
}

pub(super) fn leading_spaces(line: &str) -> usize {
    line.chars()
        .take_while(|character| *character == ' ')
        .count()
}

pub(super) fn starts_block(trimmed: &str) -> bool {
    trimmed.starts_with("def ")
        || trimmed.starts_with("class ")
        || trimmed.starts_with("module ")
        || trimmed.starts_with("if ")
        || trimmed.starts_with("unless ")
        || trimmed.starts_with("case")
        || trimmed.starts_with("begin")
        || trimmed.ends_with(" do")
        || trimmed.contains(" do |")
}

pub(super) fn assignment_name(trimmed: &str) -> Option<String> {
    if trimmed.contains("==")
        || trimmed.contains("!=")
        || trimmed.contains(">=")
        || trimmed.contains("<=")
    {
        return None;
    }

    let position = trimmed.find('=')?;
    let name = trimmed[..position].trim();
    if name
        .chars()
        .all(|character| character == '_' || character.is_ascii_alphanumeric())
    {
        Some(name.to_string())
    } else {
        None
    }
}

pub(super) fn find_numbered_parameter(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    for index in 0..bytes.len().saturating_sub(1) {
        if bytes[index] == b'_'
            && bytes[index + 1].is_ascii_digit()
            && (index == 0 || !bytes[index - 1].is_ascii_alphanumeric())
        {
            return Some(index + 1);
        }
    }

    None
}

pub(super) fn find_single_quoted_literal(line: &str) -> Option<usize> {
    let mut escaped = false;
    let mut in_double = false;

    for (index, character) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        if character == '\\' {
            escaped = true;
            continue;
        }

        if character == '"' {
            in_double = !in_double;
        } else if character == '\'' && !in_double {
            return Some(index + 1);
        } else if character == '#' && !in_double {
            break;
        }
    }

    None
}

pub(super) fn first_identifier(value: &str) -> Option<&str> {
    let end = value
        .find(|character: char| {
            !(character == '_'
                || character == '?'
                || character == '!'
                || character.is_ascii_alphanumeric())
        })
        .unwrap_or(value.len());
    let name = &value[..end];
    (!name.is_empty()).then_some(name)
}

pub(super) fn method_arguments(signature: &str) -> Vec<String> {
    let Some(start) = signature.find('(') else {
        return Vec::new();
    };
    let Some(end) = signature.rfind(')') else {
        return Vec::new();
    };

    signature[start + 1..end]
        .split(',')
        .filter_map(|arg| {
            let arg = arg
                .trim()
                .trim_start_matches('*')
                .trim_start_matches('&')
                .split(':')
                .next()
                .unwrap_or_default()
                .split('=')
                .next()
                .unwrap_or_default()
                .trim();

            if arg.is_empty() {
                None
            } else {
                Some(arg.to_string())
            }
        })
        .collect()
}

pub(super) fn find_matching_end(lines: &[SourceLine], start: usize) -> Option<usize> {
    let mut depth = 0usize;

    for (index, line) in lines.iter().enumerate().skip(start) {
        let trimmed = line.body.trim();
        if starts_block(trimmed) {
            depth += 1;
        }

        if trimmed == "end" {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(index);
            }
        }
    }

    None
}

pub(super) fn is_rspec_group_start(trimmed: &str) -> bool {
    ["describe ", "context ", "feature ", "RSpec.describe "]
        .iter()
        .any(|prefix| trimmed.starts_with(prefix))
}

pub(super) fn is_rspec_example_start(trimmed: &str) -> bool {
    ["it ", "specify ", "example ", "scenario "]
        .iter()
        .any(|prefix| trimmed.starts_with(prefix))
}

pub(super) fn symbol_argument(trimmed: &str) -> Option<&str> {
    let start = trimmed.find("(:")? + 2;
    let rest = &trimmed[start..];
    let end = rest
        .find(|character: char| !(character == '_' || character.is_ascii_alphanumeric()))
        .unwrap_or(rest.len());
    let name = &rest[..end];
    (!name.is_empty()).then_some(name)
}

pub(super) fn is_snake_case(name: &str) -> bool {
    name.chars().all(|character| {
        character == '_' || character.is_ascii_lowercase() || character.is_ascii_digit()
    })
}

pub(super) fn pending_without_reason(trimmed: &str) -> bool {
    if !(trimmed == "pending"
        || trimmed == "skip"
        || trimmed.starts_with("pending ")
        || trimmed.starts_with("skip "))
    {
        return false;
    }

    !(trimmed.contains('"') || trimmed.contains('\'') || trimmed.contains(':'))
}
