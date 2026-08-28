use super::MarkupKind;

#[test]
fn exposes_every_markup_kind() {
    assert_eq!(MarkupKind::PLAIN_TEXT, "plaintext");
    assert_eq!(MarkupKind::MARKDOWN, "markdown");
}

#[test]
fn supports_content_format_preference_ordering() {
    let preferred = [MarkupKind::MARKDOWN, MarkupKind::PLAIN_TEXT];

    assert_eq!(preferred.first(), Some(&"markdown"));
    assert!(preferred.contains(&"plaintext"));
}
