const DEFAULT_DISABLED_COPS: &[&str] = &[
    "RSpec/MessageChain",
    "RSpec/MultipleExpectations",
    "RSpec/MultipleMemoizedHelpers",
    "RSpec/PendingWithoutReason",
    "Security/IoMethods",
    "Style/Copyright",
    "Style/Documentation",
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
