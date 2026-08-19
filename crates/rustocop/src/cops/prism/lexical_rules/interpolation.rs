use super::*;

pub(super) fn variable_interpolation(
    node: &ruby_prism::EmbeddedVariableNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let variable = node.variable();
    let variable_source = context.source_file().node(&variable);
    context.replace(
        format!(
            "Replace interpolated variable `{variable_source}` with expression `#{{{variable_source}}}`."
        ),
        variable.location(),
        variable.location(),
        format!("{{{variable_source}}}"),
    );
}

pub(super) fn single_quoted_ranges(source: &str) -> Vec<std::ops::Range<usize>> {
    let bytes = source.as_bytes();
    let mut ranges = Vec::new();
    let mut index = 0;
    let mut double_quoted = false;
    while index < bytes.len() {
        if bytes[index] == b'"' {
            double_quoted = !double_quoted;
            index += 1;
            continue;
        }
        if bytes[index] != b'\'' || double_quoted {
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        while index < bytes.len() && bytes[index] != b'\'' {
            index += 1 + usize::from(bytes[index] == b'\\' && index + 1 < bytes.len());
        }
        if index < bytes.len() {
            ranges.push(start..index + 1);
        }
        index += 1;
    }
    ranges
}

pub(super) fn unmatched_closing_brace(content: &str) -> bool {
    let mut depth = 0_i32;
    for character in content.chars() {
        match character {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return true;
        }
    }
    depth != 0
}
