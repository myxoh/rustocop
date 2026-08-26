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

pub(super) fn leading_spaces(line: &str) -> usize {
    line.chars()
        .take_while(|character| *character == ' ')
        .count()
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
