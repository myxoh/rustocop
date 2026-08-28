use super::CompletionTriggerKind;

#[test]
fn exposes_every_completion_trigger_kind() {
    assert_eq!(CompletionTriggerKind::INVOKED, 1);
    assert_eq!(CompletionTriggerKind::TRIGGER_CHARACTER, 2);
    assert_eq!(CompletionTriggerKind::TRIGGER_FOR_INCOMPLETE_COMPLETIONS, 3);
}

#[test]
fn exposes_integer_protocol_discriminants() {
    let kinds: [i64; 3] = [
        CompletionTriggerKind::INVOKED,
        CompletionTriggerKind::TRIGGER_CHARACTER,
        CompletionTriggerKind::TRIGGER_FOR_INCOMPLETE_COMPLETIONS,
    ];

    assert_eq!(kinds, [1, 2, 3]);
}
