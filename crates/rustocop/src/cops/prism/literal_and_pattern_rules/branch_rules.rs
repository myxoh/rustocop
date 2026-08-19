use super::*;

pub(super) fn branch_trailing_source<'source>(
    context: &'source CopContext<'_, '_>,
    start: usize,
) -> &'source str {
    let source = context.source();
    let tail = source.get(start..).unwrap_or_default();
    let mut length = 0;
    for line in tail.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if length > 0
            && (trimmed.starts_with("in ")
                || trimmed.starts_with("else")
                || trimmed.starts_with("end"))
        {
            break;
        }
        length += line.len();
        if !trimmed.is_empty() && !trimmed.starts_with('#') && length > line.len() {
            break;
        }
    }
    &tail[..length]
}

pub(super) fn file_null(
    node: &ruby_prism::StringNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    if context
        .parent()
        .is_some_and(|parent| parent.as_array_node().is_some() || parent.as_assoc_node().is_some())
    {
        return;
    }
    let value = String::from_utf8_lossy(node.unescaped());
    let lower = value.to_ascii_lowercase();
    let null = lower == "/dev/null" || lower == "nul:" || lower == "nul";
    if !null || lower == "nul" && !context.source().to_ascii_lowercase().contains("/dev/null") {
        return;
    }
    context.replace(
        format!("Use `File::NULL` instead of `{value}`."),
        node.location(),
        node.location(),
        "File::NULL",
    );
}
