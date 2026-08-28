use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ColorPresentation {
    attributes: Attributes,
}

impl ColorPresentation {
    pub(crate) fn new(
        label: impl Into<String>,
        text_edit: Option<Value>,
        additional_text_edits: Option<Vec<Value>>,
    ) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("label", label.into());
        attributes.optional("textEdit", text_edit);
        attributes.optional("additionalTextEdits", additional_text_edits);
        Self { attributes }
    }

    pub(crate) fn label(&self) -> &str {
        self.attributes
            .fetch("label")
            .as_str()
            .expect("label is a string")
    }

    pub(crate) fn text_edit(&self) -> &Value {
        self.attributes.fetch("textEdit")
    }

    pub(crate) fn additional_text_edits(&self) -> &[Value] {
        self.attributes
            .fetch("additionalTextEdits")
            .as_array()
            .expect("additionalTextEdits is an array")
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
