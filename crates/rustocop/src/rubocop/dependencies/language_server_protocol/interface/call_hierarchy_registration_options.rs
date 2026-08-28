use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CallHierarchyRegistrationOptions {
    attributes: Attributes,
}

impl CallHierarchyRegistrationOptions {
    pub(crate) fn new(
        document_selector: Value,
        work_done_progress: Option<bool>,
        id: Option<impl Into<String>>,
    ) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("documentSelector", document_selector);
        attributes.optional("workDoneProgress", work_done_progress);
        attributes.optional("id", id.map(Into::into));
        Self { attributes }
    }

    pub(crate) fn document_selector(&self) -> &Value {
        self.attributes.fetch("documentSelector")
    }

    pub(crate) fn work_done_progress(&self) -> bool {
        self.attributes
            .fetch("workDoneProgress")
            .as_bool()
            .expect("workDoneProgress is constructed from a boolean")
    }

    pub(crate) fn id(&self) -> &str {
        self.attributes
            .fetch("id")
            .as_str()
            .expect("id is a string")
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
