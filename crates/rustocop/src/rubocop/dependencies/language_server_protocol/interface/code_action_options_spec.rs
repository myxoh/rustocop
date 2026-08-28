use serde_json::json;

use super::CodeActionOptions;

#[test]
fn preserves_all_truthy_options() {
    let options = CodeActionOptions::new(
        Some(true),
        Some(vec!["quickfix".to_string(), "refactor".to_string()]),
        Some(true),
    );

    assert!(options.work_done_progress());
    assert_eq!(options.code_action_kinds(), &["quickfix", "refactor"]);
    assert!(options.resolve_provider());
    assert_eq!(options.attributes().len(), 3);
}

#[test]
fn omits_false_and_nil_booleans_but_retains_empty_kind_lists() {
    let options = CodeActionOptions::new(Some(false), Some(Vec::<String>::new()), None);

    assert!(options.code_action_kinds().is_empty());
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&options.to_json()).unwrap(),
        json!({"codeActionKinds": []})
    );
    assert!(std::panic::catch_unwind(|| options.work_done_progress()).is_err());
    assert!(std::panic::catch_unwind(|| options.resolve_provider()).is_err());
    assert_eq!(options.to_hash(), options.attributes());
}
