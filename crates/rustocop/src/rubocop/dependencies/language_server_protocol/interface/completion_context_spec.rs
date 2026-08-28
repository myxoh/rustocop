use serde_json::json;

use super::CompletionContext;

#[test]
fn preserves_required_trigger_kind_and_present_character() {
    let context = CompletionContext::new(2, Some("."));

    assert_eq!(context.trigger_kind(), 2);
    assert_eq!(context.trigger_character(), ".");
    assert_eq!(
        context.to_hash(),
        json!({"triggerKind": 2, "triggerCharacter": "."})
            .as_object()
            .unwrap()
    );
}

#[test]
fn omits_nil_character_but_retains_an_empty_truthy_character() {
    let absent = CompletionContext::new(1, None::<String>);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&absent.to_json()).unwrap(),
        json!({"triggerKind": 1})
    );
    assert!(std::panic::catch_unwind(|| absent.trigger_character()).is_err());

    let empty = CompletionContext::new(3, Some(""));
    assert_eq!(empty.trigger_character(), "");
    assert_eq!(empty.to_hash(), empty.attributes());
}
