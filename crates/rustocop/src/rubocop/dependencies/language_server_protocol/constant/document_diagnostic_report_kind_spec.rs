use super::DocumentDiagnosticReportKind;

#[test]
fn exposes_every_document_diagnostic_report_kind() {
    assert_eq!(DocumentDiagnosticReportKind::FULL, "full");
    assert_eq!(DocumentDiagnosticReportKind::UNCHANGED, "unchanged");
}

#[test]
fn supports_protocol_comparison_and_collection_values() {
    let kinds = [
        DocumentDiagnosticReportKind::FULL,
        DocumentDiagnosticReportKind::UNCHANGED,
    ];

    assert_eq!(kinds.join(","), "full,unchanged");
}
