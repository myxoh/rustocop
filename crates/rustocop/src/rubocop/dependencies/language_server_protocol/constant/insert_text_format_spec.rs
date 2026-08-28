use super::InsertTextFormat;

#[test]
fn exposes_every_insert_text_format() {
    assert_eq!(InsertTextFormat::PLAIN_TEXT, 1);
    assert_eq!(InsertTextFormat::SNIPPET, 2);
}

#[test]
fn exposes_integer_protocol_discriminants() {
    let formats: [i64; 2] = [InsertTextFormat::PLAIN_TEXT, InsertTextFormat::SNIPPET];

    assert_eq!(formats, [1, 2]);
}
