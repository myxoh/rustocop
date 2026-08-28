use serde_json::json;

use super::CodeLensClientCapabilities;

#[test]
fn preserves_truthy_dynamic_registration() {
    let capabilities = CodeLensClientCapabilities::new(Some(true));

    assert!(capabilities.dynamic_registration());
    assert_eq!(
        capabilities.to_hash(),
        json!({"dynamicRegistration": true}).as_object().unwrap()
    );
    assert_eq!(capabilities.attributes(), capabilities.to_hash());
}

#[test]
fn omits_nil_and_false_dynamic_registration() {
    for capabilities in [
        CodeLensClientCapabilities::new(None),
        CodeLensClientCapabilities::new(Some(false)),
    ] {
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&capabilities.to_json()).unwrap(),
            json!({})
        );
        assert!(std::panic::catch_unwind(|| capabilities.dynamic_registration()).is_err());
    }
}
