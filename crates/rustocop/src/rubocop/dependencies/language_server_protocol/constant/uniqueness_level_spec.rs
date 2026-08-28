use super::UniquenessLevel;

#[test]
fn exposes_every_moniker_uniqueness_level() {
    assert_eq!(
        [
            UniquenessLevel::DOCUMENT,
            UniquenessLevel::PROJECT,
            UniquenessLevel::GROUP,
            UniquenessLevel::SCHEME,
            UniquenessLevel::GLOBAL,
        ],
        ["document", "project", "group", "scheme", "global"]
    );
}

#[test]
fn values_can_drive_scope_selection() {
    assert!(matches!(UniquenessLevel::GLOBAL, "global"));
}
