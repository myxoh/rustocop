use std::collections::HashMap;

use super::policies::*;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

#[test]
fn identifier_and_pattern_policies_match_rubocop_branches() {
    let names = vec!["example".to_owned()];
    assert!(allowed_identifier("@@example", &names));
    assert!(forbidden_identifier("$example", &names));
    assert!(!allowed_identifier("other", &names));

    let patterns = vec!["^spec/".to_owned(), "generated".to_owned()];
    assert!(matches_allowed_pattern("spec/model_spec.rb", &patterns));
    assert!(forbidden_pattern("generated_method", &patterns));
    assert!(!forbidden_pattern("ordinary", &patterns));
}

#[test]
fn deprecated_method_and_pattern_configuration_follows_rubocop_split() {
    let configured = vec!["kept".to_owned()];
    let literals = vec![ConfiguredName::Literal("legacy".to_owned())];
    assert_eq!(allowed_methods(&configured, &literals), ["kept", "legacy"]);
    assert!(allowed_method(
        "legacy",
        &allowed_methods(&configured, &literals)
    ));

    let regexp = vec![
        ConfiguredName::Literal("legacy".to_owned()),
        ConfiguredName::Pattern("^generated".to_owned()),
    ];
    assert_eq!(allowed_methods(&configured, &regexp), ["kept"]);
    assert_eq!(
        allowed_patterns(&["own".to_owned()], &["ignored".to_owned()], &regexp),
        ["own", "ignored", "legacy", "^generated"]
    );
}

#[test]
fn validates_minimum_body_and_branch_configuration() {
    assert_eq!(min_body_length(None).unwrap(), 1);
    assert_eq!(min_body_length(Some(2)).unwrap(), 2);
    assert_eq!(
        min_body_length(Some(0)).unwrap_err(),
        "MinBodyLength needs to be a positive integer!"
    );
    assert!(meets_min_body_length(1, 4, 2));
    assert!(!meets_min_body_length(1, 3, 2));

    assert_eq!(min_branches_count(None).unwrap(), 3);
    assert_eq!(
        min_branches_count(Some(-1)).unwrap_err(),
        "MinBranchesCount needs to be a positive integer!"
    );
    assert!(meets_min_branches_count(3, 3));
    assert!(!meets_min_branches_count(2, 3));
}

#[test]
fn resolves_and_validates_preferred_delimiters() {
    let preferred = PreferredDelimiters::new(
        "%w",
        HashMap::from([
            ("default".to_owned(), "[]".to_owned()),
            ("%w".to_owned(), "()".to_owned()),
        ]),
    )
    .unwrap();
    assert_eq!(preferred.delimiters(), ['(', ')']);
    assert!(PreferredDelimiters::new(
        "%w",
        HashMap::from([("invalid".to_owned(), "[]".to_owned())]),
    )
    .unwrap_err()
    .contains("invalid"));
}

#[test]
fn target_ruby_version_uses_unbounded_defaults_and_inclusive_limits() {
    let mut target = TargetRubyVersion::default();
    assert!(target.supports(1.9));
    assert!(target.supports(4.1));
    target.minimum_target_ruby_version(3.2);
    target.maximum_target_ruby_version(3.4);
    assert_eq!(target.required_minimum_ruby_version(), Some(3.2));
    assert_eq!(target.required_maximum_ruby_version(), Some(3.4));
    assert!(!target.supports(3.1));
    assert!(target.supports(3.2));
    assert!(target.supports(3.4));
    assert!(!target.supports(3.5));
}

#[test]
fn conditional_branch_collection_flattens_elsif_chains() {
    let parsed = ProcessedSource::new(
        "if first\n  one\nelsif second\n  two\nelse\n  three\nend",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let branches = if_conditional_branches(parsed.ast().unwrap());
    assert_eq!(branches.len(), 2);
    assert_eq!(branches[0].method_name(), Some("one"));
    assert_eq!(branches[1].method_name(), Some("two"));
}
