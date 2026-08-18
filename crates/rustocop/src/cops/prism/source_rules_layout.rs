use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(EmptyLines),
        Box::new(SpaceBeforeComment),
        Box::new(SpaceAfterSemicolon),
        Box::new(SpaceAfterComma),
        Box::new(SpaceBeforeSemicolon),
        Box::new(SpaceAfterNot),
        Box::new(SpaceBeforeComma),
    ]
}

macro_rules! source_cop {
    ($type:ident, $name:literal, $check:ident) => {
        struct $type;
        impl Cop for $type {
            fn name(&self) -> &'static str {
                $name
            }
            fn on_source(&self, source: &str, context: &mut Context) {
                $check(self.name(), source, context);
            }
        }
    };
}

source_cop!(EmptyLines, "Layout/EmptyLines", empty_lines);
source_cop!(
    SpaceBeforeComment,
    "Layout/SpaceBeforeComment",
    space_before_comment
);
source_cop!(
    SpaceAfterSemicolon,
    "Layout/SpaceAfterSemicolon",
    space_after_semicolon
);
source_cop!(SpaceAfterComma, "Layout/SpaceAfterComma", space_after_comma);
source_cop!(
    SpaceBeforeSemicolon,
    "Layout/SpaceBeforeSemicolon",
    space_before_semicolon
);
source_cop!(SpaceAfterNot, "Layout/SpaceAfterNot", space_after_not);
source_cop!(
    SpaceBeforeComma,
    "Layout/SpaceBeforeComma",
    space_before_comma
);

fn empty_lines(cop: &'static str, source: &str, context: &mut Context) {
    for (start, _) in source.match_indices("\n\n\n") {
        if !inside_quoted_text(source, start + 2) {
            context.remove(
                cop,
                "Extra blank line detected.",
                start + 2..start + 3,
                start + 2..start + 3,
            );
        }
    }
}

fn space_before_comment(cop: &'static str, source: &str, context: &mut Context) {
    for (offset, line) in source_lines(source) {
        let Some(hash) = line.find('#') else { continue };
        if hash == 0
            || line.as_bytes()[hash - 1].is_ascii_whitespace()
            || line[..hash].contains(['"', '\''])
        {
            continue;
        }
        context.insert(
            cop,
            "Put a space before an end-of-line comment.",
            offset + hash..offset + line.len(),
            offset + hash,
            " ",
        );
    }
}

fn space_after_semicolon(cop: &'static str, source: &str, context: &mut Context) {
    spacing_after(cop, source, context, b';', "Space missing after semicolon.");
}

fn space_after_comma(cop: &'static str, source: &str, context: &mut Context) {
    spacing_after(cop, source, context, b',', "Space missing after comma.");
}

fn spacing_after(
    cop: &'static str,
    source: &str,
    context: &mut Context,
    token: u8,
    message: &'static str,
) {
    let bytes = source.as_bytes();
    for index in 0..bytes.len() {
        if bytes[index] != token {
            continue;
        }
        let Some(next) = bytes.get(index + 1).copied() else {
            continue;
        };
        let closing_brace_requires_space = next == b'}'
            && ((token == b',' && source[..index].trim_start().starts_with("{ "))
                || (token == b';' && bytes.get(index.wrapping_sub(1)) == Some(&b' ')));
        if next == b'\n'
            || next == b' '
            || next == token
            || (matches!(next, b')' | b']' | b'}' | b'|') && !closing_brace_requires_space)
            || inside_quoted_text(source, index)
        {
            continue;
        }
        context.insert(cop, message, index..index + 1, index + 1, " ");
    }
}

fn space_before_semicolon(cop: &'static str, source: &str, context: &mut Context) {
    spacing_before(cop, source, context, b';', "Space found before semicolon.");
}

fn space_before_comma(cop: &'static str, source: &str, context: &mut Context) {
    spacing_before(cop, source, context, b',', "Space found before comma.");
}

fn spacing_before(
    cop: &'static str,
    source: &str,
    context: &mut Context,
    token: u8,
    message: &'static str,
) {
    let bytes = source.as_bytes();
    for index in 1..bytes.len() {
        if bytes[index] != token || bytes[index - 1] != b' ' || inside_quoted_text(source, index) {
            continue;
        }
        let start = source[..index].trim_end_matches(' ').len();
        context.remove(cop, message, start..index, start..index);
    }
}

fn space_after_not(cop: &'static str, source: &str, context: &mut Context) {
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
            cop,
            "Do not leave space between `!` and its argument.",
            start..expression_end,
            start..expression_end,
            format!("!{}", &source[end..expression_end]),
        );
    }
}

fn source_lines(source: &str) -> impl Iterator<Item = (usize, &str)> {
    source.split_inclusive('\n').scan(0, |offset, line| {
        let start = *offset;
        *offset += line.len();
        Some((start, line.strip_suffix('\n').unwrap_or(line)))
    })
}

fn inside_quoted_text(source: &str, offset: usize) -> bool {
    let before = &source[..offset];
    before.bytes().filter(|byte| *byte == b'"').count() % 2 == 1 || before.contains("<<-")
}
