use serde_json::json;

use super::CallHierarchyItem;

#[test]
fn preserves_all_required_and_present_optional_fields() {
    let range = json!({"start": {"line": 1}, "end": {"line": 3}});
    let selection = json!({"start": {"line": 1}, "end": {"line": 1}});
    let item = CallHierarchyItem::new(
        "call",
        12,
        Some(vec![1]),
        Some("call()"),
        "file:///a.rb",
        range.clone(),
        selection.clone(),
        Some(json!({"opaque": 1})),
    );

    assert_eq!(item.name(), "call");
    assert_eq!(item.kind(), 12);
    assert_eq!(item.tags(), &[1]);
    assert_eq!(item.detail(), "call()");
    assert_eq!(item.uri(), "file:///a.rb");
    assert_eq!(item.range(), &range);
    assert_eq!(item.selection_range(), &selection);
    assert_eq!(item.data(), &json!({"opaque": 1}));
    assert_eq!(item.attributes().len(), 8);
}

#[test]
fn omits_absent_optional_fields_and_serializes_protocol_keys() {
    let item = CallHierarchyItem::new(
        "call",
        12,
        None,
        None::<String>,
        "file:///a.rb",
        json!({}),
        json!({}),
        Some(json!(false)),
    );

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&item.to_json()).unwrap(),
        json!({
            "name": "call",
            "kind": 12,
            "uri": "file:///a.rb",
            "range": {},
            "selectionRange": {}
        })
    );
    assert!(std::panic::catch_unwind(|| item.tags()).is_err());
    assert!(std::panic::catch_unwind(|| item.detail()).is_err());
    assert!(std::panic::catch_unwind(|| item.data()).is_err());
    assert_eq!(item.to_hash(), item.attributes());
}
