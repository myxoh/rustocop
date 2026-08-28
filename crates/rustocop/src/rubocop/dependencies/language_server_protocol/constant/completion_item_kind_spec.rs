use super::CompletionItemKind;

#[test]
fn exposes_every_completion_item_kind_in_protocol_order() {
    let kinds: [i64; 25] = [
        CompletionItemKind::TEXT,
        CompletionItemKind::METHOD,
        CompletionItemKind::FUNCTION,
        CompletionItemKind::CONSTRUCTOR,
        CompletionItemKind::FIELD,
        CompletionItemKind::VARIABLE,
        CompletionItemKind::CLASS,
        CompletionItemKind::INTERFACE,
        CompletionItemKind::MODULE,
        CompletionItemKind::PROPERTY,
        CompletionItemKind::UNIT,
        CompletionItemKind::VALUE,
        CompletionItemKind::ENUM,
        CompletionItemKind::KEYWORD,
        CompletionItemKind::SNIPPET,
        CompletionItemKind::COLOR,
        CompletionItemKind::FILE,
        CompletionItemKind::REFERENCE,
        CompletionItemKind::FOLDER,
        CompletionItemKind::ENUM_MEMBER,
        CompletionItemKind::CONSTANT,
        CompletionItemKind::STRUCT,
        CompletionItemKind::EVENT,
        CompletionItemKind::OPERATOR,
        CompletionItemKind::TYPE_PARAMETER,
    ];

    assert_eq!(kinds, std::array::from_fn(|index| (index + 1) as i64));
}

#[test]
fn exposes_integer_protocol_discriminants() {
    let text: i64 = CompletionItemKind::TEXT;
    let type_parameter: i64 = CompletionItemKind::TYPE_PARAMETER;

    assert_eq!((text, type_parameter), (1, 25));
}
