use super::attributes::Attributes;
use serde_json::{Map, Value};
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CompletionOptions {
    attributes: Attributes,
}
impl CompletionOptions {
    pub(crate) fn new(
        work_done_progress: Option<bool>,
        trigger_characters: Option<Vec<String>>,
        all_commit_characters: Option<Vec<String>>,
        resolve_provider: Option<bool>,
        completion_item: Option<Value>,
    ) -> Self {
        let mut attributes = Attributes::new();
        attributes.optional("workDoneProgress", work_done_progress);
        attributes.optional("triggerCharacters", trigger_characters);
        attributes.optional("allCommitCharacters", all_commit_characters);
        attributes.optional("resolveProvider", resolve_provider);
        attributes.optional("completionItem", completion_item);
        Self { attributes }
    }
    fn boolean(&self, name: &str) -> bool {
        self.attributes
            .fetch(name)
            .as_bool()
            .expect("completion option is a boolean")
    }
    fn strings(&self, name: &str) -> Vec<&str> {
        self.attributes
            .fetch(name)
            .as_array()
            .expect("completion option is a string array")
            .iter()
            .map(|value| value.as_str().expect("completion option item is a string"))
            .collect()
    }
    pub(crate) fn work_done_progress(&self) -> bool {
        self.boolean("workDoneProgress")
    }
    pub(crate) fn trigger_characters(&self) -> Vec<&str> {
        self.strings("triggerCharacters")
    }
    pub(crate) fn all_commit_characters(&self) -> Vec<&str> {
        self.strings("allCommitCharacters")
    }
    pub(crate) fn resolve_provider(&self) -> bool {
        self.boolean("resolveProvider")
    }
    pub(crate) fn completion_item(&self) -> &Value {
        self.attributes.fetch("completionItem")
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
