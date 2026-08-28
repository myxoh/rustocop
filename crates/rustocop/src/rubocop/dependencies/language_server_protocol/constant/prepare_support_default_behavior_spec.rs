use super::PrepareSupportDefaultBehavior;

#[test]
fn exposes_the_identifier_default_behavior() {
    assert_eq!(PrepareSupportDefaultBehavior::IDENTIFIER, 1);
}

#[test]
fn uses_the_protocol_integer_representation() {
    let behavior: i64 = PrepareSupportDefaultBehavior::IDENTIFIER;
    assert_eq!(behavior, 1_i64);
}
