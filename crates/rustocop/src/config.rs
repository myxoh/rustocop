use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use regex::Regex;

mod loader;
mod path_policy;
mod selection;
pub(crate) use selection::{CopSelection, RubyVersion, SourceEncoding};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Parallelism {
    Sequential,
    Automatic,
    Fixed(usize),
}

#[derive(Clone, Debug)]
pub(crate) struct RunOptions {
    pub(crate) files: Vec<String>,
    pub(crate) formats: Vec<String>,
    pub(crate) explicit_message_format: Option<String>,
    pub(crate) display_cop_names: Option<bool>,
    pub(crate) extra_details: bool,
    pub(crate) display_style_guide: bool,
    pub(crate) debug: bool,
    pub(crate) stdin_path: Option<String>,
    pub(crate) parallelism: Parallelism,
    pub(crate) rubocop_loaders: Vec<(String, String)>,
    pub(crate) config_path: Option<String>,
    pub(crate) include_non_native_cops: bool,
    pub(crate) non_native_cops: Vec<String>,
    pub(crate) force_exclusion: bool,
    pub(crate) correction_loop: bool,
    pub(crate) inspection: InspectionConfig,
}

#[derive(Clone, Debug)]
pub(crate) struct InspectionConfig {
    pub(crate) autocorrect: AutocorrectMode,
    pub(crate) ignore_disable_comments: bool,
    pub(crate) cops: CopSelection,
    pub(crate) target_ruby_version: RubyVersion,
    pub(crate) source_encoding: SourceEncoding,
    pub(crate) cop_config: Arc<CopConfig>,
    pub(crate) inspected_path: Option<Arc<str>>,
    /// Cops visible to registry-sensitive compatibility APIs. Ordinarily this
    /// is identical to the executed selection. Focused project audits can
    /// execute one cohort while preserving the full cached RuboCop reference's
    /// registry semantics.
    pub(crate) registry_context: Option<Arc<HashSet<String>>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AutocorrectMode {
    None,
    Safe,
    All,
}

impl AutocorrectMode {
    pub(crate) const fn enabled(self) -> bool {
        !matches!(self, Self::None)
    }

    pub(crate) fn enabled_for(self, config: &CopConfig, cop: &str) -> bool {
        match self {
            Self::None => false,
            Self::Safe => {
                config.bool(cop, "Safe").unwrap_or(true)
                    && config.bool(cop, "SafeAutoCorrect").unwrap_or(true)
                    && !matches!(config.value(cop, "AutoCorrect"), Some("false" | "disabled"))
            }
            Self::All => !matches!(config.value(cop, "AutoCorrect"), Some("false" | "disabled")),
        }
    }
}

impl InspectionConfig {
    pub(crate) fn autocorrect_enabled(&self) -> bool {
        self.autocorrect.enabled()
    }

    pub(crate) fn autocorrect_for(&self, cop: &str) -> bool {
        self.autocorrect.enabled_for(&self.cop_config, cop)
    }

    pub(crate) fn registry_cop_enabled(&self, cop: &str) -> bool {
        self.registry_context
            .as_ref()
            .map_or_else(|| self.cop_enabled(cop), |cops| cops.contains(cop))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CopConfig {
    values: HashMap<String, HashMap<String, ConfigValue>>,
    explicit_sections: HashSet<String>,
    explicit_values: HashSet<(String, String)>,
    // Precompiled once per run for the authoring policy API.
    #[allow(dead_code)]
    patterns: HashMap<String, HashMap<String, Vec<Regex>>>,
    path_globs: HashMap<String, HashMap<String, Vec<path_policy::PathGlob>>>,
    root: Option<PathBuf>,
}

const RUBOCOP_DEFAULT_CONFIG: &str =
    include_str!("../../../spec/upstream/rubocop-1.87.0/config/default.yml");

#[derive(Clone, Debug)]
enum ConfigValue {
    Scalar(String),
    List(Vec<String>),
    Map {
        values: HashMap<String, String>,
        symbol_values: HashMap<String, String>,
    },
}

impl CopConfig {
    pub(crate) fn from_source(source: &str) -> Self {
        Self::from_sources([(source, HashSet::new())], None)
    }

    pub(crate) fn from_resolved_source(source: &str, config_path: Option<&str>) -> Self {
        let root = config_path
            .map(PathBuf::from)
            .and_then(|path| path.parent().map(PathBuf::from));
        Self::from_sources([(source, HashSet::new())], root)
    }

    #[cfg(test)]
    pub(crate) fn without_path_policy(mut self) -> Self {
        // RuboCop's cop unit specs investigate their supplied source directly,
        // independent of the default Include/Exclude target discovery rules.
        // The cached unit corpus models that callback contract; CLI and
        // project inspections continue to enforce path policy.
        for (section, entries) in &mut self.values {
            // The upstream spec harness supplies a cop's complete resolved
            // config, including its default Include glob, while investigating
            // synthetic source under arbitrary paths. Explicit Exclude is
            // itself behavior under test and must remain active.
            entries.remove("Include");
            if !self
                .explicit_values
                .contains(&(section.clone(), "Exclude".to_string()))
            {
                entries.remove("Exclude");
            }
        }
        self.path_globs.clear();
        self
    }

    pub(crate) fn from_path(path: &str) -> Result<Self, String> {
        loader::load(path)
    }

    fn from_sources<'a>(
        sources: impl IntoIterator<Item = (&'a str, HashSet<String>)>,
        root: Option<PathBuf>,
    ) -> Self {
        let mut values = Self::parse_values(RUBOCOP_DEFAULT_CONFIG);
        let mut explicit_sections = HashSet::new();
        let mut explicit_values = HashSet::new();
        let mut inherited_merge_keys = HashSet::new();
        for (source, merge_keys) in sources {
            inherited_merge_keys.extend(merge_keys);
            let overrides = Self::parse_values(source);
            explicit_sections.extend(overrides.keys().cloned());
            explicit_values.extend(
                overrides.iter().flat_map(|(cop, entries)| {
                    entries.keys().map(|key| (cop.clone(), key.clone()))
                }),
            );
            Self::merge_values(&mut values, overrides, &inherited_merge_keys);
        }
        let patterns = Self::compile_patterns(&values);
        let path_globs = path_policy::compile_path_globs(&values);
        let metadata_root = values
            .get("Rustocop")
            .and_then(|entries| entries.get("ProjectRoot"))
            .and_then(|value| match value {
                ConfigValue::Scalar(value) => Some(PathBuf::from(value)),
                ConfigValue::List(_) | ConfigValue::Map { .. } => None,
            });
        let root = metadata_root.map_or(root.clone(), |metadata_root| {
            if metadata_root.is_absolute() {
                Some(metadata_root)
            } else {
                Some(root.unwrap_or_default().join(metadata_root))
            }
        });
        Self {
            values,
            explicit_sections,
            explicit_values,
            patterns,
            path_globs,
            root,
        }
    }

    fn merge_values(
        base: &mut HashMap<String, HashMap<String, ConfigValue>>,
        overrides: HashMap<String, HashMap<String, ConfigValue>>,
        merge_keys: &HashSet<String>,
    ) {
        for (cop, entries) in overrides {
            let percent_literal_delimiters = cop == "Style/PercentLiteralDelimiters";
            let target = base.entry(cop).or_default();
            for (key, value) in entries {
                match (target.get_mut(&key), value) {
                    (
                        Some(ConfigValue::Map {
                            values,
                            symbol_values,
                        }),
                        ConfigValue::Map {
                            values: override_values,
                            symbol_values: override_symbols,
                        },
                    ) => {
                        if percent_literal_delimiters
                            && key == "PreferredDelimiters"
                            && override_values.contains_key("default")
                        {
                            values.clear();
                        }
                        values.extend(override_values);
                        symbol_values.extend(override_symbols);
                    }
                    (Some(ConfigValue::List(values)), ConfigValue::List(override_values))
                        if merge_keys.contains(&key) =>
                    {
                        for value in override_values {
                            if !values.contains(&value) {
                                values.push(value);
                            }
                        }
                    }
                    (_, value) => {
                        target.insert(key, value);
                    }
                }
            }
        }
    }

    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    fn parse_values(source: &str) -> HashMap<String, HashMap<String, ConfigValue>> {
        let mut values = HashMap::<String, HashMap<String, ConfigValue>>::new();
        let mut section = None;
        let mut container_key: Option<String> = None;
        let mut nested_key: Option<String> = None;
        let source_lines = source.lines().collect::<Vec<_>>();
        let mut line_index = 0;
        while line_index < source_lines.len() {
            let line = source_lines[line_index];
            line_index += 1;
            if !line.starts_with(char::is_whitespace) {
                section = config_section(line);
                if let Some(section) = section.as_ref() {
                    values.entry(section.clone()).or_default();
                }
                container_key = None;
                nested_key = None;
                continue;
            }
            let Some(section) = section.as_ref() else {
                continue;
            };
            let trimmed = line.trim();
            let indentation = line.len() - line.trim_start().len();
            if let Some(item) = trimmed.strip_prefix("- ") {
                if let Some(key) = container_key.as_ref() {
                    if let Some(nested) = nested_key.as_ref() {
                        let item = clean_config_scalar(item);
                        let entry = values
                            .entry(section.clone())
                            .or_default()
                            .entry(key.clone())
                            .or_insert_with(|| ConfigValue::Map {
                                values: HashMap::new(),
                                symbol_values: HashMap::new(),
                            });
                        if let ConfigValue::Map { values, .. } = entry {
                            let group = values.entry(nested.clone()).or_default();
                            if !group.is_empty() {
                                group.push('\n');
                            }
                            group.push_str(&item);
                            continue;
                        }
                    }
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
                if trimmed.starts_with("options:") {
                    continue;
                }
                if indentation > 4 {
                    if let (Some(container), Some(nested), Some((key, value, _))) = (
                        container_key.as_ref(),
                        nested_key.as_ref(),
                        nested_config_pair(trimmed),
                    ) {
                        let entry = values
                            .entry(section.clone())
                            .or_default()
                            .entry(container.clone())
                            .or_insert_with(|| ConfigValue::Map {
                                values: HashMap::new(),
                                symbol_values: HashMap::new(),
                            });
                        if let ConfigValue::Map { values, .. } = entry {
                            let encoded = values.entry(nested.clone()).or_default();
                            if !encoded.is_empty() {
                                encoded.push('\n');
                            }
                            encoded.push_str(&clean_config_scalar(key));
                            encoded.push('=');
                            encoded.push_str(&clean_config_scalar(value));
                        }
                        continue;
                    }
                }
                if let (Some(container), Some((key, value, symbols))) =
                    (container_key.as_ref(), nested_config_pair(trimmed))
                {
                    nested_key = Some(clean_config_scalar(key));
                    insert_config_map_entry(&mut values, section, container, key, value, symbols);
                }
                continue;
            }
            let Some((key, value)) = line.trim().split_once(':') else {
                continue;
            };
            let value = config_scalar_source(value);
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
                nested_key = None;
                ConfigValue::List(Vec::new())
            } else if value.starts_with('[') && value.ends_with(']') {
                container_key = None;
                nested_key = None;
                ConfigValue::List(
                    value[1..value.len() - 1]
                        .split(',')
                        .map(clean_config_scalar)
                        .filter(|item| !item.is_empty())
                        .collect(),
                )
            } else {
                container_key = None;
                nested_key = None;
                ConfigValue::Scalar(clean_config_scalar(value))
            };
            values
                .entry(section.clone())
                .or_default()
                .insert(key.to_string(), parsed);
        }
        values
    }

    fn compile_patterns(
        values: &HashMap<String, HashMap<String, ConfigValue>>,
    ) -> HashMap<String, HashMap<String, Vec<Regex>>> {
        values
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
                                .filter_map(|pattern| {
                                    let pattern = pattern
                                        .replace("\\\\", "\\")
                                        .replace("\\A", "^")
                                        .replace("\\z", "$");
                                    Regex::new(&pattern).ok()
                                })
                                .collect(),
                        ))
                    })
                    .collect();
                (cop.clone(), entries)
            })
            .collect()
    }

    pub(crate) fn value(&self, cop: &str, key: &str) -> Option<&str> {
        match self.values.get(cop)?.get(key)? {
            ConfigValue::Scalar(value) => Some(value),
            ConfigValue::List(_) | ConfigValue::Map { .. } => None,
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

    pub(crate) fn contains(&self, cop: &str, key: &str) -> bool {
        self.values
            .get(cop)
            .is_some_and(|values| values.contains_key(key))
    }

    pub(crate) fn explicitly_contains(&self, cop: &str, key: &str) -> bool {
        self.explicit_values
            .contains(&(cop.to_string(), key.to_string()))
    }

    pub(crate) fn explicitly_configures(&self, cop: &str) -> bool {
        self.explicit_sections.contains(cop)
    }

    pub(crate) fn is_compiled(&self) -> bool {
        self.value("Rustocop", "SchemaVersion").is_some()
    }

    pub(crate) fn non_native_cops(&self) -> &[String] {
        self.values("Rustocop", "NonNativeCops")
    }

    /// Whether RuboCop would enable this cop without an explicit `--only`
    /// selection. Directive-aware cops need both this and the command-line
    /// selection state to reproduce Registry#enabled?.
    pub(crate) fn normally_enables(&self, cop: &str) -> bool {
        selection::normally_enabled(cop, self)
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
            ConfigValue::Map { values, .. } => Some(values),
            ConfigValue::Scalar(_) | ConfigValue::List(_) => None,
        }
    }

    pub(crate) fn symbol_map(&self, cop: &str, key: &str) -> Option<&HashMap<String, String>> {
        match self.values.get(cop)?.get(key)? {
            ConfigValue::Map { symbol_values, .. } => Some(symbol_values),
            ConfigValue::Scalar(_) | ConfigValue::List(_) => None,
        }
    }
}

impl Default for CopConfig {
    fn default() -> Self {
        Self::from_source("")
    }
}

fn insert_config_map_entry(
    config: &mut HashMap<String, HashMap<String, ConfigValue>>,
    section: &str,
    container: &str,
    key: &str,
    value: &str,
    symbols: bool,
) {
    let entry = config
        .entry(section.to_string())
        .or_default()
        .entry(container.to_string())
        .or_insert_with(|| ConfigValue::Map {
            values: HashMap::new(),
            symbol_values: HashMap::new(),
        });
    if !matches!(entry, ConfigValue::Map { .. }) {
        *entry = ConfigValue::Map {
            values: HashMap::new(),
            symbol_values: HashMap::new(),
        };
    }
    if let ConfigValue::Map {
        values,
        symbol_values,
    } = entry
    {
        let key = clean_config_scalar(key);
        let value = clean_config_scalar(value);
        values.insert(key.clone(), value.clone());
        if symbols {
            symbol_values.insert(key, value);
        }
    }
}

fn nested_config_pair(line: &str) -> Option<(&str, &str, bool)> {
    if let Some(symbol_pair) = line.strip_prefix(':') {
        let (key, value) = symbol_pair.split_once(": ")?;
        return Some((key, value.strip_prefix(':')?, true));
    }
    if let Some(key) = line.strip_suffix(':') {
        return Some((key, "", false));
    }
    let (key, value) = line.split_once(": ").or_else(|| line.split_once(':'))?;
    Some((key, value, false))
}

fn config_scalar_source(value: &str) -> &str {
    let value = value.trim();
    if value.starts_with(['\'', '"']) {
        value
    } else {
        value.split('#').next().unwrap_or_default().trim()
    }
}

fn config_section(line: &str) -> Option<String> {
    line.strip_suffix(':')
        .filter(|name| {
            name.contains('/')
                || *name == "AllCops"
                || name.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
        })
        .map(str::to_string)
}

fn clean_config_scalar(value: &str) -> String {
    let value = value.trim();
    if value.starts_with('"') && value.ends_with('"') {
        return serde_json::from_str(value).unwrap_or_else(|_| value.trim_matches('"').to_string());
    }
    if let Some(value) = value
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
    {
        return value.replace("''", "'");
    }
    value.to_string()
}

impl InspectionConfig {
    pub(crate) fn cop_enabled(&self, cop: &str) -> bool {
        self.cops.enabled(cop, &self.cop_config)
            && self
                .inspected_path
                .as_deref()
                .is_none_or(|path| self.cop_config.cop_applies_to_path(cop, path))
    }

    pub(crate) fn scoped_to_path(&self, path: &str) -> Self {
        let mut scoped = self.clone();
        scoped.inspected_path = Some(Arc::from(path));
        scoped
    }
}

#[cfg(test)]
#[path = "engine/config_tests.rs"]
mod tests;
