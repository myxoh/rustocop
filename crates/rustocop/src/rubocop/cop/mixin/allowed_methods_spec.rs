use regex::Regex;

use super::allowed_methods::{AllowedMethods, ConfiguredMethod};

#[test]
fn deprecated_string_lists_are_appended_in_ignored_then_excluded_order() {
    let policy = AllowedMethods::new(
        vec!["allowed".into()],
        vec![ConfiguredMethod::Name("ignored".into())],
        vec![ConfiguredMethod::Name("excluded".into())],
    );
    assert_eq!(policy.allowed_methods(), ["allowed", "ignored", "excluded"]);
    assert!(policy.allowed_method("ignored"));
    assert!(policy.ignored_method("excluded"));
}

#[test]
fn any_deprecated_regexp_excludes_the_entire_deprecated_configuration() {
    let pattern = Regex::new("^legacy").unwrap();
    let policy = AllowedMethods::new(
        vec!["allowed".into()],
        vec![ConfiguredMethod::Pattern(pattern)],
        vec![ConfiguredMethod::Name("also_dropped".into())],
    );
    assert_eq!(policy.allowed_methods(), ["allowed"]);
    assert_eq!(policy.cop_config_deprecated_values().len(), 2);
}
