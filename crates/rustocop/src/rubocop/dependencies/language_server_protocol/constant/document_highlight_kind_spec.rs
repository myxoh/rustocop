use super::DocumentHighlightKind;

#[test]
fn exposes_every_document_highlight_kind() {
    assert_eq!(DocumentHighlightKind::TEXT, 1);
    assert_eq!(DocumentHighlightKind::READ, 2);
    assert_eq!(DocumentHighlightKind::WRITE, 3);
}

#[test]
fn exposes_integer_protocol_discriminants() {
    let kinds: [i64; 3] = [
        DocumentHighlightKind::TEXT,
        DocumentHighlightKind::READ,
        DocumentHighlightKind::WRITE,
    ];

    assert_eq!(kinds, [1, 2, 3]);
}
