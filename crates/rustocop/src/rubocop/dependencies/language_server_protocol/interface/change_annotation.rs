use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ChangeAnnotation {
    attributes: Attributes,
}

impl ChangeAnnotation {
    pub(crate) fn new(
        label: impl Into<String>,
        needs_confirmation: Option<bool>,
        description: Option<impl Into<String>>,
    ) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("label", label.into());
        attributes.optional("needsConfirmation", needs_confirmation);
        attributes.optional("description", description.map(Into::into));
        Self { attributes }
    }

    pub(crate) fn label(&self) -> &str {
        self.attributes
            .fetch("label")
            .as_str()
            .expect("label is a string")
    }

    pub(crate) fn needs_confirmation(&self) -> bool {
        self.attributes
            .fetch("needsConfirmation")
            .as_bool()
            .expect("needsConfirmation is a boolean")
    }

    pub(crate) fn description(&self) -> &str {
        self.attributes
            .fetch("description")
            .as_str()
            .expect("description is a string")
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
