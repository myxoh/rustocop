use serde_json::json;

use super::CallHierarchyIncomingCall;

#[test]
fn preserves_required_caller_and_ranges() {
    let caller = json!({"name": "target", "uri": "file:///a.rb"});
    let ranges = vec![json!({"start": {"line": 1}, "end": {"line": 2}})];
    let call = CallHierarchyIncomingCall::new(caller.clone(), ranges.clone());

    assert_eq!(call.from(), &caller);
    assert_eq!(call.from_ranges(), ranges.as_slice());
    assert_eq!(
        call.to_hash(),
        json!({"from": caller, "fromRanges": ranges})
            .as_object()
            .unwrap()
    );
}

#[test]
fn serializes_empty_required_ranges_without_omitting_them() {
    let call = CallHierarchyIncomingCall::new(json!({"name": "target"}), Vec::new());

    assert!(call.from_ranges().is_empty());
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&call.to_json()).unwrap(),
        json!({"from": {"name": "target"}, "fromRanges": []})
    );
}
