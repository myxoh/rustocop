use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Command {
    attributes: Attributes,
}

impl Command {
    pub(crate) fn new(
        title: impl Into<String>,
        command: impl Into<String>,
        arguments: Option<Vec<Value>>,
    ) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("title", title.into());
        attributes.required("command", command.into());
        attributes.optional("arguments", arguments);
        Self { attributes }
    }

    pub(crate) fn title(&self) -> &str {
        self.string("title")
    }

    pub(crate) fn command(&self) -> &str {
        self.string("command")
    }

    pub(crate) fn arguments(&self) -> &[Value] {
        self.attributes
            .fetch("arguments")
            .as_array()
            .expect("arguments is an array")
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
