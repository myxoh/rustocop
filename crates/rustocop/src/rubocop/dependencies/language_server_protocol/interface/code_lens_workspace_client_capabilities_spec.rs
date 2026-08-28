use serde_json::json;

use super::CodeLensWorkspaceClientCapabilities;

#[test]
fn preserves_truthy_refresh_support() {
    let capabilities = CodeLensWorkspaceClientCapabilities::new(Some(true));

    assert!(capabilities.refresh_support());
    assert_eq!(
        capabilities.to_hash(),
        json!({"refreshSupport": true}).as_object().unwrap()
    );
}

#[test]
fn omits_nil_and_false_refresh_support() {
    for capabilities in [
        CodeLensWorkspaceClientCapabilities::new(None),
        CodeLensWorkspaceClientCapabilities::new(Some(false)),
    ] {
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&capabilities.to_json()).unwrap(),
            json!({})
        );
        assert!(std::panic::catch_unwind(|| capabilities.refresh_support()).is_err());
        assert_eq!(capabilities.to_hash(), capabilities.attributes());
    }
}
