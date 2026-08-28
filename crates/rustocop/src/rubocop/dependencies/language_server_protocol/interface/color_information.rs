use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ColorInformation {
    attributes: Attributes,
}

impl ColorInformation {
    pub(crate) fn new(range: Value, color: Value) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("range", range);
        attributes.required("color", color);
        Self { attributes }
    }

    pub(crate) fn range(&self) -> &Value {
        self.attributes.fetch("range")
    }

    pub(crate) fn color(&self) -> &Value {
        self.attributes.fetch("color")
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
