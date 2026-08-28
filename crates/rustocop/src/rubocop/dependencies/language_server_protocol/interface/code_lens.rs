use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CodeLens {
    attributes: Attributes,
}

impl CodeLens {
    pub(crate) fn new(range: Value, command: Option<Value>, data: Option<Value>) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("range", range);
        attributes.optional("command", command);
        attributes.optional("data", data);
        Self { attributes }
    }

    pub(crate) fn range(&self) -> &Value {
        self.attributes.fetch("range")
    }

    pub(crate) fn command(&self) -> &Value {
        self.attributes.fetch("command")
    }

    pub(crate) fn data(&self) -> &Value {
        self.attributes.fetch("data")
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
