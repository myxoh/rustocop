pub(super) fn ternary_colon(source: &str, question: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut nested = 0usize;
    let mut index = question + 1;
    while index < bytes.len() {
        if bytes[index] == b'?' && index > 0 && bytes[index - 1].is_ascii_whitespace() {
            nested += 1;
        } else if bytes[index] == b':' {
            if nested == 0 {
                return Some(index);
            }
            nested -= 1;
        }
        index += 1;
    }
    None
}

pub(super) fn convert_multiline_ternary(source: &str) -> Option<String> {
    if !source.contains('\n') {
        return None;
    }
    let question = source.find(" ?").map(|at| at + 1)?;
    let colon = ternary_colon(source, question)?;
    let condition = source[..question].trim();
    let truthy = source[question + 1..colon].trim();
    let falsey = source[colon + 1..].trim();
    let falsey = convert_multiline_ternary(falsey).unwrap_or_else(|| falsey.to_string());
    Some(format!("if {condition}\n  {truthy}\nelse\n  {falsey}\nend"))
}
