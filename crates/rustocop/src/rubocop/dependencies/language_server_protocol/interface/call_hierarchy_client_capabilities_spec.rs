use serde_json::json;

use super::CallHierarchyClientCapabilities;

#[test]
fn includes_a_truthy_dynamic_registration_capability() {
    let capabilities = CallHierarchyClientCapabilities::new(Some(true));

    assert!(capabilities.dynamic_registration());
    assert_eq!(
        capabilities.to_hash(),
        json!({"dynamicRegistration": true}).as_object().unwrap()
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&capabilities.to_json()).unwrap(),
        json!({"dynamicRegistration": true})
    );
}

#[test]
fn omits_nil_and_false_like_the_ruby_truthiness_guard() {
    for capabilities in [
        CallHierarchyClientCapabilities::new(None),
        CallHierarchyClientCapabilities::new(Some(false)),
    ] {
        assert!(capabilities.attributes().is_empty());
        assert!(std::panic::catch_unwind(|| capabilities.dynamic_registration()).is_err());
    }
}
