use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ClientCapabilities {
    attributes: Attributes,
}

impl ClientCapabilities {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        workspace: Option<Value>,
        text_document: Option<Value>,
        notebook_document: Option<Value>,
        window: Option<Value>,
        general: Option<Value>,
        experimental: Option<Value>,
    ) -> Self {
        let mut attributes = Attributes::new();
        attributes.optional("workspace", workspace);
        attributes.optional("textDocument", text_document);
        attributes.optional("notebookDocument", notebook_document);
        attributes.optional("window", window);
        attributes.optional("general", general);
        attributes.optional("experimental", experimental);
        Self { attributes }
    }

    pub(crate) fn workspace(&self) -> &Value {
        self.attributes.fetch("workspace")
    }

    pub(crate) fn text_document(&self) -> &Value {
        self.attributes.fetch("textDocument")
    }

    pub(crate) fn notebook_document(&self) -> &Value {
        self.attributes.fetch("notebookDocument")
    }

    pub(crate) fn window(&self) -> &Value {
        self.attributes.fetch("window")
    }

    pub(crate) fn general(&self) -> &Value {
        self.attributes.fetch("general")
    }

    pub(crate) fn experimental(&self) -> &Value {
        self.attributes.fetch("experimental")
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
