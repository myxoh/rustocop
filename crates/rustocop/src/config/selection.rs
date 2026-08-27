use super::CopConfig;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SourceEncoding {
    Utf8,
    UsAscii,
}

impl SourceEncoding {
    #[cfg(test)]
    pub(crate) fn parse(value: &str) -> Self {
        if value.eq_ignore_ascii_case("US-ASCII") {
            Self::UsAscii
        } else {
            Self::Utf8
        }
    }
}

// Extension cops are not present in RuboCop's built-in configuration. Preserve
// their established defaults until extension configuration is loaded directly.
const DEFAULT_DISABLED_EXTENSION_COPS: &[&str] = &[
    "RSpec/MessageChain",
    "RSpec/MultipleExpectations",
    "RSpec/MultipleMemoizedHelpers",
    "RSpec/PendingWithoutReason",
];

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

    pub(crate) fn as_f64(self) -> f64 {
        f64::from(self.major) + f64::from(self.minor) / 10.0
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
    excluded: Vec<String>,
}

impl CopSelection {
    pub(crate) fn default_enabled() -> Self {
        Self::from_only(None)
    }

    #[cfg(test)]
    pub(crate) fn only(value: &str) -> Self {
        let requested: Vec<_> = value.split(',').map(str::trim).collect();
        Self::from_only(Some(&requested))
    }

    pub(crate) fn select_only(&mut self, value: &str) {
        self.requested = Some(
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect(),
        );
    }

    pub(crate) fn except(&mut self, value: &str) {
        self.excluded.extend(
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
        );
    }

    pub(crate) fn enabled(&self, cop: &str, config: &CopConfig) -> bool {
        if self
            .excluded
            .iter()
            .any(|selection| selection_matches(selection, cop))
        {
            return false;
        }
        match &self.requested {
            None => normally_enabled(cop, config),
            Some(requested) => requested.iter().any(|selection| {
                selection == cop
                    || (normally_enabled(cop, config) && selection_matches(selection, cop))
            }),
        }
    }

    pub(crate) fn requested(&self) -> Option<&[String]> {
        self.requested.as_deref()
    }

    fn from_only(requested: Option<&[&str]>) -> Self {
        Self {
            requested: requested
                .map(|values| values.iter().map(|value| (*value).to_string()).collect()),
            excluded: Vec::new(),
        }
    }
}

fn selection_matches(selection: &str, cop: &str) -> bool {
    selection == cop
        || cop
            .strip_prefix(selection)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

pub(super) fn normally_enabled(cop: &str, config: &CopConfig) -> bool {
    if config.explicitly_contains(cop, "Enabled") {
        return config.bool(cop, "Enabled").unwrap_or(false);
    }

    if config.bool("AllCops", "DisabledByDefault") == Some(true) {
        return config.explicitly_configures(cop);
    }

    if config.bool("AllCops", "EnabledByDefault") == Some(true) {
        return true;
    }

    if let Some((department, _)) = cop.split_once('/') {
        if config.explicitly_contains(department, "Enabled")
            && config.bool(department, "Enabled") != Some(true)
        {
            return false;
        }
    }

    match config.value(cop, "Enabled") {
        Some("false") => false,
        Some("pending") => config.value("AllCops", "NewCops") == Some("enable"),
        _ => !DEFAULT_DISABLED_EXTENSION_COPS.contains(&cop),
    }
}
