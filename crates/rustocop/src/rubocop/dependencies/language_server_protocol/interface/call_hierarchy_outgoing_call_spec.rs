use serde_json::json;

use super::CallHierarchyOutgoingCall;

#[test]
fn preserves_required_target_and_ranges() {
    let target = json!({"name": "callee"});
    let ranges = vec![json!({"start": {"line": 1}, "end": {"line": 1}})];
    let call = CallHierarchyOutgoingCall::new(target.clone(), ranges.clone());

    assert_eq!(call.to(), &target);
    assert_eq!(call.from_ranges(), ranges.as_slice());
    assert_eq!(
        call.to_hash(),
        json!({"to": target, "fromRanges": ranges})
            .as_object()
            .unwrap()
    );
}

#[test]
fn serializes_empty_required_ranges_without_omitting_them() {
    let call = CallHierarchyOutgoingCall::new(json!({"name": "callee"}), Vec::new());

    assert!(call.from_ranges().is_empty());
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&call.to_json()).unwrap(),
        json!({"to": {"name": "callee"}, "fromRanges": []})
    );
}
