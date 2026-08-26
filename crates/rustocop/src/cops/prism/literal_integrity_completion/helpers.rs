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
