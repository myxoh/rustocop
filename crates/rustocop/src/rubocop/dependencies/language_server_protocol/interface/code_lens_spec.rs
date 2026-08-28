use serde_json::json;

use super::CodeLens;

#[test]
fn preserves_required_range_and_present_command_and_data() {
    let range = json!({"start": {"line": 1}, "end": {"line": 1}});
    let lens = CodeLens::new(
        range.clone(),
        Some(json!({"title": "Run", "command": "run"})),
        Some(json!({"id": 1})),
    );

    assert_eq!(lens.range(), &range);
    assert_eq!(lens.command(), &json!({"title": "Run", "command": "run"}));
    assert_eq!(lens.data(), &json!({"id": 1}));
    assert_eq!(lens.attributes().len(), 3);
}

#[test]
fn omits_nil_and_false_options_but_never_the_required_range() {
    let lens = CodeLens::new(json!({}), None, Some(json!(false)));

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&lens.to_json()).unwrap(),
        json!({"range": {}})
    );
    assert!(std::panic::catch_unwind(|| lens.command()).is_err());
    assert!(std::panic::catch_unwind(|| lens.data()).is_err());
    assert_eq!(lens.to_hash(), lens.attributes());
}
