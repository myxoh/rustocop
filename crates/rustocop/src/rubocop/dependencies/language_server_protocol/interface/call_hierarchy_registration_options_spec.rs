use serde_json::json;

use super::CallHierarchyRegistrationOptions;

#[test]
fn preserves_required_selector_and_present_optional_fields() {
    let selector = json!([{"language": "ruby"}]);
    let options =
        CallHierarchyRegistrationOptions::new(selector.clone(), Some(true), Some("call-hierarchy"));

    assert_eq!(options.document_selector(), &selector);
    assert!(options.work_done_progress());
    assert_eq!(options.id(), "call-hierarchy");
    assert_eq!(
        options.to_hash(),
        json!({
            "documentSelector": selector,
            "workDoneProgress": true,
            "id": "call-hierarchy"
        })
        .as_object()
        .unwrap()
    );
}

#[test]
fn retains_null_required_selector_and_omits_false_or_nil_options() {
    let options =
        CallHierarchyRegistrationOptions::new(serde_json::Value::Null, Some(false), None::<String>);

    assert_eq!(options.document_selector(), &serde_json::Value::Null);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&options.to_json()).unwrap(),
        json!({"documentSelector": null})
    );
    assert!(std::panic::catch_unwind(|| options.work_done_progress()).is_err());
    assert!(std::panic::catch_unwind(|| options.id()).is_err());
}
