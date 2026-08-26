use regex::Regex;

use super::allowed_pattern::{AllowedPattern, PatternValue};

#[test]
fn current_allowed_and_ignored_patterns_are_always_combined() {
    let policy = AllowedPattern::new(
        vec![PatternValue::Source("allowed".into())],
        vec![PatternValue::Source("legacy".into())],
        vec![],
        vec![],
    );
    assert!(policy.allowed_line("an allowed line"));
    assert!(policy.ignored_line("a legacy line"));
    assert_eq!(policy.cop_config_patterns_values().len(), 2);
}

#[test]
fn deprecated_method_configuration_is_merged_only_when_it_contains_a_regexp() {
    let string_only = AllowedPattern::new(
        vec![],
        vec![],
        vec![PatternValue::Source("ignored".into())],
        vec![],
    );
    assert!(!string_only.matches_allowed_pattern("ignored"));

    let regexp = AllowedPattern::new(
        vec![],
        vec![],
        vec![PatternValue::Regexp(Regex::new("ignored").unwrap())],
        vec![PatternValue::Source("also".into())],
    );
    assert!(regexp.matches_ignored_pattern("ignored"));
    assert!(regexp.matches_allowed_pattern("also"));
    assert_eq!(regexp.cop_config_deprecated_methods_values().len(), 2);
}
