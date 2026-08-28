use serde_json::json;

use super::ColorInformation;

#[test]
fn preserves_required_range_and_color_payloads() {
    let range = json!({"start": {"line": 1}, "end": {"line": 1}});
    let color = json!({"red": 1.0, "green": 0.0, "blue": 0.0, "alpha": 1.0});
    let information = ColorInformation::new(range.clone(), color.clone());

    assert_eq!(information.range(), &range);
    assert_eq!(information.color(), &color);
    assert_eq!(
        information.to_hash(),
        json!({"range": range, "color": color}).as_object().unwrap()
    );
}

#[test]
fn retains_empty_required_objects_and_serializes_them() {
    let information = ColorInformation::new(json!({}), json!({}));

    assert_eq!(information.attributes(), information.to_hash());
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&information.to_json()).unwrap(),
        json!({"range": {}, "color": {}})
    );
}
