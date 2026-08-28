use serde_json::json;

use super::CodeLensParams;

#[test]
fn preserves_both_tokens_and_required_document() {
    let params = CodeLensParams::new(
        Some(json!(0)),
        Some(json!("partial")),
        json!({"uri": "file:///a.rb"}),
    );

    assert_eq!(params.work_done_token(), &json!(0));
    assert_eq!(params.partial_result_token(), &json!("partial"));
    assert_eq!(params.text_document(), &json!({"uri": "file:///a.rb"}));
    assert_eq!(params.attributes().len(), 3);
}

#[test]
fn omits_nil_and_false_tokens_but_retains_required_document() {
    let params = CodeLensParams::new(None, Some(json!(false)), json!({}));

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&params.to_json()).unwrap(),
        json!({"textDocument": {}})
    );
    assert!(std::panic::catch_unwind(|| params.work_done_token()).is_err());
    assert!(std::panic::catch_unwind(|| params.partial_result_token()).is_err());
    assert_eq!(params.to_hash(), params.attributes());
}
