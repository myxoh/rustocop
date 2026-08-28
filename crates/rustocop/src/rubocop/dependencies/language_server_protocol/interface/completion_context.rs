use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CompletionContext {
    attributes: Attributes,
}

impl CompletionContext {
    pub(crate) fn new(trigger_kind: i64, trigger_character: Option<impl Into<String>>) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("triggerKind", trigger_kind);
        attributes.optional("triggerCharacter", trigger_character.map(Into::into));
        Self { attributes }
    }

    pub(crate) fn trigger_kind(&self) -> i64 {
        self.attributes
            .fetch("triggerKind")
            .as_i64()
            .expect("triggerKind is an integer")
    }

    pub(crate) fn trigger_character(&self) -> &str {
        self.attributes
            .fetch("triggerCharacter")
            .as_str()
            .expect("triggerCharacter is a string")
    }

    pub(crate) fn attributes(&self) -> &Map<String, Value> {
        self.attributes.as_map()
    }

    pub(crate) fn to_hash(&self) -> &Map<String, Value> {
        self.attributes()
    }

    pub(crate) fn to_json(&self) -> String {
        self.attributes.to_json()
    }
}
