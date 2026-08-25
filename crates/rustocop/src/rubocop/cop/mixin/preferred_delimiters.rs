// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/preferred_delimiters.rb
// Source SHA-256: 87ef71d90fd6bba41e0816d4b77bbb29c4f38e27523df86801cb135dc8eb6204

use std::collections::BTreeMap;

const PERCENT_LITERAL_TYPES: [&str; 10] =
    ["%", "%i", "%I", "%q", "%Q", "%r", "%s", "%w", "%W", "%x"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PreferredDelimiters {
    type_name: String,
    config: BTreeMap<String, String>,
    supplied: Option<BTreeMap<String, String>>,
}

impl PreferredDelimiters {
    pub(crate) fn type_name(&self) -> &str {
        &self.type_name
    }

    pub(crate) fn config(&self) -> &BTreeMap<String, String> {
        &self.config
    }

    pub(crate) fn initialize(
        type_name: impl Into<String>,
        config: BTreeMap<String, String>,
        preferred_delimiters: Option<BTreeMap<String, String>>,
    ) -> Self {
        Self {
            type_name: type_name.into(),
            config,
            supplied: preferred_delimiters,
        }
    }

    pub(crate) fn delimiters(&self) -> Result<Vec<char>, String> {
        Ok(self
            .preferred_delimiters()?
            .get(&self.type_name)
            .map_or_else(Vec::new, |delimiters| delimiters.chars().collect()))
    }

    pub(crate) fn ensure_valid_preferred_delimiters(&self) -> Result<(), String> {
        let invalid: Vec<_> = self
            .preferred_delimiters_config()
            .keys()
            .filter(|key| {
                key.as_str() != "default" && !PERCENT_LITERAL_TYPES.contains(&key.as_str())
            })
            .cloned()
            .collect();
        if invalid.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "Invalid preferred delimiter config key: {}",
                invalid.join(", ")
            ))
        }
    }

    pub(crate) fn preferred_delimiters(&self) -> Result<BTreeMap<String, String>, String> {
        if let Some(supplied) = &self.supplied {
            return Ok(supplied.clone());
        }
        self.ensure_valid_preferred_delimiters()?;
        let configured = self.preferred_delimiters_config();
        let Some(default) = configured.get("default") else {
            return Ok(configured.clone());
        };
        Ok(PERCENT_LITERAL_TYPES
            .iter()
            .map(|type_name| {
                (
                    (*type_name).into(),
                    configured.get(*type_name).unwrap_or(default).clone(),
                )
            })
            .collect())
    }

    pub(crate) fn preferred_delimiters_config(&self) -> &BTreeMap<String, String> {
        &self.config
    }
}
