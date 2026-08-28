use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CallHierarchyIncomingCall {
    attributes: Attributes,
}

impl CallHierarchyIncomingCall {
    pub(crate) fn new(from: Value, from_ranges: Vec<Value>) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("from", from);
        attributes.required("fromRanges", from_ranges);
        Self { attributes }
    }

    pub(crate) fn from(&self) -> &Value {
        self.attributes.fetch("from")
    }

    pub(crate) fn from_ranges(&self) -> &[Value] {
        self.attributes
            .fetch("fromRanges")
            .as_array()
            .expect("fromRanges is constructed from an array")
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
