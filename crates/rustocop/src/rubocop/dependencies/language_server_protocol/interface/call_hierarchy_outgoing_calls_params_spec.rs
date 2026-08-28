use serde_json::json;

use super::CallHierarchyOutgoingCallsParams;

#[test]
fn preserves_tokens_and_required_item() {
    let item = json!({"name": "caller"});
    let params =
        CallHierarchyOutgoingCallsParams::new(Some(json!(0)), Some(json!("partial")), item.clone());

    assert_eq!(params.work_done_token(), &json!(0));
    assert_eq!(params.partial_result_token(), &json!("partial"));
    assert_eq!(params.item(), &item);
    assert_eq!(
        params.to_hash(),
        json!({"workDoneToken": 0, "partialResultToken": "partial", "item": item})
            .as_object()
            .unwrap()
    );
}

#[test]
fn omits_nil_and_false_tokens_and_serializes_the_item() {
    let params =
        CallHierarchyOutgoingCallsParams::new(Some(json!(false)), None, json!({"name": "x"}));

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&params.to_json()).unwrap(),
        json!({"item": {"name": "x"}})
    );
    assert!(std::panic::catch_unwind(|| params.work_done_token()).is_err());
    assert!(std::panic::catch_unwind(|| params.partial_result_token()).is_err());
}
