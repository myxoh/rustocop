use serde_json::json;

use super::CallHierarchyIncomingCallsParams;

#[test]
fn preserves_tokens_and_required_item() {
    let item = json!({"name": "callee"});
    let params =
        CallHierarchyIncomingCallsParams::new(Some(json!("work")), Some(json!(0)), item.clone());

    assert_eq!(params.work_done_token(), &json!("work"));
    assert_eq!(params.partial_result_token(), &json!(0));
    assert_eq!(params.item(), &item);
    assert_eq!(
        params.to_hash(),
        json!({"workDoneToken": "work", "partialResultToken": 0, "item": item})
            .as_object()
            .unwrap()
    );
}

#[test]
fn omits_nil_and_false_optional_tokens_but_keeps_the_item() {
    let params =
        CallHierarchyIncomingCallsParams::new(None, Some(json!(false)), json!({"name": "x"}));

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&params.to_json()).unwrap(),
        json!({"item": {"name": "x"}})
    );
    assert!(std::panic::catch_unwind(|| params.work_done_token()).is_err());
    assert!(std::panic::catch_unwind(|| params.partial_result_token()).is_err());
}
