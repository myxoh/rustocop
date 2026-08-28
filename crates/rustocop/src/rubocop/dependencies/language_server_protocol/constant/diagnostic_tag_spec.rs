use super::DiagnosticTag;

#[test]
fn exposes_every_diagnostic_tag() {
    assert_eq!(DiagnosticTag::UNNECESSARY, 1);
    assert_eq!(DiagnosticTag::DEPRECATED, 2);
}

#[test]
fn exposes_integer_protocol_discriminants() {
    let tags: [i64; 2] = [DiagnosticTag::UNNECESSARY, DiagnosticTag::DEPRECATED];

    assert_eq!(tags, [1, 2]);
}
