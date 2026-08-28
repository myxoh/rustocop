use serde_json::json;

use super::ColorPresentation;

#[test]
fn preserves_required_label_and_present_edits() {
    let edit = json!({"range": {}, "newText": "#fff"});
    let additional = vec![json!({"range": {}, "newText": "#ffffff"})];
    let presentation =
        ColorPresentation::new("white", Some(edit.clone()), Some(additional.clone()));

    assert_eq!(presentation.label(), "white");
    assert_eq!(presentation.text_edit(), &edit);
    assert_eq!(presentation.additional_text_edits(), additional.as_slice());
    assert_eq!(presentation.attributes().len(), 3);
}

#[test]
fn omits_nil_or_false_edits_but_retains_empty_label_and_edit_array() {
    let presentation = ColorPresentation::new("", Some(json!(false)), Some(Vec::new()));

    assert_eq!(presentation.label(), "");
    assert!(presentation.additional_text_edits().is_empty());
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&presentation.to_json()).unwrap(),
        json!({"label": "", "additionalTextEdits": []})
    );
    assert!(std::panic::catch_unwind(|| presentation.text_edit()).is_err());
    assert_eq!(presentation.to_hash(), presentation.attributes());
}
