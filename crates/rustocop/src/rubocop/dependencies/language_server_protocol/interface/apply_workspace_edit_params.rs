use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ApplyWorkspaceEditParams {
    attributes: Attributes,
}

impl ApplyWorkspaceEditParams {
    pub(crate) fn new(label: Option<impl Into<String>>, edit: Value) -> Self {
        let mut attributes = Attributes::new();
        attributes.optional("label", label.map(Into::into));
        attributes.required("edit", edit);
        Self { attributes }
    }

    pub(crate) fn label(&self) -> &str {
        self.attributes
            .fetch("label")
            .as_str()
            .expect("label is constructed from a string")
    }

    pub(crate) fn edit(&self) -> &Value {
        self.attributes.fetch("edit")
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
