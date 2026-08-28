use super::FileOperationPatternKind;

#[test]
fn exposes_every_file_operation_pattern_kind() {
    assert_eq!(FileOperationPatternKind::FILE, "file");
    assert_eq!(FileOperationPatternKind::FOLDER, "folder");
}

#[test]
fn supports_protocol_pattern_kind_selection() {
    let kinds = [
        FileOperationPatternKind::FILE,
        FileOperationPatternKind::FOLDER,
    ];

    assert_eq!(
        kinds.iter().find(|kind| **kind == "folder"),
        Some(&"folder")
    );
}
