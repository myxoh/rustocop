use serde_json::{Map, Value};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Attributes(Map<String, Value>);

impl Attributes {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn required(&mut self, name: &str, value: impl Into<Value>) {
        self.0.insert(name.to_string(), value.into());
    }

    pub(crate) fn optional(&mut self, name: &str, value: Option<impl Into<Value>>) {
        if let Some(value) = value {
            let value = value.into();
            if !value.is_null() && value != Value::Bool(false) {
                self.0.insert(name.to_string(), value);
            }
        }
    }

    pub(crate) fn fetch(&self, name: &str) -> &Value {
        self.0
            .get(name)
            .unwrap_or_else(|| panic!("key not found: {name}"))
    }

    pub(crate) fn as_map(&self) -> &Map<String, Value> {
        &self.0
    }

    pub(crate) fn to_json(&self) -> String {
        serde_json::to_string(&self.0).expect("LSP attributes are JSON values")
    }
}
