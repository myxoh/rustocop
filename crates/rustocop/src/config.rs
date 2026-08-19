use std::collections::HashMap;
use std::sync::Arc;

use regex::Regex;

use crate::catalog::DEFAULT_DISABLED_COPS;

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
    values: HashMap<String, HashMap<String, ConfigValue>>,
    // Precompiled once per run for the authoring policy API.
    #[allow(dead_code)]
    patterns: HashMap<String, HashMap<String, Vec<Regex>>>,
}

#[derive(Clone, Debug)]
enum ConfigValue {
    Scalar(String),
    List(Vec<String>),
    Map(HashMap<String, String>),
}

impl CopConfig {
    pub(crate) fn from_source(source: &str) -> Self {
        let mut values = HashMap::<String, HashMap<String, ConfigValue>>::new();
        let mut section = None;
        let mut container_key: Option<String> = None;
        let source_lines = source.lines().collect::<Vec<_>>();
        let mut line_index = 0;
        while line_index < source_lines.len() {
            let line = source_lines[line_index];
            line_index += 1;
            if !line.starts_with(char::is_whitespace) {
                section = line
                    .strip_suffix(':')
                    .filter(|name| name.contains('/') || *name == "AllCops")
                    .map(str::to_string);
                container_key = None;
                continue;
            }
            let Some(section) = section.as_ref() else {
                continue;
            };
            let trimmed = line.trim();
            let indentation = line.len() - line.trim_start().len();
            if let Some(item) = trimmed.strip_prefix("- ") {
                if let Some(key) = container_key.as_ref() {
                    let entry = values
                        .entry(section.clone())
                        .or_default()
                        .entry(key.clone())
                        .or_insert_with(|| ConfigValue::List(Vec::new()));
                    if !matches!(entry, ConfigValue::List(_)) {
                        *entry = ConfigValue::List(Vec::new());
                    }
                    if let ConfigValue::List(items) = entry {
                        let item = item
                            .split_once(':')
                            .filter(|(nested_key, _)| clean_config_scalar(nested_key) == "$regexp")
                            .map_or_else(
                                || clean_config_scalar(item),
                                |(_, pattern)| clean_config_scalar(pattern),
                            );
                        items.push(item);
                    }
                }
                continue;
            }
            if indentation > 2 {
                if container_key
                    .as_ref()
                    .is_some_and(|key| key.contains("Pattern") && trimmed.starts_with("options:"))
                {
                    continue;
                }
                if let (Some(container), Some((key, value))) =
                    (container_key.as_ref(), trimmed.split_once(':'))
                {
                    let entry = values
                        .entry(section.clone())
                        .or_default()
                        .entry(container.clone())
                        .or_insert_with(|| ConfigValue::Map(HashMap::new()));
                    if !matches!(entry, ConfigValue::Map(_)) {
                        *entry = ConfigValue::Map(HashMap::new());
                    }
                    if let ConfigValue::Map(entries) = entry {
                        entries.insert(clean_config_scalar(key), clean_config_scalar(value));
                    }
                }
                continue;
            }
            let Some((key, value)) = line.trim().split_once(':') else {
                continue;
            };
            let raw_value = value.trim();
            let value = if raw_value.starts_with(['\'', '"']) {
                raw_value
            } else {
                raw_value.split('#').next().unwrap_or_default().trim()
            };
            if value == "|" || value == ">" {
                let base_indentation = indentation;
                let mut block = Vec::new();
                while line_index < source_lines.len() {
                    let candidate = source_lines[line_index];
                    let candidate_indentation = candidate.len() - candidate.trim_start().len();
                    if candidate.trim().is_empty() {
                        block.push(String::new());
                        line_index += 1;
                        continue;
                    }
                    if candidate_indentation <= base_indentation {
                        break;
                    }
                    block.push(candidate.trim_start().to_string());
                    line_index += 1;
                }
                let separator = if value == ">" { " " } else { "\n" };
                let mut scalar = block.join(separator);
                scalar.push('\n');
                container_key = None;
                values
                    .entry(section.clone())
                    .or_default()
                    .insert(key.to_string(), ConfigValue::Scalar(scalar));
                continue;
            }
            let parsed = if value.is_empty() {
                container_key = Some(key.to_string());
                ConfigValue::List(Vec::new())
            } else if value.starts_with('[') && value.ends_with(']') {
                container_key = None;
                ConfigValue::List(
                    value[1..value.len() - 1]
                        .split(',')
                        .map(clean_config_scalar)
                        .filter(|item| !item.is_empty())
                        .collect(),
                )
            } else {
                container_key = None;
                ConfigValue::Scalar(clean_config_scalar(value))
            };
            values
                .entry(section.clone())
                .or_default()
                .insert(key.to_string(), parsed);
        }
        let patterns = values
            .iter()
            .map(|(cop, entries)| {
                let entries = entries
                    .iter()
                    .filter(|(key, _value)| key.contains("Pattern"))
                    .filter_map(|(key, value)| {
                        let ConfigValue::List(patterns) = value else {
                            return None;
                        };
                        Some((
                            key.clone(),
                            patterns
                                .iter()
                                .filter_map(|pattern| Regex::new(pattern).ok())
                                .collect(),
                        ))
                    })
                    .collect();
                (cop.clone(), entries)
            })
            .collect();
        Self { values, patterns }
    }

    pub(crate) fn value(&self, cop: &str, key: &str) -> Option<&str> {
        match self.values.get(cop)?.get(key)? {
            ConfigValue::Scalar(value) => Some(value),
            ConfigValue::List(_) | ConfigValue::Map(_) => None,
        }
    }

    pub(crate) fn bool(&self, cop: &str, key: &str) -> Option<bool> {
        match self.value(cop, key)? {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn usize(&self, cop: &str, key: &str) -> Option<usize> {
        self.value(cop, key)?.parse().ok()
    }

    pub(crate) fn values(&self, cop: &str, key: &str) -> &[String] {
        match self.values.get(cop).and_then(|values| values.get(key)) {
            Some(ConfigValue::List(values)) => values,
            _ => &[],
        }
    }

    #[allow(dead_code)]
    pub(crate) fn patterns(&self, cop: &str, key: &str) -> &[Regex] {
        self.patterns
            .get(cop)
            .and_then(|patterns| patterns.get(key))
            .map_or(&[], Vec::as_slice)
    }

    pub(crate) fn map(&self, cop: &str, key: &str) -> Option<&HashMap<String, String>> {
        match self.values.get(cop)?.get(key)? {
            ConfigValue::Map(values) => Some(values),
            ConfigValue::Scalar(_) | ConfigValue::List(_) => None,
        }
    }
}

fn clean_config_scalar(value: &str) -> String {
    value.trim().trim_matches(['\'', '"']).to_string()
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

    pub(crate) fn major(self) -> u16 {
        self.major
    }

    pub(crate) fn minor(self) -> u16 {
        self.minor
    }
}

impl Default for RubyVersion {
    fn default() -> Self {
        Self::new(2, 7)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CopSelection {
    requested: Option<Vec<String>>,
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
        match &self.requested {
            None => !DEFAULT_DISABLED_COPS.contains(&cop),
            Some(requested) => requested.iter().any(|selection| {
                selection == cop
                    || (!DEFAULT_DISABLED_COPS.contains(&cop)
                        && cop
                            .strip_prefix(selection)
                            .is_some_and(|suffix| suffix.starts_with('/')))
            }),
        }
    }

    fn from_only(requested: Option<&[&str]>) -> Self {
        Self {
            requested: requested
                .map(|values| values.iter().map(|value| (*value).to_string()).collect()),
        }
    }
}

#[cfg(test)]
#[path = "engine/config_tests.rs"]
mod tests;
