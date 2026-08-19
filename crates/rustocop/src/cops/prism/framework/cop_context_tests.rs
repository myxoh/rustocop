use super::*;
use crate::config::CopConfig;

#[test]
fn correction_plan_swaps_disjoint_ranges_without_rebuilding_source() {
    let source = "left middle right";
    let mut correction = CorrectionPlan::default();

    assert!(correction.swap(source, 0..4, 12..17));
    assert!(!correction.swap(source, 0..8, 4..12));
    assert_eq!(
        correction.into_edits(),
        vec![(0..4, "right".to_string()), (12..17, "left".to_string())]
    );
}

#[test]
fn applies_common_allow_style_and_path_policies() {
    let config = CopConfig::from_source(
        "AllCops:\n  Exclude:\n    - '**/vendor/**'\nStyle/Example:\n  EnforcedStyle: compact\n  AllowedMethods:\n    - map\n  AllowedPatterns:\n    - '^find_'\n  AllowedReceivers: [ENV]\n  Include:\n    - '**/*.rb'\n",
    );
    let policy = CopPolicy::new(&config, "Style/Example");

    assert_eq!(policy.enforced_style("expanded"), "compact");
    assert!(policy.allows_method(b"map"));
    assert!(policy.allows_method(b"find_user"));
    assert!(policy.allows_receiver(b"ENV"));
    assert!(policy.included_path("app/example.rb"));
    assert!(policy.excluded_path("app/vendor/example.rb"));
}
