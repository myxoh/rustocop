use serde_json::json;

use super::AnnotatedTextEdit;

#[test]
fn exposes_the_complete_frozen_attribute_shape() {
    let range = json!({"start":{"line":1,"character":2},"end":{"line":1,"character":4}});
    let edit = AnnotatedTextEdit::new(range.clone(), "new", "change-1");

    assert_eq!(edit.range(), &range);
    assert_eq!(edit.new_text(), "new");
    assert_eq!(edit.annotation_id(), "change-1");
    assert_eq!(
        edit.attributes(),
        &json!({"range":range,"newText":"new","annotationId":"change-1"})
            .as_object()
            .unwrap()
            .clone()
    );
    assert_eq!(edit.to_hash(), edit.attributes());
}

#[test]
fn serializes_with_the_protocol_key_names() {
    let edit = AnnotatedTextEdit::new(json!({"start":{},"end":{}}), "", "id");
    let json: serde_json::Value = serde_json::from_str(&edit.to_json()).unwrap();
    assert_eq!(json["newText"], "");
    assert_eq!(json["annotationId"], "id");
}
