use super::TextDocumentSyncKind;

#[test]
fn exposes_every_text_document_sync_kind() {
    assert_eq!(TextDocumentSyncKind::NONE, 0);
    assert_eq!(TextDocumentSyncKind::FULL, 1);
    assert_eq!(TextDocumentSyncKind::INCREMENTAL, 2);
}

#[test]
fn preserves_the_zero_based_protocol_values() {
    let kinds: [i64; 3] = [
        TextDocumentSyncKind::NONE,
        TextDocumentSyncKind::FULL,
        TextDocumentSyncKind::INCREMENTAL,
    ];
    assert_eq!(kinds, [0, 1, 2]);
}
