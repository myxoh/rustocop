use super::MonikerKind;

#[test]
fn exposes_every_moniker_kind() {
    assert_eq!(MonikerKind::IMPORT, "import");
    assert_eq!(MonikerKind::EXPORT, "export");
    assert_eq!(MonikerKind::LOCAL, "local");
}

#[test]
fn supports_protocol_kind_matching() {
    fn project_visibility(kind: &str) -> &'static str {
        match kind {
            MonikerKind::IMPORT | MonikerKind::EXPORT => "external",
            MonikerKind::LOCAL => "local",
            _ => "unknown",
        }
    }

    assert_eq!(project_visibility(MonikerKind::IMPORT), "external");
    assert_eq!(project_visibility(MonikerKind::LOCAL), "local");
}
