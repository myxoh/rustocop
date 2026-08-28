use serde_json::{Map, Value};

use super::attributes::Attributes;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Color {
    attributes: Attributes,
}

impl Color {
    pub(crate) fn new(red: f64, green: f64, blue: f64, alpha: f64) -> Self {
        let mut attributes = Attributes::new();
        attributes.required("red", red);
        attributes.required("green", green);
        attributes.required("blue", blue);
        attributes.required("alpha", alpha);
        Self { attributes }
    }

    pub(crate) fn red(&self) -> f64 {
        self.component("red")
    }

    pub(crate) fn green(&self) -> f64 {
        self.component("green")
    }

    pub(crate) fn blue(&self) -> f64 {
        self.component("blue")
    }

    pub(crate) fn alpha(&self) -> f64 {
        self.component("alpha")
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

    fn component(&self, name: &str) -> f64 {
        self.attributes
            .fetch(name)
            .as_f64()
            .expect("color component is numeric")
    }
}
