use super::CompletionItemTag;

#[test]
fn exposes_the_deprecated_completion_item_tag() {
    assert_eq!(CompletionItemTag::DEPRECATED, 1);
}

#[test]
fn exposes_an_integer_protocol_discriminant() {
    let deprecated: i64 = CompletionItemTag::DEPRECATED;

    assert_eq!(deprecated, 1);
}
