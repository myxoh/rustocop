use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CallHierarchyOptions {
    attributes: Attributes,
}

impl CallHierarchyOptions {
    pub(crate) fn new(work_done_progress: Option<bool>) -> Self {
        let mut attributes = Attributes::new();
        attributes.optional("workDoneProgress", work_done_progress);
        Self { attributes }
    }

    pub(crate) fn work_done_progress(&self) -> bool {
        self.attributes
            .fetch("workDoneProgress")
            .as_bool()
            .expect("workDoneProgress is constructed from a boolean")
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
