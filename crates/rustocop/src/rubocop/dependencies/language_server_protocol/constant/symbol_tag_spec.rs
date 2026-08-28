use super::SymbolTag;

#[test]
fn exposes_the_deprecated_symbol_tag() {
    assert_eq!(SymbolTag::DEPRECATED, 1);
}

#[test]
fn uses_the_protocol_integer_representation() {
    let tag: i64 = SymbolTag::DEPRECATED;
    assert_eq!(tag, 1);
}
