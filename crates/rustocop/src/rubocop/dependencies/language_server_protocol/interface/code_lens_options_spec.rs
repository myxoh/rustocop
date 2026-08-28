use serde_json::json;

use super::CodeLensOptions;

#[test]
fn preserves_both_truthy_options() {
    let options = CodeLensOptions::new(Some(true), Some(true));

    assert!(options.work_done_progress());
    assert!(options.resolve_provider());
    assert_eq!(
        options.to_hash(),
        json!({"workDoneProgress": true, "resolveProvider": true})
            .as_object()
            .unwrap()
    );
}

#[test]
fn omits_nil_and_false_options() {
    let options = CodeLensOptions::new(Some(false), None);

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&options.to_json()).unwrap(),
        json!({})
    );
    assert!(std::panic::catch_unwind(|| options.work_done_progress()).is_err());
    assert!(std::panic::catch_unwind(|| options.resolve_provider()).is_err());
    assert_eq!(options.attributes(), options.to_hash());
}
