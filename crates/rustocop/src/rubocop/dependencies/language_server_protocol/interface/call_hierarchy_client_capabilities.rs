use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CallHierarchyClientCapabilities {
    attributes: Attributes,
}

impl CallHierarchyClientCapabilities {
    pub(crate) fn new(dynamic_registration: Option<bool>) -> Self {
        let mut attributes = Attributes::new();
        attributes.optional("dynamicRegistration", dynamic_registration);
        Self { attributes }
    }

    pub(crate) fn dynamic_registration(&self) -> bool {
        self.attributes
            .fetch("dynamicRegistration")
            .as_bool()
            .expect("dynamicRegistration is constructed from a boolean")
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
