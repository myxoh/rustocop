use super::DiagnosticSeverity;

#[test]
fn exposes_every_diagnostic_severity() {
    assert_eq!(DiagnosticSeverity::ERROR, 1);
    assert_eq!(DiagnosticSeverity::WARNING, 2);
    assert_eq!(DiagnosticSeverity::INFORMATION, 3);
    assert_eq!(DiagnosticSeverity::HINT, 4);
}

#[test]
fn exposes_integer_protocol_discriminants() {
    let severities: [i64; 4] = [
        DiagnosticSeverity::ERROR,
        DiagnosticSeverity::WARNING,
        DiagnosticSeverity::INFORMATION,
        DiagnosticSeverity::HINT,
    ];

    assert_eq!(severities, [1, 2, 3, 4]);
}
