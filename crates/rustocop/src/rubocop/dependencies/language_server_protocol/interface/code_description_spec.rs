use serde_json::json;

use super::CodeDescription;

#[test]
fn preserves_href_and_complete_public_surface() {
    let description = CodeDescription::new("https://example.com/E001");

    assert_eq!(description.href(), "https://example.com/E001");
    assert_eq!(
        description.to_hash(),
        json!({"href": "https://example.com/E001"})
            .as_object()
            .unwrap()
    );
    assert_eq!(description.attributes(), description.to_hash());
}

#[test]
fn retains_an_empty_required_href_and_serializes_it() {
    let description = CodeDescription::new("");

    assert_eq!(description.href(), "");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&description.to_json()).unwrap(),
        json!({"href": ""})
    );
}
