// RuboCop 1.87.0
// Source: lib/rubocop/cop/severity.rb
// Source SHA-256: f627759bfebf54473189ac18a10df973b142dde6e21a2d10d4bdeb9eddb942ab

use std::fmt;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum Severity {
    Info,
    Refactor,
    Convention,
    Warning,
    Error,
    Fatal,
}

impl Severity {
    pub(crate) const NAMES: [&'static str; 6] = [
        "info",
        "refactor",
        "convention",
        "warning",
        "error",
        "fatal",
    ];

    pub(crate) fn name(self) -> &'static str {
        Self::NAMES[self as usize]
    }

    pub(crate) fn code(self) -> char {
        self.name().chars().next().unwrap().to_ascii_uppercase()
    }

    pub(crate) fn level(self) -> usize {
        self as usize + 1
    }

    pub(crate) fn name_from_code(name_or_code: &str) -> &str {
        match name_or_code {
            "I" => "info",
            "R" => "refactor",
            "C" => "convention",
            "W" => "warning",
            "E" => "error",
            "F" => "fatal",
            name => name,
        }
    }

    pub(crate) fn equivalent(self, other: Self) -> bool {
        self.name() == other.name()
    }

    pub(crate) fn compare(self, other: Self) -> std::cmp::Ordering {
        self.level().cmp(&other.level())
    }

    pub(crate) fn hash_value(self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.name().hash(&mut hasher);
        hasher.finish()
    }

    pub(crate) fn display(self) -> &'static str {
        self.name()
    }
}

impl FromStr for Severity {
    type Err = String;

    fn from_str(name_or_code: &str) -> Result<Self, Self::Err> {
        match Self::name_from_code(name_or_code) {
            "info" => Ok(Self::Info),
            "refactor" => Ok(Self::Refactor),
            "convention" => Ok(Self::Convention),
            "warning" => Ok(Self::Warning),
            "error" => Ok(Self::Error),
            "fatal" => Ok(Self::Fatal),
            unknown => Err(format!("Unknown severity: {unknown}")),
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}
