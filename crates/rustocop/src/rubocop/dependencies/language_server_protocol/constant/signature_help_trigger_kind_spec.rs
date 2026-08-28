use super::SignatureHelpTriggerKind;

#[test]
fn exposes_every_signature_help_trigger_kind() {
    assert_eq!(SignatureHelpTriggerKind::INVOKED, 1);
    assert_eq!(SignatureHelpTriggerKind::TRIGGER_CHARACTER, 2);
    assert_eq!(SignatureHelpTriggerKind::CONTENT_CHANGE, 3);
}

#[test]
fn uses_one_protocol_integer_type() {
    let values: [i64; 3] = [
        SignatureHelpTriggerKind::INVOKED,
        SignatureHelpTriggerKind::TRIGGER_CHARACTER,
        SignatureHelpTriggerKind::CONTENT_CHANGE,
    ];
    assert_eq!(values, [1, 2, 3]);
}
