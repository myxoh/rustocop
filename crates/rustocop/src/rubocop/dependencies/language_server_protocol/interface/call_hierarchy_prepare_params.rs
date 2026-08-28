use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CallHierarchyPrepareParams {
    attributes: Attributes,
}

impl CallHierarchyPrepareParams {
    pub(crate) fn new(
        text_document: Value,
        position: Value,
        work_done_token: Option<Value>,
    ) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("textDocument", text_document);
        attributes.required("position", position);
        attributes.optional("workDoneToken", work_done_token);
        Self { attributes }
    }

    pub(crate) fn text_document(&self) -> &Value {
        self.attributes.fetch("textDocument")
    }

    pub(crate) fn position(&self) -> &Value {
        self.attributes.fetch("position")
    }

    pub(crate) fn work_done_token(&self) -> &Value {
        self.attributes.fetch("workDoneToken")
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
