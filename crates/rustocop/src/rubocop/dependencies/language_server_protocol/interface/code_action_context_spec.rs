use serde_json::json;

use super::CodeActionContext;

#[test]
fn preserves_required_diagnostics_and_present_filters() {
    let diagnostics = vec![json!({"message": "offense"})];
    let context = CodeActionContext::new(
        diagnostics.clone(),
        Some(vec!["quickfix".to_string()]),
        Some(2),
    );

    assert_eq!(context.diagnostics(), diagnostics.as_slice());
    assert_eq!(context.only(), &["quickfix"]);
    assert_eq!(context.trigger_kind(), 2);
    assert_eq!(
        context.to_hash(),
        json!({"diagnostics": diagnostics, "only": ["quickfix"], "triggerKind": 2})
            .as_object()
            .unwrap()
    );
}

#[test]
fn retains_empty_required_diagnostics_and_empty_only_list() {
    let context = CodeActionContext::new(Vec::new(), Some(Vec::<String>::new()), None);

    assert!(context.diagnostics().is_empty());
    assert!(context.only().is_empty());
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&context.to_json()).unwrap(),
        json!({"diagnostics": [], "only": []})
    );
    assert!(std::panic::catch_unwind(|| context.trigger_kind()).is_err());
    assert_eq!(context.attributes(), context.to_hash());
}
