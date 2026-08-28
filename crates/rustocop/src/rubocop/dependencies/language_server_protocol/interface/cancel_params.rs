use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CancelParams {
    attributes: Attributes,
}

impl CancelParams {
    pub(crate) fn new(id: Value) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("id", id);
        Self { attributes }
    }

    pub(crate) fn id(&self) -> &Value {
        self.attributes.fetch("id")
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
