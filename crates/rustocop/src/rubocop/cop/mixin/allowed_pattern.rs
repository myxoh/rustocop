// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/allowed_pattern.rb
// Source SHA-256: 6acd0681206c4fda1b11401e5fb57b1cc045b4c4bb3ebb31eb3520e28e0d5238

use regex::Regex;

#[derive(Clone, Debug)]
pub(crate) enum PatternValue {
    Source(String),
    Regexp(Regex),
}

#[derive(Clone, Debug)]
pub(crate) struct AllowedPattern {
    allowed: Vec<PatternValue>,
    ignored: Vec<PatternValue>,
    ignored_methods: Vec<PatternValue>,
    excluded_methods: Vec<PatternValue>,
}

impl AllowedPattern {
    pub(crate) fn new(
        allowed: Vec<PatternValue>,
        ignored: Vec<PatternValue>,
        ignored_methods: Vec<PatternValue>,
        excluded_methods: Vec<PatternValue>,
    ) -> Self {
        Self {
            allowed,
            ignored,
            ignored_methods,
            excluded_methods,
        }
    }

    pub(crate) fn allowed_line(&self, line: &str) -> bool {
        self.matches_allowed_pattern(line)
    }

    pub(crate) fn ignored_line(&self, line: &str) -> bool {
        self.allowed_line(line)
    }

    pub(crate) fn matches_allowed_pattern(&self, line: &str) -> bool {
        self.allowed_patterns().iter().any(|pattern| match pattern {
            PatternValue::Source(source) => {
                Regex::new(source).is_ok_and(|regexp| regexp.is_match(line))
            }
            PatternValue::Regexp(regexp) => regexp.is_match(line),
        })
    }

    pub(crate) fn matches_ignored_pattern(&self, line: &str) -> bool {
        self.matches_allowed_pattern(line)
    }

    pub(crate) fn allowed_patterns(&self) -> Vec<&PatternValue> {
        let current = self.cop_config_patterns_values();
        let deprecated = self.cop_config_deprecated_methods_values();
        if deprecated
            .iter()
            .any(|value| matches!(value, PatternValue::Regexp(_)))
        {
            current.into_iter().chain(deprecated).collect()
        } else {
            current
        }
    }

    pub(crate) fn cop_config_patterns_values(&self) -> Vec<&PatternValue> {
        self.allowed.iter().chain(&self.ignored).collect()
    }

    pub(crate) fn cop_config_deprecated_methods_values(&self) -> Vec<&PatternValue> {
        self.ignored_methods
            .iter()
            .chain(&self.excluded_methods)
            .collect()
    }
}
