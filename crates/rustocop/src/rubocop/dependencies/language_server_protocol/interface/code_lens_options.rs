use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CodeLensOptions {
    attributes: Attributes,
}

impl CodeLensOptions {
    pub(crate) fn new(work_done_progress: Option<bool>, resolve_provider: Option<bool>) -> Self {
        let mut attributes = Attributes::new();
        attributes.optional("workDoneProgress", work_done_progress);
        attributes.optional("resolveProvider", resolve_provider);
        Self { attributes }
    }

    pub(crate) fn work_done_progress(&self) -> bool {
        self.boolean("workDoneProgress")
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
            .expect("code lens option is a boolean")
    }
}
