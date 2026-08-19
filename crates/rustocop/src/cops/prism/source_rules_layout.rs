use super::source_helpers::source_lines;
use super::*;

declare_source_cops! {
    EmptyLines => "Layout/EmptyLines" => empty_lines,
    SpaceBeforeComment => "Layout/SpaceBeforeComment" => space_before_comment,
    SpaceAfterSemicolon => "Layout/SpaceAfterSemicolon" => space_after_semicolon,
    SpaceAfterComma => "Layout/SpaceAfterComma" => space_after_comma,
    SpaceBeforeSemicolon => "Layout/SpaceBeforeSemicolon" => space_before_semicolon,
    SpaceAfterNot => "Layout/SpaceAfterNot" => space_after_not,
    SpaceBeforeComma => "Layout/SpaceBeforeComma" => space_before_comma,
}

fn empty_lines(source: &str, context: &mut Reporter<'_>) {
    for (start, _) in source.match_indices("\n\n\n") {
        if !inside_quoted_text(source, start + 2) {
            context.remove(
                "Extra blank line detected.",
                start + 2..start + 3,
                start + 2..start + 3,
            );
        }
    }
}

fn space_before_comment(source: &str, context: &mut Reporter<'_>) {
    for (offset, line) in source_lines(source) {
        let Some(hash) = line.find('#') else { continue };
        if hash == 0
            || line.as_bytes()[hash - 1].is_ascii_whitespace()
            || line[..hash].contains(['"', '\''])
        {
            continue;
        }
        context.insert(
            "Put a space before an end-of-line comment.",
            offset + hash..offset + line.len(),
            offset + hash,
            " ",
        );
    }
}

fn space_after_semicolon(source: &str, context: &mut Reporter<'_>) {
    spacing_after(source, context, b';', "Space missing after semicolon.");
}

fn space_after_comma(source: &str, context: &mut Reporter<'_>) {
    spacing_after(source, context, b',', "Space missing after comma.");
}

fn spacing_after(source: &str, context: &mut Reporter<'_>, token: u8, message: &'static str) {
    let bytes = source.as_bytes();
    for index in 0..bytes.len() {
        if bytes[index] != token {
            continue;
        }
        let Some(next) = bytes.get(index + 1).copied() else {
            continue;
        };
        let no_space_inside_braces = token == b';'
            && next == b'}'
            && context.related_config_value("Layout/SpaceInsideBlockBraces", "EnforcedStyle")
                == Some("no_space");
        let closing_brace_requires_space = next == b'}'
            && ((token == b',' && source[..index].trim_start().starts_with("{ "))
                || (token == b';' && bytes.get(index.wrapping_sub(1)) == Some(&b' ')));
        if next == b'\n'
            || next == b' '
            || next == token
            || no_space_inside_braces
            || (matches!(next, b')' | b']' | b'}' | b'|') && !closing_brace_requires_space)
            || inside_quoted_text(source, index)
        {
            continue;
        }
        context.insert(message, index..index + 1, index + 1, " ");
    }
}

fn space_before_semicolon(source: &str, context: &mut Reporter<'_>) {
    spacing_before(source, context, b';', "Space found before semicolon.");
}

fn space_before_comma(source: &str, context: &mut Reporter<'_>) {
    spacing_before(source, context, b',', "Space found before comma.");
}

fn spacing_before(source: &str, context: &mut Reporter<'_>, token: u8, message: &'static str) {
    let bytes = source.as_bytes();
    for index in 1..bytes.len() {
        if bytes[index] != token || bytes[index - 1] != b' ' || inside_quoted_text(source, index) {
            continue;
        }
        let start = source[..index].trim_end_matches(' ').len();
        if token == b';'
            && source.as_bytes().get(start.wrapping_sub(1)) == Some(&b'{')
            && context.related_config_value("Layout/SpaceInsideBlockBraces", "EnforcedStyle")
                == Some("space")
        {
            continue;
        }
        context.remove(message, start..index, start..index);
    }
}

fn space_after_not(source: &str, context: &mut Reporter<'_>) {
    for (start, _) in source.match_indices('!') {
        if source
            .as_bytes()
            .get(start + 1)
            .is_none_or(|byte| !byte.is_ascii_whitespace())
        {
            continue;
        }
        let end = source[start + 1..]
            .find(|c: char| !c.is_whitespace())
            .map_or(source.len(), |offset| start + 1 + offset);
        let expression_end = source[end..]
            .find('\n')
            .map_or(source.len(), |offset| end + offset);
        context.replace(
            "Do not leave space between `!` and its argument.",
            start..expression_end,
            start..expression_end,
            format!("!{}", &source[end..expression_end]),
        );
    }
}

fn inside_quoted_text(source: &str, offset: usize) -> bool {
    let before = &source[..offset];
    before.bytes().filter(|byte| *byte == b'"').count() % 2 == 1 || before.contains("<<-")
}
