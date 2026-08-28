use super::InsertTextMode;

#[test]
fn exposes_every_insert_text_mode() {
    assert_eq!(InsertTextMode::AS_IS, 1);
    assert_eq!(InsertTextMode::ADJUST_INDENTATION, 2);
}

#[test]
fn exposes_integer_protocol_discriminants() {
    let modes: [i64; 2] = [InsertTextMode::AS_IS, InsertTextMode::ADJUST_INDENTATION];

    assert_eq!(modes, [1, 2]);
}
