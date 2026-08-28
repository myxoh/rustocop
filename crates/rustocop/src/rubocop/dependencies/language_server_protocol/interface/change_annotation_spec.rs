use serde_json::json;

use super::ChangeAnnotation;

#[test]
fn preserves_required_label_and_present_optional_fields() {
    let annotation = ChangeAnnotation::new("Rename", Some(true), Some("Updates references"));

    assert_eq!(annotation.label(), "Rename");
    assert!(annotation.needs_confirmation());
    assert_eq!(annotation.description(), "Updates references");
    assert_eq!(
        annotation.to_hash(),
        json!({
            "label": "Rename",
            "needsConfirmation": true,
            "description": "Updates references"
        })
        .as_object()
        .unwrap()
    );
}

#[test]
fn omits_false_and_nil_optional_fields_but_retains_empty_strings() {
    let omitted = ChangeAnnotation::new("Rename", Some(false), None::<String>);
    assert_eq!(
        omitted.attributes(),
        json!({"label": "Rename"}).as_object().unwrap()
    );
    assert!(std::panic::catch_unwind(|| omitted.needs_confirmation()).is_err());
    assert!(std::panic::catch_unwind(|| omitted.description()).is_err());

    let empty = ChangeAnnotation::new("", None, Some(""));
    assert_eq!(empty.label(), "");
    assert_eq!(empty.description(), "");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&empty.to_json()).unwrap(),
        json!({"label": "", "description": ""})
    );
}
