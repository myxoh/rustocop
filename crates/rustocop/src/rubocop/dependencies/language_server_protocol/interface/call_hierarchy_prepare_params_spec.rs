use serde_json::json;

use super::CallHierarchyPrepareParams;

#[test]
fn preserves_required_document_position_and_present_token() {
    let document = json!({"uri": "file:///a.rb"});
    let position = json!({"line": 3, "character": 2});
    let params =
        CallHierarchyPrepareParams::new(document.clone(), position.clone(), Some(json!(0)));

    assert_eq!(params.text_document(), &document);
    assert_eq!(params.position(), &position);
    assert_eq!(params.work_done_token(), &json!(0));
    assert_eq!(
        params.to_hash(),
        json!({"textDocument": document, "position": position, "workDoneToken": 0})
            .as_object()
            .unwrap()
    );
}

#[test]
fn omits_nil_or_false_token_without_omitting_required_fields() {
    let params = CallHierarchyPrepareParams::new(json!({}), json!({}), Some(json!(false)));

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&params.to_json()).unwrap(),
        json!({"textDocument": {}, "position": {}})
    );
    assert!(std::panic::catch_unwind(|| params.work_done_token()).is_err());
}
