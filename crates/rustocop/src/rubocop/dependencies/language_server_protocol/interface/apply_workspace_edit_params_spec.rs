use serde_json::json;

use super::ApplyWorkspaceEditParams;

#[test]
fn preserves_present_optional_label_and_required_edit() {
    let edit = json!({"changes": {"file:///a.rb": []}});
    let params = ApplyWorkspaceEditParams::new(Some("Rename"), edit.clone());

    assert_eq!(params.label(), "Rename");
    assert_eq!(params.edit(), &edit);
    assert_eq!(
        params.to_hash(),
        &json!({"label": "Rename", "edit": edit})
            .as_object()
            .unwrap()
            .clone()
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&params.to_json()).unwrap(),
        json!({"label": "Rename", "edit": {"changes": {"file:///a.rb": []}}})
    );
}

#[test]
fn omits_absent_label_but_retains_an_empty_string() {
    let absent = ApplyWorkspaceEditParams::new(None::<String>, json!({}));
    assert_eq!(
        absent.attributes(),
        json!({"edit": {}}).as_object().unwrap()
    );
    assert!(std::panic::catch_unwind(|| absent.label()).is_err());

    let empty = ApplyWorkspaceEditParams::new(Some(""), json!({}));
    assert_eq!(empty.label(), "");
    assert!(empty.attributes().contains_key("label"));
}
