use serde_json::json;

use super::CompletionItemLabelDetails;

#[test]
fn preserves_both_present_label_details() {
    let details = CompletionItemLabelDetails::new(Some("(value)"), Some("Enumerable#map"));
    assert_eq!(details.detail(), "(value)");
    assert_eq!(details.description(), "Enumerable#map");
    assert_eq!(
        details.to_hash(),
        json!({"detail":"(value)","description":"Enumerable#map"})
            .as_object()
            .unwrap()
    );
}

#[test]
fn omits_nil_but_retains_empty_strings() {
    let details = CompletionItemLabelDetails::new(None::<String>, Some(""));
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&details.to_json()).unwrap(),
        json!({"description":""})
    );
    assert!(std::panic::catch_unwind(|| details.detail()).is_err());
    assert_eq!(details.description(), "");
    assert_eq!(details.attributes(), details.to_hash());
}
