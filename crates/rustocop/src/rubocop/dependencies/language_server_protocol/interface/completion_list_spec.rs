use serde_json::json;

use super::CompletionList;

#[test]
fn preserves_required_incomplete_and_items_plus_present_defaults() {
    let items = vec![json!({"label":"map"})];
    let list = CompletionList::new(
        false,
        Some(json!({"commitCharacters":["("]})),
        items.clone(),
    );
    assert!(!list.is_incomplete());
    assert_eq!(list.item_defaults(), &json!({"commitCharacters":["("]}));
    assert_eq!(list.items(), items.as_slice());
    assert_eq!(list.attributes().len(), 3);
}

#[test]
fn omits_false_defaults_but_never_required_false_or_empty_items() {
    let list = CompletionList::new(false, Some(json!(false)), Vec::new());
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&list.to_json()).unwrap(),
        json!({"isIncomplete":false,"items":[]})
    );
    assert!(std::panic::catch_unwind(|| list.item_defaults()).is_err());
    assert!(list.items().is_empty());
    assert_eq!(list.attributes(), list.to_hash());
}
