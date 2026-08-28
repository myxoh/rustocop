use serde_json::json;

use super::Color;

#[test]
fn preserves_all_required_rgba_components() {
    let color = Color::new(0.125, 0.25, 0.5, 1.0);

    assert_eq!(color.red(), 0.125);
    assert_eq!(color.green(), 0.25);
    assert_eq!(color.blue(), 0.5);
    assert_eq!(color.alpha(), 1.0);
    assert_eq!(
        color.to_hash(),
        json!({"red": 0.125, "green": 0.25, "blue": 0.5, "alpha": 1.0})
            .as_object()
            .unwrap()
    );
}

#[test]
fn retains_zero_and_boundary_components_in_json() {
    let color = Color::new(0.0, 1.0, 0.0, 1.0);

    assert_eq!(color.attributes(), color.to_hash());
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&color.to_json()).unwrap(),
        json!({"red": 0.0, "green": 1.0, "blue": 0.0, "alpha": 1.0})
    );
}
