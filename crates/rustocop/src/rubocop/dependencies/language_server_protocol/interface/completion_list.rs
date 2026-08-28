use super::attributes::Attributes;
use serde_json::{Map, Value};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CompletionList {
    attributes: Attributes,
}

impl CompletionList {
    pub(crate) fn new(
        is_incomplete: bool,
        item_defaults: Option<Value>,
        items: Vec<Value>,
    ) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("isIncomplete", is_incomplete);
        attributes.optional("itemDefaults", item_defaults);
        attributes.required("items", items);
        Self { attributes }
    }
    pub(crate) fn is_incomplete(&self) -> bool {
        self.attributes
            .fetch("isIncomplete")
            .as_bool()
            .expect("boolean isIncomplete")
    }
    pub(crate) fn item_defaults(&self) -> &Value {
        self.attributes.fetch("itemDefaults")
    }
    pub(crate) fn items(&self) -> &[Value] {
        self.attributes
            .fetch("items")
            .as_array()
            .expect("items is an array")
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
