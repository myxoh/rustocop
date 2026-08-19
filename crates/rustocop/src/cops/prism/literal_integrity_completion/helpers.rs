pub(super) fn top_level_entries(source: &str) -> Vec<(usize, &str)> {
    let mut entries = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut quote = None;
    let bytes = source.as_bytes();
    for (index, byte) in bytes.iter().copied().enumerate() {
        if let Some(active) = quote {
            if byte == active && bytes.get(index.wrapping_sub(1)) != Some(&b'\\') {
                quote = None;
            }
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'[' | b'{' | b'(' => depth += 1,
            b']' | b'}' | b')' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                entries.push((start, &source[start..index]));
                start = index + 1;
            }
            _ => {}
        }
    }
    entries.push((start, &source[start..]));
    entries
}

pub(super) fn matching_delimiter(source: &str, open: u8, close: u8) -> Option<usize> {
    let mut depth = 0usize;
    for (index, byte) in source.bytes().enumerate() {
        if byte == open {
            depth += 1;
        } else if byte == close {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index + 1);
            }
        }
    }
    None
}

pub(super) fn returns_after_continuation(lines: &[(usize, &str)], indent: usize) -> bool {
    for (_, line) in lines {
        if line.trim().is_empty() {
            continue;
        }
        if matches!(line.trim(), "end" | "rescue" | "ensure" | "else") {
            return true;
        }
        if line.len() - line.trim_start().len() <= indent {
            return false;
        }
    }
    false
}
