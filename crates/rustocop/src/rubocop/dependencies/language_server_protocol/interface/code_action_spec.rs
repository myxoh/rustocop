use serde_json::json;

use super::CodeAction;

#[test]
fn preserves_required_title_and_all_present_optional_payloads() {
    let action = CodeAction::new(
        "Fix offense",
        Some("quickfix"),
        Some(Vec::new()),
        Some(true),
        Some(json!({"reason": "unsafe"})),
        Some(json!({"changes": {}})),
        Some(json!({"title": "Run", "command": "run"})),
        Some(json!({"id": 1})),
    );

    assert_eq!(action.title(), "Fix offense");
    assert_eq!(action.kind(), "quickfix");
    assert!(action.diagnostics().is_empty());
    assert!(action.is_preferred());
    assert_eq!(action.disabled(), &json!({"reason": "unsafe"}));
    assert_eq!(action.edit(), &json!({"changes": {}}));
    assert_eq!(action.command(), &json!({"title": "Run", "command": "run"}));
    assert_eq!(action.data(), &json!({"id": 1}));
    assert_eq!(action.attributes().len(), 8);
}

#[test]
fn omits_nil_and_false_options_but_retains_empty_collections() {
    let action = CodeAction::new(
        "Fix",
        None::<String>,
        Some(Vec::new()),
        Some(false),
        None,
        Some(json!({})),
        None,
        Some(json!(false)),
    );

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&action.to_json()).unwrap(),
        json!({"title": "Fix", "diagnostics": [], "edit": {}})
    );
    assert!(std::panic::catch_unwind(|| action.kind()).is_err());
    assert!(std::panic::catch_unwind(|| action.is_preferred()).is_err());
    assert!(std::panic::catch_unwind(|| action.data()).is_err());
    assert_eq!(action.to_hash(), action.attributes());
}
