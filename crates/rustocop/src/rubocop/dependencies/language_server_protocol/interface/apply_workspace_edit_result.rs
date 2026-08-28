use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ApplyWorkspaceEditResult {
    attributes: Attributes,
}

impl ApplyWorkspaceEditResult {
    pub(crate) fn new(
        applied: bool,
        failure_reason: Option<impl Into<String>>,
        failed_change: Option<i64>,
    ) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("applied", applied);
        attributes.optional("failureReason", failure_reason.map(Into::into));
        attributes.optional("failedChange", failed_change);
        Self { attributes }
    }

    pub(crate) fn applied(&self) -> bool {
        self.attributes
            .fetch("applied")
            .as_bool()
            .expect("applied is constructed from a boolean")
    }

    pub(crate) fn failure_reason(&self) -> &str {
        self.attributes
            .fetch("failureReason")
            .as_str()
            .expect("failureReason is constructed from a string")
    }

    pub(crate) fn failed_change(&self) -> i64 {
        self.attributes
            .fetch("failedChange")
            .as_i64()
            .expect("failedChange is constructed from an integer")
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
