use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CodeActionContext {
    attributes: Attributes,
}

impl CodeActionContext {
    pub(crate) fn new(
        diagnostics: Vec<Value>,
        only: Option<Vec<String>>,
        trigger_kind: Option<i64>,
    ) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("diagnostics", diagnostics);
        attributes.optional("only", only);
        attributes.optional("triggerKind", trigger_kind);
        Self { attributes }
    }

    pub(crate) fn diagnostics(&self) -> &[Value] {
        self.attributes
            .fetch("diagnostics")
            .as_array()
            .expect("diagnostics is an array")
    }

    pub(crate) fn only(&self) -> Vec<&str> {
        self.attributes
            .fetch("only")
            .as_array()
            .expect("only is an array")
            .iter()
            .map(|kind| kind.as_str().expect("code action kind is a string"))
            .collect()
    }

    pub(crate) fn trigger_kind(&self) -> i64 {
        self.attributes
            .fetch("triggerKind")
            .as_i64()
            .expect("triggerKind is an integer")
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
