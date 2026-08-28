use serde_json::json;

use super::Command;

#[test]
fn preserves_required_strings_and_present_arguments() {
    let command = Command::new("Save", "workspace.save", Some(vec![json!(1), json!("all")]));

    assert_eq!(command.title(), "Save");
    assert_eq!(command.command(), "workspace.save");
    assert_eq!(command.arguments(), &[json!(1), json!("all")]);
    assert_eq!(command.attributes().len(), 3);
}

#[test]
fn omits_nil_arguments_but_retains_empty_strings_and_arrays() {
    let no_arguments = Command::new("", "", None);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&no_arguments.to_json()).unwrap(),
        json!({"title": "", "command": ""})
    );
    assert!(std::panic::catch_unwind(|| no_arguments.arguments()).is_err());

    let empty_arguments = Command::new("Save", "save", Some(Vec::new()));
    assert!(empty_arguments.arguments().is_empty());
    assert_eq!(empty_arguments.to_hash(), empty_arguments.attributes());
}
