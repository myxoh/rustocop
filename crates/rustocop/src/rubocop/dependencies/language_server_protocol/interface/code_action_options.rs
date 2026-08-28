use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CodeActionOptions {
    attributes: Attributes,
}

impl CodeActionOptions {
    pub(crate) fn new(
        work_done_progress: Option<bool>,
        code_action_kinds: Option<Vec<String>>,
        resolve_provider: Option<bool>,
    ) -> Self {
        let mut attributes = Attributes::new();
        attributes.optional("workDoneProgress", work_done_progress);
        attributes.optional("codeActionKinds", code_action_kinds);
        attributes.optional("resolveProvider", resolve_provider);
        Self { attributes }
    }

    pub(crate) fn work_done_progress(&self) -> bool {
        self.boolean("workDoneProgress")
    }

    pub(crate) fn code_action_kinds(&self) -> Vec<&str> {
        self.attributes
            .fetch("codeActionKinds")
            .as_array()
            .expect("codeActionKinds is an array")
            .iter()
            .map(|kind| kind.as_str().expect("code action kind is a string"))
            .collect()
    }

    pub(crate) fn resolve_provider(&self) -> bool {
        self.boolean("resolveProvider")
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
            .expect("boolean option")
    }
}
