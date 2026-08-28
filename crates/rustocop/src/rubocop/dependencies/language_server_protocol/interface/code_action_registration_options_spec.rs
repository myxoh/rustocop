use serde_json::json;

use super::CodeActionRegistrationOptions;

#[test]
fn preserves_required_selector_and_all_truthy_options() {
    let options = CodeActionRegistrationOptions::new(
        json!([{"language": "ruby"}]),
        Some(true),
        Some(vec!["quickfix".to_string()]),
        Some(true),
    );

    assert_eq!(options.document_selector(), &json!([{"language": "ruby"}]));
    assert!(options.work_done_progress());
    assert_eq!(options.code_action_kinds(), &["quickfix"]);
    assert!(options.resolve_provider());
    assert_eq!(options.attributes().len(), 4);
}

#[test]
fn retains_null_selector_and_empty_kind_list_while_omitting_false_booleans() {
    let options = CodeActionRegistrationOptions::new(
        serde_json::Value::Null,
        Some(false),
        Some(Vec::new()),
        Some(false),
    );

    assert_eq!(options.document_selector(), &serde_json::Value::Null);
    assert!(options.code_action_kinds().is_empty());
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&options.to_json()).unwrap(),
        json!({"documentSelector": null, "codeActionKinds": []})
    );
    assert!(std::panic::catch_unwind(|| options.work_done_progress()).is_err());
    assert!(std::panic::catch_unwind(|| options.resolve_provider()).is_err());
    assert_eq!(options.to_hash(), options.attributes());
}
