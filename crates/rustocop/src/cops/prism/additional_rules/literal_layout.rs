use super::*;

pub(super) fn empty_heredoc(
    node: &ruby_prism::StringNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let (Some(opening), Some(closing)) = (node.opening_loc(), node.closing_loc()) else {
        return;
    };
    if !opening.as_slice().starts_with(b"<<") || !node.unescaped().is_empty() {
        return;
    }
    let source_file = context.source_file();
    let header_end = source_file.line_end(opening.start_offset());
    let full_end = (closing.end_offset()
        + usize::from(context.source().as_bytes().get(closing.end_offset()) == Some(&b'\n')))
    .min(context.source().len());
    let quotes = if context.related_config_value("Style/StringLiterals", "EnforcedStyle")
        == Some("double_quotes")
    {
        "\"\""
    } else {
        "''"
    };
    let replacement = format!(
        "{quotes}{}\n",
        &context.source()[opening.end_offset()..header_end]
    );
    context.replace(
        "Use an empty string literal instead of heredoc.",
        opening.start_offset()..opening.end_offset(),
        opening.start_offset()..full_end,
        replacement,
    );
}
