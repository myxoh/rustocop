use super::TokenFormat;

#[test]
fn exposes_the_relative_token_format() {
    assert_eq!(TokenFormat::RELATIVE, "relative");
}

#[test]
fn value_can_be_selected_as_a_protocol_format() {
    let formats = [TokenFormat::RELATIVE];
    assert!(formats.contains(&"relative"));
}
