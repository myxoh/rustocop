use super::FailureHandlingKind;

#[test]
fn exposes_every_failure_handling_kind() {
    assert_eq!(FailureHandlingKind::ABORT, "abort");
    assert_eq!(FailureHandlingKind::TRANSACTIONAL, "transactional");
    assert_eq!(
        FailureHandlingKind::TEXT_ONLY_TRANSACTIONAL,
        "textOnlyTransactional"
    );
    assert_eq!(FailureHandlingKind::UNDO, "undo");
}

#[test]
fn supports_protocol_value_selection() {
    let supported = [
        FailureHandlingKind::ABORT,
        FailureHandlingKind::TRANSACTIONAL,
        FailureHandlingKind::TEXT_ONLY_TRANSACTIONAL,
        FailureHandlingKind::UNDO,
    ];

    assert_eq!(supported[2], "textOnlyTransactional");
}
