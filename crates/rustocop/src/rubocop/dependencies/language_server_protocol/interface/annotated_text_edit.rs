use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AnnotatedTextEdit {
    attributes: Attributes,
}

impl AnnotatedTextEdit {
    pub(crate) fn new(
        range: Value,
        new_text: impl Into<String>,
        annotation_id: impl Into<String>,
    ) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("range", range);
        attributes.required("newText", new_text.into());
        attributes.required("annotationId", annotation_id.into());
        Self { attributes }
    }

    pub(crate) fn range(&self) -> &Value {
        self.attributes.fetch("range")
    }

    pub(crate) fn new_text(&self) -> &str {
        self.attributes
            .fetch("newText")
            .as_str()
            .expect("newText is constructed from a string")
    }

    pub(crate) fn annotation_id(&self) -> &str {
        self.attributes
            .fetch("annotationId")
            .as_str()
            .expect("annotationId is constructed from a string")
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
