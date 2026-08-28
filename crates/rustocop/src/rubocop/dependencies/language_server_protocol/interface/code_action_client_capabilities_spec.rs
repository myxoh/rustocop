use serde_json::json;

use super::CodeActionClientCapabilities;

#[test]
fn preserves_all_present_capability_fields() {
    let capabilities = CodeActionClientCapabilities::new(
        Some(true),
        Some(json!({"codeActionKind": {"valueSet": ["quickfix"]}})),
        Some(true),
        Some(true),
        Some(true),
        Some(json!({"properties": ["edit"]})),
        Some(true),
    );

    assert!(capabilities.dynamic_registration());
    assert_eq!(
        capabilities.code_action_literal_support(),
        &json!({"codeActionKind": {"valueSet": ["quickfix"]}})
    );
    assert!(capabilities.is_preferred_support());
    assert!(capabilities.disabled_support());
    assert!(capabilities.data_support());
    assert_eq!(
        capabilities.resolve_support(),
        &json!({"properties": ["edit"]})
    );
    assert!(capabilities.honors_change_annotations());
    assert_eq!(capabilities.attributes().len(), 7);
}

#[test]
fn omits_nil_and_false_fields_but_retains_empty_objects() {
    let capabilities = CodeActionClientCapabilities::new(
        Some(false),
        Some(json!({})),
        None,
        Some(false),
        None,
        Some(json!({})),
        Some(false),
    );

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&capabilities.to_json()).unwrap(),
        json!({"codeActionLiteralSupport": {}, "resolveSupport": {}})
    );
    assert!(std::panic::catch_unwind(|| capabilities.dynamic_registration()).is_err());
    assert!(std::panic::catch_unwind(|| capabilities.disabled_support()).is_err());
    assert!(std::panic::catch_unwind(|| capabilities.honors_change_annotations()).is_err());
    assert_eq!(capabilities.to_hash(), capabilities.attributes());
}
