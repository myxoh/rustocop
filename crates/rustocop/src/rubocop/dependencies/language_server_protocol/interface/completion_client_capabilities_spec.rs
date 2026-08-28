use serde_json::json;

use super::CompletionClientCapabilities;

#[test]
fn preserves_all_present_completion_capabilities() {
    let capabilities = CompletionClientCapabilities::new(
        Some(true),
        Some(json!({"snippetSupport": true})),
        Some(json!({"valueSet": [1, 2]})),
        Some(true),
        Some(2),
        Some(json!({"itemDefaults": ["editRange"]})),
    );

    assert!(capabilities.dynamic_registration());
    assert_eq!(
        capabilities.completion_item(),
        &json!({"snippetSupport": true})
    );
    assert_eq!(
        capabilities.completion_item_kind(),
        &json!({"valueSet": [1, 2]})
    );
    assert!(capabilities.context_support());
    assert_eq!(capabilities.insert_text_mode(), 2);
    assert_eq!(
        capabilities.completion_list(),
        &json!({"itemDefaults": ["editRange"]})
    );
    assert_eq!(capabilities.attributes().len(), 6);
}

#[test]
fn omits_false_and_nil_fields_but_retains_empty_objects_and_zero_mode() {
    let capabilities = CompletionClientCapabilities::new(
        Some(false),
        Some(json!({})),
        None,
        Some(false),
        Some(0),
        Some(json!({})),
    );

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&capabilities.to_json()).unwrap(),
        json!({"completionItem": {}, "insertTextMode": 0, "completionList": {}})
    );
    assert!(std::panic::catch_unwind(|| capabilities.dynamic_registration()).is_err());
    assert!(std::panic::catch_unwind(|| capabilities.context_support()).is_err());
    assert!(std::panic::catch_unwind(|| capabilities.completion_item_kind()).is_err());
    assert_eq!(capabilities.to_hash(), capabilities.attributes());
}
