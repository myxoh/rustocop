// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/allowed_identifiers.rb
// Source SHA-256: 1d9f31e92ac62ddda4824eb7bae0e5f129ad5f8a7ae4bec7b93bee8564aeb366
// Source: lib/rubocop/cop/mixin/allowed_methods.rb
// Source SHA-256: 9714485347fa9559538fa65617d28c7745f9273abff96057c5bb13e77acc8cf2
// Source: lib/rubocop/cop/mixin/allowed_pattern.rb
// Source SHA-256: 6acd0681206c4fda1b11401e5fb57b1cc045b4c4bb3ebb31eb3520e28e0d5238
// Source: lib/rubocop/cop/mixin/forbidden_identifiers.rb
// Source SHA-256: 795e070a8faa1aa57afcc4968740befa9dc5f9b38912ca8d8c7789cdfe5e1d7b
// Source: lib/rubocop/cop/mixin/forbidden_pattern.rb
// Source SHA-256: f93818a71d03eb854b495caa451f786b054eb496592e23c83d9d328fd38a50df
// Source: lib/rubocop/cop/mixin/min_body_length.rb
// Source SHA-256: 05e6bbd1ffb8c637bc1847b2b5b594e68ce7538f5ae80ad7b6f1b5352bb6b1bd
// Source: lib/rubocop/cop/mixin/min_branches_count.rb
// Source SHA-256: 03cebbe1604cf6dfc231e6263ce567b44266261050e601460db928038d431423
// Source: lib/rubocop/cop/mixin/preferred_delimiters.rb
// Source SHA-256: 87ef71d90fd6bba41e0816d4b77bbb29c4f38e27523df86801cb135dc8eb6204
// Source: lib/rubocop/cop/mixin/target_ruby_version.rb
// Source SHA-256: b9348665203dd8bb551a46de8e057084666dfa3dacd4b91063835635d8221536

use std::collections::HashMap;

use regex::Regex;

use crate::rubocop::ast::node::core::NodeRef;

#[derive(Clone, Debug)]
pub(crate) enum ConfiguredName {
    Literal(String),
    Pattern(String),
}

pub(crate) fn allowed_identifier(name: &str, allowed_identifiers: &[String]) -> bool {
    !allowed_identifiers.is_empty()
        && allowed_identifiers
            .iter()
            .any(|allowed| allowed == &without_sigils(name))
}

pub(crate) fn forbidden_identifier(name: &str, forbidden_identifiers: &[String]) -> bool {
    !forbidden_identifiers.is_empty()
        && forbidden_identifiers
            .iter()
            .any(|forbidden| forbidden == &without_sigils(name))
}

fn without_sigils(name: &str) -> String {
    name.chars()
        .filter(|character| !matches!(character, '@' | '$'))
        .collect()
}

pub(crate) fn allowed_methods(configured: &[String], deprecated: &[ConfiguredName]) -> Vec<String> {
    let mut methods = configured.to_vec();
    if !deprecated
        .iter()
        .any(|value| matches!(value, ConfiguredName::Pattern(_)))
    {
        methods.extend(deprecated.iter().filter_map(|value| match value {
            ConfiguredName::Literal(name) => Some(name.clone()),
            ConfiguredName::Pattern(_) => None,
        }));
    }
    methods
}

pub(crate) fn allowed_method(name: &str, methods: &[String]) -> bool {
    methods.iter().any(|method| method == name)
}

pub(crate) fn allowed_patterns(
    configured: &[String],
    ignored: &[String],
    deprecated: &[ConfiguredName],
) -> Vec<String> {
    let mut patterns = configured.to_vec();
    patterns.extend_from_slice(ignored);
    if deprecated
        .iter()
        .any(|value| matches!(value, ConfiguredName::Pattern(_)))
    {
        patterns.extend(deprecated.iter().map(|value| match value {
            ConfiguredName::Literal(name) | ConfiguredName::Pattern(name) => name.clone(),
        }));
    }
    patterns
}

pub(crate) fn matches_allowed_pattern(line: &str, patterns: &[String]) -> bool {
    patterns
        .iter()
        .any(|pattern| Regex::new(pattern).is_ok_and(|regexp| regexp.is_match(line)))
}

pub(crate) fn forbidden_pattern(name: &str, patterns: &[String]) -> bool {
    patterns
        .iter()
        .any(|pattern| Regex::new(pattern).is_ok_and(|regexp| regexp.is_match(name)))
}

pub(crate) fn min_body_length(configured: Option<i64>) -> Result<usize, &'static str> {
    let length = configured.unwrap_or(1);
    if length > 0 {
        Ok(length as usize)
    } else {
        Err("MinBodyLength needs to be a positive integer!")
    }
}

pub(crate) fn meets_min_body_length(keyword_line: usize, end_line: usize, minimum: usize) -> bool {
    end_line.saturating_sub(keyword_line) > minimum
}

pub(crate) fn min_branches_count(configured: Option<i64>) -> Result<usize, &'static str> {
    let length = configured.unwrap_or(3);
    if length > 0 {
        Ok(length as usize)
    } else {
        Err("MinBranchesCount needs to be a positive integer!")
    }
}

pub(crate) fn meets_min_branches_count(branch_count: usize, minimum: usize) -> bool {
    branch_count >= minimum
}

pub(crate) fn if_conditional_branches<'ast>(mut node: NodeRef<'ast>) -> Vec<NodeRef<'ast>> {
    let mut branches = Vec::new();
    loop {
        if node.kind() != "if" {
            break;
        }
        if let Some(branch) = node.if_branch() {
            branches.push(branch);
        }
        let Some(next) = node.else_branch().filter(|branch| branch.kind() == "if") else {
            break;
        };
        node = next;
    }
    branches
}

#[derive(Debug)]
pub(crate) struct PreferredDelimiters {
    kind: String,
    delimiters: HashMap<String, String>,
}

impl PreferredDelimiters {
    pub(crate) const PERCENT_LITERAL_TYPES: [&'static str; 10] =
        ["%", "%i", "%I", "%q", "%Q", "%r", "%s", "%w", "%W", "%x"];

    pub(crate) fn new(
        kind: impl Into<String>,
        configured: HashMap<String, String>,
    ) -> Result<Self, String> {
        let invalid: Vec<_> = configured
            .keys()
            .filter(|key| {
                key.as_str() != "default" && !Self::PERCENT_LITERAL_TYPES.contains(&key.as_str())
            })
            .cloned()
            .collect();
        if !invalid.is_empty() {
            return Err(format!(
                "Invalid preferred delimiter config key: {}",
                invalid.join(", ")
            ));
        }
        let delimiters = if let Some(default) = configured.get("default") {
            Self::PERCENT_LITERAL_TYPES
                .iter()
                .map(|kind| {
                    (
                        (*kind).to_owned(),
                        configured.get(*kind).unwrap_or(default).clone(),
                    )
                })
                .collect()
        } else {
            configured
        };
        Ok(Self {
            kind: kind.into(),
            delimiters,
        })
    }

    pub(crate) fn delimiters(&self) -> Vec<char> {
        self.delimiters
            .get(&self.kind)
            .map(|value| value.chars().collect())
            .unwrap_or_default()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct TargetRubyVersion {
    minimum: Option<f64>,
    maximum: Option<f64>,
}

impl TargetRubyVersion {
    pub(crate) fn minimum_target_ruby_version(&mut self, version: f64) {
        self.minimum = Some(version);
    }

    pub(crate) fn maximum_target_ruby_version(&mut self, version: f64) {
        self.maximum = Some(version);
    }

    pub(crate) fn required_minimum_ruby_version(self) -> Option<f64> {
        self.minimum
    }

    pub(crate) fn required_maximum_ruby_version(self) -> Option<f64> {
        self.maximum
    }

    pub(crate) fn supports(self, version: f64) -> bool {
        version >= self.minimum.unwrap_or(0.0) && version <= self.maximum.unwrap_or(f64::INFINITY)
    }
}
