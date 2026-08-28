use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CodeDescription {
    attributes: Attributes,
}

impl CodeDescription {
    pub(crate) fn new(href: impl Into<String>) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("href", href.into());
        Self { attributes }
    }

    pub(crate) fn href(&self) -> &str {
        self.attributes
            .fetch("href")
            .as_str()
            .expect("href is a string")
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
