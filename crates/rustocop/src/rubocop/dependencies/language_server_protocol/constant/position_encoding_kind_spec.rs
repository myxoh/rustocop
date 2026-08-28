use super::PositionEncodingKind;

#[test]
fn exposes_the_complete_position_encoding_kind_mapping() {
    assert_eq!(PositionEncodingKind::UTF8, "utf-8");
    assert_eq!(PositionEncodingKind::UTF16, "utf-16");
    assert_eq!(PositionEncodingKind::UTF32, "utf-32");
}

#[test]
fn values_can_drive_position_unit_selection() {
    let supported = [PositionEncodingKind::UTF8, PositionEncodingKind::UTF16];
    assert!(supported.contains(&"utf-8"));
    assert!(!supported.contains(&"utf-32"));
}
