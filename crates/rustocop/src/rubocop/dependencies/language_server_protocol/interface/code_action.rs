use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CodeAction {
    attributes: Attributes,
}

impl CodeAction {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        title: impl Into<String>,
        kind: Option<impl Into<String>>,
        diagnostics: Option<Vec<Value>>,
        is_preferred: Option<bool>,
        disabled: Option<Value>,
        edit: Option<Value>,
        command: Option<Value>,
        data: Option<Value>,
    ) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("title", title.into());
        attributes.optional("kind", kind.map(Into::into));
        attributes.optional("diagnostics", diagnostics);
        attributes.optional("isPreferred", is_preferred);
        attributes.optional("disabled", disabled);
        attributes.optional("edit", edit);
        attributes.optional("command", command);
        attributes.optional("data", data);
        Self { attributes }
    }

    pub(crate) fn title(&self) -> &str {
        self.string("title")
    }

    pub(crate) fn kind(&self) -> &str {
        self.string("kind")
    }

    pub(crate) fn diagnostics(&self) -> &[Value] {
        self.attributes
            .fetch("diagnostics")
            .as_array()
            .expect("diagnostics is an array")
    }

    pub(crate) fn is_preferred(&self) -> bool {
        self.attributes
            .fetch("isPreferred")
            .as_bool()
            .expect("isPreferred is a boolean")
    }

    pub(crate) fn disabled(&self) -> &Value {
        self.attributes.fetch("disabled")
    }

    pub(crate) fn edit(&self) -> &Value {
        self.attributes.fetch("edit")
    }

    pub(crate) fn command(&self) -> &Value {
        self.attributes.fetch("command")
    }

    pub(crate) fn data(&self) -> &Value {
        self.attributes.fetch("data")
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

    fn string(&self, name: &str) -> &str {
        self.attributes.fetch(name).as_str().expect("string field")
    }
}
