use super::*;

pub(super) fn redundant_self_assignment(context: &mut CopContext<'_, '_>) {
    let mutating = [
        "append",
        "clear",
        "collect!",
        "compare_by_identity",
        "concat",
        "delete_if",
        "fill",
        "initialize_copy",
        "insert",
        "keep_if",
        "map!",
        "merge!",
        "prepend",
        "push",
        "rehash",
        "replace",
        "reverse!",
        "rotate!",
        "shuffle!",
        "sort!",
        "sort_by!",
        "transform_keys!",
        "transform_values!",
        "unshift",
        "update",
    ];
    let source = context.source();
    for spacing_start in context.source_file().code_offsets(" = ") {
        let equals = spacing_start + 1;
        let line_start = source[..spacing_start].rfind('\n').map_or(0, |at| at + 1);
        let left_start = line_start
            + source[line_start..spacing_start]
                .bytes()
                .take_while(|byte| matches!(byte, b' ' | b'\t'))
                .count();
        let left = &source[left_start..spacing_start];
        if left.is_empty()
            || !left.bytes().enumerate().all(|(index, byte)| {
                byte.is_ascii_alphanumeric()
                    || byte == b'_'
                    || byte == b'@'
                    || byte == b'.'
                    || byte == b'&'
                    || byte == b'$' && index == 0
            })
        {
            continue;
        }
        let right_start = equals
            + 1
            + source[equals + 1..]
                .bytes()
                .take_while(|byte| byte.is_ascii_whitespace())
                .count();
        let right = &source[right_start..];
        if !right.starts_with(&format!("{left}.")) && !right.starts_with(&format!("{left}&.")) {
            continue;
        }
        let separator = usize::from(right[left.len()..].starts_with("&.")) + 1;
        let method = right[left.len() + separator..]
            .split(['(', ' ', '{'])
            .next()
            .unwrap_or_default();
        if !mutating.contains(&method) {
            continue;
        }
        context.remove(
            format!("Redundant self assignment detected. Method `{method}` modifies its receiver in place."),
            equals..equals + 1,
            left_start..right_start,
        );
    }
}
