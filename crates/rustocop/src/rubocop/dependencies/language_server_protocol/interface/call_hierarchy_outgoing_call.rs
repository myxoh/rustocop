use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CallHierarchyOutgoingCall {
    attributes: Attributes,
}

impl CallHierarchyOutgoingCall {
    pub(crate) fn new(to: Value, from_ranges: Vec<Value>) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("to", to);
        attributes.required("fromRanges", from_ranges);
        Self { attributes }
    }

    pub(crate) fn to(&self) -> &Value {
        self.attributes.fetch("to")
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
