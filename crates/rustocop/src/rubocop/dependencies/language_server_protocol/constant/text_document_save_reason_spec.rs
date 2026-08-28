use super::TextDocumentSaveReason;

#[test]
fn exposes_every_text_document_save_reason() {
    assert_eq!(TextDocumentSaveReason::MANUAL, 1);
    assert_eq!(TextDocumentSaveReason::AFTER_DELAY, 2);
    assert_eq!(TextDocumentSaveReason::FOCUS_OUT, 3);
}

#[test]
fn uses_one_protocol_integer_type() {
    let reasons: [i64; 3] = [
        TextDocumentSaveReason::MANUAL,
        TextDocumentSaveReason::AFTER_DELAY,
        TextDocumentSaveReason::FOCUS_OUT,
    ];
    assert_eq!(reasons, [1, 2, 3]);
}
