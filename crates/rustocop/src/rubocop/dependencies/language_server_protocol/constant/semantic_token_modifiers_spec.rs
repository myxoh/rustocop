use super::SemanticTokenModifiers;

#[test]
fn exposes_every_semantic_token_modifier() {
    assert_eq!(
        [
            SemanticTokenModifiers::DECLARATION,
            SemanticTokenModifiers::DEFINITION,
            SemanticTokenModifiers::READONLY,
            SemanticTokenModifiers::STATIC,
            SemanticTokenModifiers::DEPRECATED,
            SemanticTokenModifiers::ABSTRACT,
            SemanticTokenModifiers::ASYNC,
            SemanticTokenModifiers::MODIFICATION,
            SemanticTokenModifiers::DOCUMENTATION,
            SemanticTokenModifiers::DEFAULT_LIBRARY,
        ],
        [
            "declaration",
            "definition",
            "readonly",
            "static",
            "deprecated",
            "abstract",
            "async",
            "modification",
            "documentation",
            "defaultLibrary",
        ]
    );
}

#[test]
fn preserves_the_default_library_casing() {
    assert_eq!(SemanticTokenModifiers::DEFAULT_LIBRARY, "defaultLibrary");
}
