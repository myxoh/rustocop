use std::collections::HashSet;

use crate::cop_registry::{DEFAULT_DISABLED_COPS, SUPPORTED_COPS};

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
}
