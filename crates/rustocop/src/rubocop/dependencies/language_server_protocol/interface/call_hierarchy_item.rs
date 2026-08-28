use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CallHierarchyItem {
    attributes: Attributes,
}

impl CallHierarchyItem {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        name: impl Into<String>,
        kind: i64,
        tags: Option<Vec<i64>>,
        detail: Option<impl Into<String>>,
        uri: impl Into<String>,
        range: Value,
        selection_range: Value,
        data: Option<Value>,
    ) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("name", name.into());
        attributes.required("kind", kind);
        attributes.optional("tags", tags);
        attributes.optional("detail", detail.map(Into::into));
        attributes.required("uri", uri.into());
        attributes.required("range", range);
        attributes.required("selectionRange", selection_range);
        attributes.optional("data", data);
        Self { attributes }
    }

    pub(crate) fn name(&self) -> &str {
        self.attributes.fetch("name").as_str().expect("string name")
    }

    pub(crate) fn kind(&self) -> i64 {
        self.attributes
            .fetch("kind")
            .as_i64()
            .expect("integer kind")
    }

    pub(crate) fn tags(&self) -> Vec<i64> {
        self.attributes
            .fetch("tags")
            .as_array()
            .expect("array tags")
            .iter()
            .map(|tag| tag.as_i64().expect("integer tag"))
            .collect()
    }

    pub(crate) fn detail(&self) -> &str {
        self.attributes
            .fetch("detail")
            .as_str()
            .expect("string detail")
    }

    pub(crate) fn uri(&self) -> &str {
        self.attributes.fetch("uri").as_str().expect("string uri")
    }

    pub(crate) fn range(&self) -> &Value {
        self.attributes.fetch("range")
    }

    pub(crate) fn selection_range(&self) -> &Value {
        self.attributes.fetch("selectionRange")
    }

    pub(crate) fn data(&self) -> &Value {
        self.attributes.fetch("data")
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
