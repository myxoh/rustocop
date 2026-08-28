use super::InitializeErrorCodes;

#[test]
fn exposes_the_unknown_protocol_version_code() {
    assert_eq!(InitializeErrorCodes::UNKNOWN_PROTOCOL_VERSION, 1);
}

#[test]
fn exposes_an_integer_protocol_discriminant() {
    let code: i64 = InitializeErrorCodes::UNKNOWN_PROTOCOL_VERSION;

    assert_eq!(code, 1);
}
