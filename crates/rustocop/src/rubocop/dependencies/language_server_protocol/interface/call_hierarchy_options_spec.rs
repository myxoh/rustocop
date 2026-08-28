use serde_json::json;

use super::CallHierarchyOptions;

#[test]
fn includes_truthy_work_done_progress() {
    let options = CallHierarchyOptions::new(Some(true));

    assert!(options.work_done_progress());
    assert_eq!(
        options.to_hash(),
        json!({"workDoneProgress": true}).as_object().unwrap()
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&options.to_json()).unwrap(),
        json!({"workDoneProgress": true})
    );
}

#[test]
fn omits_nil_and_false_work_done_progress() {
    for options in [
        CallHierarchyOptions::new(None),
        CallHierarchyOptions::new(Some(false)),
    ] {
        assert!(options.attributes().is_empty());
        assert!(std::panic::catch_unwind(|| options.work_done_progress()).is_err());
    }
}
