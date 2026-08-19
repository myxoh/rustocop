use std::ops::Range;

use super::source_helpers::{all_offsets, source_lines};

pub(super) struct Definition<'source> {
    pub(super) name: &'source str,
    pub(super) arguments: Range<usize>,
}

pub(super) fn definitions(source: &str) -> Vec<Definition<'_>> {
    let mut definitions = Vec::new();
    for (start, line) in source_lines(source) {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("def ") {
            continue;
        }
        let Some(open) = line.find('(') else {
            continue;
        };
        let Some(close) = line.rfind(')') else {
            continue;
        };
        let name_start = trimmed.find(' ').unwrap_or(0) + 1;
        let name = trimmed[name_start..]
            .split('(')
            .next()
            .unwrap_or_default()
            .rsplit('.')
            .next()
            .unwrap_or_default();
        definitions.push(Definition {
            name,
            arguments: start + open + 1..start + close,
        });
    }
    definitions
}

pub(super) fn call_ranges(source: &str, needle: &str) -> Vec<Range<usize>> {
    all_offsets(source, needle)
        .filter_map(|start| {
            let open = start + needle.len() - 1;
            matching_delimiter(source, open, b'(', b')').map(|close| start..close + 1)
        })
        .collect()
}

pub(super) fn matching_delimiter(source: &str, open: usize, left: u8, right: u8) -> Option<usize> {
    let mut depth = 0_usize;
    let mut quote = None;
    for (index, byte) in source.as_bytes().iter().copied().enumerate().skip(open) {
        if let Some(delimiter) = quote {
            if byte == delimiter && source.as_bytes().get(index.wrapping_sub(1)) != Some(&b'\\') {
                quote = None;
            }
            continue;
        }
        if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
        } else if byte == left {
            depth += 1;
        } else if byte == right {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

pub(super) fn split_arguments(source: &str, start: usize, end: usize) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut element = start;
    let mut depths = (0_i32, 0_i32, 0_i32);
    let mut quote = None;
    for (relative, character) in source[start..end].char_indices() {
        let offset = start + relative;
        if let Some(delimiter) = quote {
            if character == delimiter
                && source.as_bytes().get(offset.wrapping_sub(1)) != Some(&b'\\')
            {
                quote = None;
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            quote = Some(character);
            continue;
        }
        match character {
            '(' => depths.0 += 1,
            ')' => depths.0 -= 1,
            '[' => depths.1 += 1,
            ']' => depths.1 -= 1,
            '{' => depths.2 += 1,
            '}' => depths.2 -= 1,
            ',' if depths == (0, 0, 0) => {
                ranges.push(element..offset);
                element = offset + 1;
            }
            _ => {}
        }
    }
    if !source[element..end].trim().is_empty() {
        ranges.push(element..end);
    }
    ranges
}

pub(super) fn top_level_elements(source: &str, start: usize, end: usize) -> Vec<Range<usize>> {
    split_arguments(source, start, end)
        .into_iter()
        .map(|range| trim_range(source, range))
        .collect()
}

pub(super) fn trim_range(source: &str, range: Range<usize>) -> Range<usize> {
    let value = &source[range.clone()];
    let leading = value.len() - value.trim_start().len();
    let trailing = value.len() - value.trim_end().len();
    range.start + leading..range.end - trailing
}

pub(super) fn first_quoted(source: &str) -> Option<&str> {
    let (start, quote) = source
        .char_indices()
        .find(|(_, character)| matches!(character, '\'' | '"'))?;
    let rest = &source[start + 1..];
    let end = rest.find(quote)?;
    Some(&rest[..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_only_top_level_arguments() {
        let source = "one, call(two, three), [four, five], 'six, seven'";
        assert_eq!(
            split_arguments(source, 0, source.len())
                .into_iter()
                .map(|range| source[range].trim())
                .collect::<Vec<_>>(),
            ["one", "call(two, three)", "[four, five]", "'six, seven'"]
        );
    }

    #[test]
    fn finds_nested_call_boundaries() {
        let source = "before Hash.new(call(one, two)) after";
        let ranges = call_ranges(source, "Hash.new(");
        assert_eq!(&source[ranges[0].clone()], "Hash.new(call(one, two))");
    }
}
