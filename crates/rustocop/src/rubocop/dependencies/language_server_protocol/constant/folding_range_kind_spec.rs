use super::FoldingRangeKind;

#[test]
fn exposes_every_predefined_folding_range_kind() {
    assert_eq!(FoldingRangeKind::COMMENT, "comment");
    assert_eq!(FoldingRangeKind::IMPORTS, "imports");
    assert_eq!(FoldingRangeKind::REGION, "region");
}

#[test]
fn supports_open_set_protocol_values() {
    let predefined = [
        FoldingRangeKind::COMMENT,
        FoldingRangeKind::IMPORTS,
        FoldingRangeKind::REGION,
    ];
    let custom = "custom.section";

    assert!(!predefined.contains(&custom));
    assert_eq!(custom.to_owned(), "custom.section");
}
