use serde_json::json;

use super::CancelParams;

#[test]
fn preserves_a_numeric_request_id() {
    let params = CancelParams::new(json!(17));

    assert_eq!(params.id(), &json!(17));
    assert_eq!(params.to_hash(), json!({"id": 17}).as_object().unwrap());
}

#[test]
fn preserves_a_string_request_id_and_serializes_it() {
    let params = CancelParams::new(json!("request-17"));

    assert_eq!(params.id(), &json!("request-17"));
    assert_eq!(params.attributes(), params.to_hash());
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&params.to_json()).unwrap(),
        json!({"id": "request-17"})
    );
}
