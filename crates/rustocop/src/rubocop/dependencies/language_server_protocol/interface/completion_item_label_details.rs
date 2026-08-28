use super::attributes::Attributes;
use serde_json::{Map, Value};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CompletionItemLabelDetails {
    attributes: Attributes,
}

impl CompletionItemLabelDetails {
    pub(crate) fn new(
        detail: Option<impl Into<String>>,
        description: Option<impl Into<String>>,
    ) -> Self {
        let mut attributes = Attributes::new();
        attributes.optional("detail", detail.map(Into::into));
        attributes.optional("description", description.map(Into::into));
        Self { attributes }
    }
    pub(crate) fn detail(&self) -> &str {
        self.string("detail")
    }
    pub(crate) fn description(&self) -> &str {
        self.string("description")
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
        self.attributes.fetch(name).as_str().expect("string detail")
    }
}
