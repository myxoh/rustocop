use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CodeLensParams {
    attributes: Attributes,
}

impl CodeLensParams {
    pub(crate) fn new(
        work_done_token: Option<Value>,
        partial_result_token: Option<Value>,
        text_document: Value,
    ) -> Self {
        let mut attributes = Attributes::new();
        attributes.optional("workDoneToken", work_done_token);
        attributes.optional("partialResultToken", partial_result_token);
        attributes.required("textDocument", text_document);
        Self { attributes }
    }

    pub(crate) fn work_done_token(&self) -> &Value {
        self.attributes.fetch("workDoneToken")
    }

    pub(crate) fn partial_result_token(&self) -> &Value {
        self.attributes.fetch("partialResultToken")
    }

    pub(crate) fn text_document(&self) -> &Value {
        self.attributes.fetch("textDocument")
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
