use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CodeLensWorkspaceClientCapabilities {
    attributes: Attributes,
}

impl CodeLensWorkspaceClientCapabilities {
    pub(crate) fn new(refresh_support: Option<bool>) -> Self {
        let mut attributes = Attributes::new();
        attributes.optional("refreshSupport", refresh_support);
        Self { attributes }
    }

    pub(crate) fn refresh_support(&self) -> bool {
        self.attributes
            .fetch("refreshSupport")
            .as_bool()
            .expect("refreshSupport is a boolean")
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
