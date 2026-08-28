use super::InlayHintKind;

#[test]
fn exposes_every_inlay_hint_kind() {
    assert_eq!(InlayHintKind::TYPE, 1);
    assert_eq!(InlayHintKind::PARAMETER, 2);
}

#[test]
fn exposes_integer_protocol_discriminants() {
    let kinds: [i64; 2] = [InlayHintKind::TYPE, InlayHintKind::PARAMETER];

    assert_eq!(kinds, [1, 2]);
}
