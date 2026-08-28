use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CodeActionClientCapabilities {
    attributes: Attributes,
}

impl CodeActionClientCapabilities {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        dynamic_registration: Option<bool>,
        code_action_literal_support: Option<Value>,
        is_preferred_support: Option<bool>,
        disabled_support: Option<bool>,
        data_support: Option<bool>,
        resolve_support: Option<Value>,
        honors_change_annotations: Option<bool>,
    ) -> Self {
        let mut attributes = Attributes::new();
        attributes.optional("dynamicRegistration", dynamic_registration);
        attributes.optional("codeActionLiteralSupport", code_action_literal_support);
        attributes.optional("isPreferredSupport", is_preferred_support);
        attributes.optional("disabledSupport", disabled_support);
        attributes.optional("dataSupport", data_support);
        attributes.optional("resolveSupport", resolve_support);
        attributes.optional("honorsChangeAnnotations", honors_change_annotations);
        Self { attributes }
    }

    pub(crate) fn dynamic_registration(&self) -> bool {
        self.boolean("dynamicRegistration")
    }

    pub(crate) fn code_action_literal_support(&self) -> &Value {
        self.attributes.fetch("codeActionLiteralSupport")
    }

    pub(crate) fn is_preferred_support(&self) -> bool {
        self.boolean("isPreferredSupport")
    }

    pub(crate) fn disabled_support(&self) -> bool {
        self.boolean("disabledSupport")
    }

    pub(crate) fn data_support(&self) -> bool {
        self.boolean("dataSupport")
    }

    pub(crate) fn resolve_support(&self) -> &Value {
        self.attributes.fetch("resolveSupport")
    }

    pub(crate) fn honors_change_annotations(&self) -> bool {
        self.boolean("honorsChangeAnnotations")
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
            .expect("boolean capability")
    }
}
