use serde_json::json;

use super::CodeActionParams;

#[test]
fn preserves_tokens_and_all_required_request_payloads() {
    let params = CodeActionParams::new(
        Some(json!(0)),
        Some(json!("partial")),
        json!({"uri": "file:///a.rb"}),
        json!({"start": {}, "end": {}}),
        json!({"diagnostics": []}),
    );

    assert_eq!(params.work_done_token(), &json!(0));
    assert_eq!(params.partial_result_token(), &json!("partial"));
    assert_eq!(params.text_document(), &json!({"uri": "file:///a.rb"}));
    assert_eq!(params.range(), &json!({"start": {}, "end": {}}));
    assert_eq!(params.context(), &json!({"diagnostics": []}));
    assert_eq!(params.attributes().len(), 5);
}

#[test]
fn omits_nil_and_false_tokens_but_never_required_fields() {
    let params = CodeActionParams::new(None, Some(json!(false)), json!({}), json!({}), json!({}));

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&params.to_json()).unwrap(),
        json!({"textDocument": {}, "range": {}, "context": {}})
    );
    assert!(std::panic::catch_unwind(|| params.work_done_token()).is_err());
    assert!(std::panic::catch_unwind(|| params.partial_result_token()).is_err());
    assert_eq!(params.to_hash(), params.attributes());
}
