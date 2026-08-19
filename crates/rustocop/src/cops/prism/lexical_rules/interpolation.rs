use super::*;

pub(super) fn variable_interpolation(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for hash in all_offsets(source, "#") {
        if source.as_bytes().get(hash + 1) == Some(&b'{') {
            continue;
        }
        let Some(marker @ (b'$' | b'@')) = source.as_bytes().get(hash + 1).copied() else {
            continue;
        };
        let start = hash + 1;
        let mut end = start + 1;
        if marker == b'@' && source.as_bytes().get(end) == Some(&b'@') {
            end += 1;
        }
        while source
            .as_bytes()
            .get(end)
            .is_some_and(|byte| identifier_byte(*byte))
        {
            end += 1;
        }
        if marker == b'$' && end == start + 1 && source.as_bytes().get(end).is_some() {
            end += 1;
        }
        let variable = &source[start..end];
        context.replace(
            format!(
                "Replace interpolated variable `{variable}` with expression `#{{{variable}}}`."
            ),
            start..end,
            start..end,
            format!("{{{variable}}}"),
        );
    }
}

pub(super) fn interpolation_ranges(source: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut search = 0;
    while let Some(relative) = source[search..].find("#{") {
        let start = search + relative;
        let Some(close) = source[start + 2..].find('}') else {
            break;
        };
        let end = start + 2 + close + 1;
        ranges.push((start, end));
        search = end;
    }
    ranges
}

pub(super) fn percent_word_literal(source: &str, offset: usize) -> bool {
    let line_start = source[..offset].rfind('\n').map_or(0, |at| at + 1);
    source[line_start..offset].contains("%W[") || source[line_start..offset].contains("%I[")
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
