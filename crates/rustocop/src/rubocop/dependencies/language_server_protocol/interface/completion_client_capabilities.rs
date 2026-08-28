use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CompletionClientCapabilities {
    attributes: Attributes,
}

impl CompletionClientCapabilities {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        dynamic_registration: Option<bool>,
        completion_item: Option<Value>,
        completion_item_kind: Option<Value>,
        context_support: Option<bool>,
        insert_text_mode: Option<i64>,
        completion_list: Option<Value>,
    ) -> Self {
        let mut attributes = Attributes::new();
        attributes.optional("dynamicRegistration", dynamic_registration);
        attributes.optional("completionItem", completion_item);
        attributes.optional("completionItemKind", completion_item_kind);
        attributes.optional("contextSupport", context_support);
        attributes.optional("insertTextMode", insert_text_mode);
        attributes.optional("completionList", completion_list);
        Self { attributes }
    }

    pub(crate) fn dynamic_registration(&self) -> bool {
        self.boolean("dynamicRegistration")
    }

    pub(crate) fn completion_item(&self) -> &Value {
        self.attributes.fetch("completionItem")
    }

    pub(crate) fn completion_item_kind(&self) -> &Value {
        self.attributes.fetch("completionItemKind")
    }

    pub(crate) fn context_support(&self) -> bool {
        self.boolean("contextSupport")
    }

    pub(crate) fn insert_text_mode(&self) -> i64 {
        self.attributes
            .fetch("insertTextMode")
            .as_i64()
            .expect("insertTextMode is an integer")
    }

    pub(crate) fn completion_list(&self) -> &Value {
        self.attributes.fetch("completionList")
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

    fn boolean(&self, name: &str) -> bool {
        self.attributes
            .fetch(name)
            .as_bool()
            .expect("completion capability is a boolean")
    }
}
