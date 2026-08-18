use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::catalog::{DEFAULT_DISABLED_COPS, SUPPORTED_COPS};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Parallelism {
    Sequential,
    Automatic,
    Fixed(usize),
}

#[derive(Clone, Debug)]
pub(crate) struct RunOptions {
    pub(crate) files: Vec<String>,
    pub(crate) format: String,
    pub(crate) stdin_path: Option<String>,
    pub(crate) parallelism: Parallelism,
    pub(crate) inspection: InspectionConfig,
}

#[derive(Clone, Debug)]
pub(crate) struct InspectionConfig {
    pub(crate) autocorrect: bool,
    pub(crate) cops: CopSelection,
    pub(crate) target_ruby_version: RubyVersion,
    pub(crate) cop_config: Arc<CopConfig>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct CopConfig {
    values: HashMap<String, HashMap<String, String>>,
}

impl CopConfig {
    pub(crate) fn from_source(source: &str) -> Self {
        let mut values = HashMap::<String, HashMap<String, String>>::new();
        let mut section = None;
        for line in source.lines() {
            if !line.starts_with(char::is_whitespace) {
                section = line
                    .strip_suffix(':')
                    .filter(|name| name.contains('/'))
                    .map(str::to_string);
                continue;
            }
            let Some(section) = section.as_ref() else {
                continue;
            };
            let Some((key, value)) = line.trim().split_once(':') else {
                continue;
            };
            let value = value
                .split('#')
                .next()
                .unwrap_or_default()
                .trim()
                .trim_matches(['\'', '"']);
            values
                .entry(section.clone())
                .or_default()
                .insert(key.to_string(), value.to_string());
        }
        Self { values }
    }

    pub(crate) fn value(&self, cop: &str, key: &str) -> Option<&str> {
        self.values.get(cop)?.get(key).map(String::as_str)
    }
}

impl InspectionConfig {
    pub(crate) fn cop_enabled(&self, cop: &str) -> bool {
        self.cops.enabled(cop)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RubyVersion {
    major: u16,
    minor: u16,
}

impl RubyVersion {
    pub(crate) fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        let mut parts = value.trim_matches(['\'', '"']).split('.');
        Some(Self::new(
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
        ))
    }

    pub(crate) fn at_least(self, major: u16, minor: u16) -> bool {
        (self.major, self.minor) >= (major, minor)
    }
}

impl Default for RubyVersion {
    fn default() -> Self {
        Self::new(2, 7)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CopSelection {
    enabled: HashSet<&'static str>,
}

impl CopSelection {
    pub(crate) fn default_enabled() -> Self {
        Self::from_only(None)
    }

    pub(crate) fn only(value: &str) -> Self {
        let requested: Vec<_> = value.split(',').map(str::trim).collect();
        Self::from_only(Some(&requested))
    }

    pub(crate) fn enabled(&self, cop: &str) -> bool {
        self.enabled.contains(cop)
    }

    fn from_only(requested: Option<&[&str]>) -> Self {
        let enabled = SUPPORTED_COPS
            .iter()
            .copied()
            .filter(|cop| match requested {
                None => !DEFAULT_DISABLED_COPS.contains(cop),
                Some(requested) => requested.iter().any(|selection| {
                    selection == cop
                        || (!DEFAULT_DISABLED_COPS.contains(cop)
                            && cop
                                .strip_prefix(selection)
                                .is_some_and(|suffix| suffix.starts_with('/')))
                }),
            })
            .collect();
        Self { enabled }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_departments_without_enabling_default_disabled_cops() {
        let selection = CopSelection::only("Style");
        assert!(selection.enabled("Style/HashSyntax"));
        assert!(!selection.enabled("Style/Documentation"));
    }

    #[test]
    fn explicitly_enables_default_disabled_cops() {
        let selection = CopSelection::only("Style/Documentation");
        assert!(selection.enabled("Style/Documentation"));
        assert!(!selection.enabled("Style/HashSyntax"));
    }

    #[test]
    fn reads_scoped_cop_configuration_values() {
        let config = CopConfig::from_source(
            "Style/Example:\n  EnforcedStyle: custom\nOther/Rule:\n  Allowed: false\n",
        );

        assert_eq!(
            config.value("Style/Example", "EnforcedStyle"),
            Some("custom")
        );
        assert_eq!(config.value("Other/Rule", "Allowed"), Some("false"));
        assert_eq!(config.value("Style/Example", "Allowed"), None);
    }
}
