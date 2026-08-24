use super::CopConfig;

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

    pub(crate) fn enabled(&self, cop: &str, config: &CopConfig) -> bool {
        match &self.requested {
            None => normally_enabled(cop, config),
            Some(requested) => requested.iter().any(|selection| {
                selection == cop
                    || (normally_enabled(cop, config)
                        && cop
                            .strip_prefix(selection)
                            .is_some_and(|suffix| suffix.starts_with('/')))
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
        }
    }
}

fn normally_enabled(cop: &str, config: &CopConfig) -> bool {
    if config.explicitly_contains(cop, "Enabled") {
        return config.bool(cop, "Enabled").unwrap_or(false);
    }

    if config.bool("AllCops", "DisabledByDefault") == Some(true) {
        return config.explicitly_configures(cop);
    }

    if config.bool("AllCops", "EnabledByDefault") == Some(true) {
        return true;
    }

    match config.value(cop, "Enabled") {
        Some("false") => false,
        Some("pending") => config.value("AllCops", "NewCops") == Some("enable"),
        _ => !DEFAULT_DISABLED_EXTENSION_COPS.contains(&cop),
    }
}
