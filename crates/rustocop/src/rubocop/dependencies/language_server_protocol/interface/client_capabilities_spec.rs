use serde_json::json;

use super::ClientCapabilities;

#[test]
fn preserves_all_present_capability_payloads_and_protocol_keys() {
    let capabilities = ClientCapabilities::new(
        Some(json!({"applyEdit": true})),
        Some(json!({"hover": {}})),
        Some(json!({"synchronization": {}})),
        Some(json!({"workDoneProgress": true})),
        Some(json!({"positionEncodings": ["utf-8"]})),
        Some(json!({"custom": 1})),
    );

    assert_eq!(capabilities.workspace(), &json!({"applyEdit": true}));
    assert_eq!(capabilities.text_document(), &json!({"hover": {}}));
    assert_eq!(
        capabilities.notebook_document(),
        &json!({"synchronization": {}})
    );
    assert_eq!(capabilities.window(), &json!({"workDoneProgress": true}));
    assert_eq!(
        capabilities.general(),
        &json!({"positionEncodings": ["utf-8"]})
    );
    assert_eq!(capabilities.experimental(), &json!({"custom": 1}));
    assert_eq!(capabilities.attributes().len(), 6);
}

#[test]
fn omits_nil_and_false_values_and_retains_empty_payloads() {
    let capabilities = ClientCapabilities::new(
        None,
        Some(json!(false)),
        Some(json!({})),
        Some(json!([])),
        Some(json!(0)),
        Some(json!("")),
    );

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&capabilities.to_json()).unwrap(),
        json!({"notebookDocument": {}, "window": [], "general": 0, "experimental": ""})
    );
    assert!(std::panic::catch_unwind(|| capabilities.workspace()).is_err());
    assert!(std::panic::catch_unwind(|| capabilities.text_document()).is_err());
    assert_eq!(capabilities.to_hash(), capabilities.attributes());
}
