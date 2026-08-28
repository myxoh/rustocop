use serde_json::json;

use super::ApplyWorkspaceEditResult;

#[test]
fn preserves_applied_and_present_failure_details() {
    let result = ApplyWorkspaceEditResult::new(false, Some("Conflict"), Some(0));

    assert!(!result.applied());
    assert_eq!(result.failure_reason(), "Conflict");
    assert_eq!(result.failed_change(), 0);
    assert_eq!(
        result.to_hash(),
        json!({"applied": false, "failureReason": "Conflict", "failedChange": 0})
            .as_object()
            .unwrap()
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&result.to_json()).unwrap(),
        json!({"applied": false, "failureReason": "Conflict", "failedChange": 0})
    );
}

#[test]
fn omits_absent_optional_failure_details() {
    let result = ApplyWorkspaceEditResult::new(true, None::<String>, None);

    assert!(result.applied());
    assert_eq!(
        result.attributes(),
        json!({"applied": true}).as_object().unwrap()
    );
    assert!(std::panic::catch_unwind(|| result.failure_reason()).is_err());
    assert!(std::panic::catch_unwind(|| result.failed_change()).is_err());
}
