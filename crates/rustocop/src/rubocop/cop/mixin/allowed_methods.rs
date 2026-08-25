// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/allowed_methods.rb
// Source SHA-256: 9714485347fa9559538fa65617d28c7745f9273abff96057c5bb13e77acc8cf2

use regex::Regex;

#[derive(Clone, Debug)]
pub(crate) enum ConfiguredMethod {
    Name(String),
    Pattern(Regex),
}

#[derive(Clone, Debug)]
pub(crate) struct AllowedMethods {
    allowed: Vec<String>,
    ignored: Vec<ConfiguredMethod>,
    excluded: Vec<ConfiguredMethod>,
}

impl AllowedMethods {
    pub(crate) fn new(
        allowed: Vec<String>,
        ignored: Vec<ConfiguredMethod>,
        excluded: Vec<ConfiguredMethod>,
    ) -> Self {
        Self {
            allowed,
            ignored,
            excluded,
        }
    }

    pub(crate) fn allowed_method(&self, name: &str) -> bool {
        self.allowed_methods().iter().any(|allowed| allowed == name)
    }

    pub(crate) fn ignored_method(&self, name: &str) -> bool {
        self.allowed_method(name)
    }

    pub(crate) fn allowed_methods(&self) -> Vec<String> {
        let deprecated = self.cop_config_deprecated_values();
        if deprecated
            .iter()
            .any(|value| matches!(value, ConfiguredMethod::Pattern(_)))
        {
            self.cop_config_allowed_methods().to_vec()
        } else {
            self.cop_config_allowed_methods()
                .iter()
                .cloned()
                .chain(deprecated.iter().filter_map(|value| match value {
                    ConfiguredMethod::Name(name) => Some(name.clone()),
                    ConfiguredMethod::Pattern(_) => None,
                }))
                .collect()
        }
    }

    pub(crate) fn cop_config_allowed_methods(&self) -> &[String] {
        &self.allowed
    }

    pub(crate) fn cop_config_deprecated_values(&self) -> Vec<&ConfiguredMethod> {
        self.ignored.iter().chain(&self.excluded).collect()
    }
}
