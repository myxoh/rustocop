use serde_json::json;

use super::CompletionItem;

#[test]
fn preserves_every_present_completion_item_field() {
    let item = CompletionItem::new(
        "map",
        Some(json!({"detail": "Enumerable"})),
        Some(3),
        Some(vec![1]),
        Some("method"),
        Some(json!({"kind": "markdown", "value": "Maps values"})),
        Some(true),
        Some(true),
        Some("001"),
        Some("map"),
        Some("map { $1 }"),
        Some(2),
        Some(2),
        Some(json!({"range": {}, "newText": "map"})),
        Some("map"),
        Some(vec![json!({"range": {}, "newText": "require 'x'"})]),
        Some(vec!["(".to_string()]),
        Some(json!({"title": "After", "command": "after"})),
        Some(json!({"opaque": 1})),
    );

    assert_eq!(item.label(), "map");
    assert_eq!(item.label_details(), &json!({"detail": "Enumerable"}));
    assert_eq!(item.kind(), 3);
    assert_eq!(item.tags(), vec![1]);
    assert_eq!(item.detail(), "method");
    assert_eq!(
        item.documentation(),
        &json!({"kind": "markdown", "value": "Maps values"})
    );
    assert!(item.deprecated());
    assert!(item.preselect());
    assert_eq!(item.sort_text(), "001");
    assert_eq!(item.filter_text(), "map");
    assert_eq!(item.insert_text(), "map { $1 }");
    assert_eq!(item.insert_text_format(), 2);
    assert_eq!(item.insert_text_mode(), 2);
    assert_eq!(item.text_edit(), &json!({"range": {}, "newText": "map"}));
    assert_eq!(item.text_edit_text(), "map");
    assert_eq!(item.additional_text_edits().len(), 1);
    assert_eq!(item.commit_characters(), vec!["("]);
    assert_eq!(
        item.command(),
        &json!({"title": "After", "command": "after"})
    );
    assert_eq!(item.data(), &json!({"opaque": 1}));
    assert_eq!(item.attributes().len(), 19);
}

#[test]
fn preserves_ruby_truthiness_for_absent_false_and_empty_optional_values() {
    let item = CompletionItem::new(
        "",
        None,
        None,
        Some(Vec::new()),
        Some(""),
        Some(json!(false)),
        Some(false),
        Some(false),
        None::<String>,
        None::<String>,
        None::<String>,
        None,
        None,
        None,
        Some(""),
        Some(Vec::new()),
        Some(Vec::new()),
        None,
        Some(json!(false)),
    );

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&item.to_json()).unwrap(),
        json!({
            "label": "",
            "tags": [],
            "detail": "",
            "textEditText": "",
            "additionalTextEdits": [],
            "commitCharacters": []
        })
    );
    assert!(std::panic::catch_unwind(|| item.documentation()).is_err());
    assert!(std::panic::catch_unwind(|| item.deprecated()).is_err());
    assert!(std::panic::catch_unwind(|| item.data()).is_err());
    assert_eq!(item.to_hash(), item.attributes());
}
