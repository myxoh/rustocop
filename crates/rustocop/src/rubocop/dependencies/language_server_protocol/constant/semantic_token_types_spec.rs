use super::SemanticTokenTypes;

#[test]
fn exposes_every_semantic_token_type_in_protocol_order() {
    assert_eq!(
        [
            SemanticTokenTypes::NAMESPACE,
            SemanticTokenTypes::TYPE,
            SemanticTokenTypes::CLASS,
            SemanticTokenTypes::ENUM,
            SemanticTokenTypes::INTERFACE,
            SemanticTokenTypes::STRUCT,
            SemanticTokenTypes::TYPE_PARAMETER,
            SemanticTokenTypes::PARAMETER,
            SemanticTokenTypes::VARIABLE,
            SemanticTokenTypes::PROPERTY,
            SemanticTokenTypes::ENUM_MEMBER,
            SemanticTokenTypes::EVENT,
            SemanticTokenTypes::FUNCTION,
            SemanticTokenTypes::METHOD,
            SemanticTokenTypes::MACRO,
            SemanticTokenTypes::KEYWORD,
            SemanticTokenTypes::MODIFIER,
            SemanticTokenTypes::COMMENT,
            SemanticTokenTypes::STRING,
            SemanticTokenTypes::NUMBER,
            SemanticTokenTypes::REGEXP,
            SemanticTokenTypes::OPERATOR,
            SemanticTokenTypes::DECORATOR,
        ],
        [
            "namespace",
            "type",
            "class",
            "enum",
            "interface",
            "struct",
            "typeParameter",
            "parameter",
            "variable",
            "property",
            "enumMember",
            "event",
            "function",
            "method",
            "macro",
            "keyword",
            "modifier",
            "comment",
            "string",
            "number",
            "regexp",
            "operator",
            "decorator",
        ]
    );
}

#[test]
fn preserves_camel_cased_protocol_values() {
    assert_eq!(SemanticTokenTypes::TYPE_PARAMETER, "typeParameter");
    assert_eq!(SemanticTokenTypes::ENUM_MEMBER, "enumMember");
}
