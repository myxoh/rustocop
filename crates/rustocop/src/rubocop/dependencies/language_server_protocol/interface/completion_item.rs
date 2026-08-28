use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CompletionItem {
    attributes: Attributes,
}

impl CompletionItem {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        label: impl Into<String>,
        label_details: Option<Value>,
        kind: Option<i64>,
        tags: Option<Vec<i64>>,
        detail: Option<impl Into<String>>,
        documentation: Option<Value>,
        deprecated: Option<bool>,
        preselect: Option<bool>,
        sort_text: Option<impl Into<String>>,
        filter_text: Option<impl Into<String>>,
        insert_text: Option<impl Into<String>>,
        insert_text_format: Option<i64>,
        insert_text_mode: Option<i64>,
        text_edit: Option<Value>,
        text_edit_text: Option<impl Into<String>>,
        additional_text_edits: Option<Vec<Value>>,
        commit_characters: Option<Vec<String>>,
        command: Option<Value>,
        data: Option<Value>,
    ) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("label", label.into());
        attributes.optional("labelDetails", label_details);
        attributes.optional("kind", kind);
        attributes.optional("tags", tags);
        attributes.optional("detail", detail.map(Into::into));
        attributes.optional("documentation", documentation);
        attributes.optional("deprecated", deprecated);
        attributes.optional("preselect", preselect);
        attributes.optional("sortText", sort_text.map(Into::into));
        attributes.optional("filterText", filter_text.map(Into::into));
        attributes.optional("insertText", insert_text.map(Into::into));
        attributes.optional("insertTextFormat", insert_text_format);
        attributes.optional("insertTextMode", insert_text_mode);
        attributes.optional("textEdit", text_edit);
        attributes.optional("textEditText", text_edit_text.map(Into::into));
        attributes.optional("additionalTextEdits", additional_text_edits);
        attributes.optional("commitCharacters", commit_characters);
        attributes.optional("command", command);
        attributes.optional("data", data);
        Self { attributes }
    }

    pub(crate) fn label(&self) -> &str {
        self.string("label")
    }

    pub(crate) fn label_details(&self) -> &Value {
        self.attributes.fetch("labelDetails")
    }

    pub(crate) fn kind(&self) -> i64 {
        self.integer("kind")
    }

    pub(crate) fn tags(&self) -> Vec<i64> {
        self.integer_array("tags")
    }

    pub(crate) fn detail(&self) -> &str {
        self.string("detail")
    }

    pub(crate) fn documentation(&self) -> &Value {
        self.attributes.fetch("documentation")
    }

    pub(crate) fn deprecated(&self) -> bool {
        self.boolean("deprecated")
    }

    pub(crate) fn preselect(&self) -> bool {
        self.boolean("preselect")
    }

    pub(crate) fn sort_text(&self) -> &str {
        self.string("sortText")
    }

    pub(crate) fn filter_text(&self) -> &str {
        self.string("filterText")
    }

    pub(crate) fn insert_text(&self) -> &str {
        self.string("insertText")
    }

    pub(crate) fn insert_text_format(&self) -> i64 {
        self.integer("insertTextFormat")
    }

    pub(crate) fn insert_text_mode(&self) -> i64 {
        self.integer("insertTextMode")
    }

    pub(crate) fn text_edit(&self) -> &Value {
        self.attributes.fetch("textEdit")
    }

    pub(crate) fn text_edit_text(&self) -> &str {
        self.string("textEditText")
    }

    pub(crate) fn additional_text_edits(&self) -> &[Value] {
        self.attributes
            .fetch("additionalTextEdits")
            .as_array()
            .expect("additionalTextEdits is an array")
    }

    pub(crate) fn commit_characters(&self) -> Vec<&str> {
        self.attributes
            .fetch("commitCharacters")
            .as_array()
            .expect("commitCharacters is an array")
            .iter()
            .map(|character| character.as_str().expect("commit character is a string"))
            .collect()
    }

    pub(crate) fn command(&self) -> &Value {
        self.attributes.fetch("command")
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

    fn string(&self, name: &str) -> &str {
        self.attributes.fetch(name).as_str().expect("string field")
    }

    fn integer(&self, name: &str) -> i64 {
        self.attributes.fetch(name).as_i64().expect("integer field")
    }

    fn boolean(&self, name: &str) -> bool {
        self.attributes
            .fetch(name)
            .as_bool()
            .expect("boolean field")
    }

    fn integer_array(&self, name: &str) -> Vec<i64> {
        self.attributes
            .fetch(name)
            .as_array()
            .expect("integer array field")
            .iter()
            .map(|item| item.as_i64().expect("integer array item"))
            .collect()
    }
}
