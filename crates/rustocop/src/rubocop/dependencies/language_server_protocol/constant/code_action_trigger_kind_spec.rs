use super::CodeActionTriggerKind;

#[test]
fn exposes_every_code_action_trigger_kind() {
    assert_eq!(CodeActionTriggerKind::INVOKED, 1);
    assert_eq!(CodeActionTriggerKind::AUTOMATIC, 2);
}

#[test]
fn exposes_integer_protocol_discriminants() {
    let invoked: i64 = CodeActionTriggerKind::INVOKED;
    let automatic: i64 = CodeActionTriggerKind::AUTOMATIC;

    assert_eq!([invoked, automatic], [1, 2]);
}
