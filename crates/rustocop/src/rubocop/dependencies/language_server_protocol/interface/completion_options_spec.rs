use super::CompletionOptions;
use serde_json::json;

#[test]
fn preserves_all_present_options() {
    let value = CompletionOptions::new(
        Some(true),
        Some(vec![".".into()]),
        Some(vec!["(".into()]),
        Some(true),
        Some(json!({"labelDetailsSupport":true})),
    );
    assert!(value.work_done_progress());
    assert_eq!(value.trigger_characters(), vec!["."]);
    assert_eq!(value.all_commit_characters(), vec!["("]);
    assert!(value.resolve_provider());
    assert_eq!(
        value.completion_item(),
        &json!({"labelDetailsSupport":true})
    );
    assert_eq!(value.attributes().len(), 5);
}

#[test]
fn omits_false_but_retains_empty_arrays_and_objects() {
    let value = CompletionOptions::new(
        Some(false),
        Some(Vec::new()),
        Some(Vec::new()),
        None,
        Some(json!({})),
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&value.to_json()).unwrap(),
        json!({"triggerCharacters":[],"allCommitCharacters":[],"completionItem":{}})
    );
    assert!(std::panic::catch_unwind(|| value.work_done_progress()).is_err());
    assert!(std::panic::catch_unwind(|| value.resolve_provider()).is_err());
    assert_eq!(value.attributes(), value.to_hash());
}
