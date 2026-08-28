use super::CodeActionKind;

#[test]
fn exposes_every_predefined_code_action_kind() {
    assert_eq!(CodeActionKind::EMPTY, "");
    assert_eq!(CodeActionKind::QUICK_FIX, "quickfix");
    assert_eq!(CodeActionKind::REFACTOR, "refactor");
    assert_eq!(CodeActionKind::REFACTOR_EXTRACT, "refactor.extract");
    assert_eq!(CodeActionKind::REFACTOR_INLINE, "refactor.inline");
    assert_eq!(CodeActionKind::REFACTOR_REWRITE, "refactor.rewrite");
    assert_eq!(CodeActionKind::SOURCE, "source");
    assert_eq!(
        CodeActionKind::SOURCE_ORGANIZE_IMPORTS,
        "source.organizeImports"
    );
    assert_eq!(CodeActionKind::SOURCE_FIX_ALL, "source.fixAll");
}

#[test]
fn preserves_the_hierarchical_protocol_identifiers() {
    assert!(CodeActionKind::REFACTOR_EXTRACT.starts_with(CodeActionKind::REFACTOR));
    assert!(CodeActionKind::REFACTOR_INLINE.starts_with(CodeActionKind::REFACTOR));
    assert!(CodeActionKind::REFACTOR_REWRITE.starts_with(CodeActionKind::REFACTOR));
    assert!(CodeActionKind::SOURCE_ORGANIZE_IMPORTS.starts_with(CodeActionKind::SOURCE));
    assert!(CodeActionKind::SOURCE_FIX_ALL.starts_with(CodeActionKind::SOURCE));
}
