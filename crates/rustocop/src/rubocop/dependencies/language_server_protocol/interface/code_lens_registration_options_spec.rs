use serde_json::json;

use super::CodeLensRegistrationOptions;

#[test]
fn preserves_required_selector_and_truthy_options() {
    let options =
        CodeLensRegistrationOptions::new(json!([{"language": "ruby"}]), Some(true), Some(true));

    assert_eq!(options.document_selector(), &json!([{"language": "ruby"}]));
    assert!(options.work_done_progress());
    assert!(options.resolve_provider());
    assert_eq!(options.attributes().len(), 3);
}

#[test]
fn retains_null_selector_and_omits_false_or_nil_options() {
    let options = CodeLensRegistrationOptions::new(serde_json::Value::Null, Some(false), None);

    assert_eq!(options.document_selector(), &serde_json::Value::Null);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&options.to_json()).unwrap(),
        json!({"documentSelector": null})
    );
    assert!(std::panic::catch_unwind(|| options.work_done_progress()).is_err());
    assert!(std::panic::catch_unwind(|| options.resolve_provider()).is_err());
    assert_eq!(options.to_hash(), options.attributes());
}
