use super::configuration::*;

#[test]
fn array_min_size_builds_the_same_auto_generated_configuration() {
    let mut state = ArrayMinSize::new(3);
    assert_eq!(state.min_size_config(), 3);
    assert!(state.below_array_length(2));
    assert_eq!(
        state.largest_brackets_size(ArrayStyle::Brackets, 4),
        Some(4)
    );
    assert_eq!(state.smallest_percent_size(ArrayStyle::Percent, 6), Some(6));
    state.array_style_detected(ArrayStyle::Brackets, 4);
    state.array_style_detected(ArrayStyle::Percent, 6);
    assert_eq!(
        state.config_to_allow_offenses().get("EnforcedStyle"),
        Some(&AutoConfigValue::Style("percent".into()))
    );
    assert_eq!(
        state.config_to_allow_offenses().get("MinSize"),
        Some(&AutoConfigValue::Integer(5))
    );
    state.array_style_detected(ArrayStyle::Percent, 3);
    state.array_style_detected(ArrayStyle::Brackets, 4);
    assert_eq!(
        state.config_to_allow_offenses(),
        &[("Enabled".into(), AutoConfigValue::Bool(false))].into()
    );
}

#[test]
fn enforced_style_intersects_ambiguous_observations_and_disables_conflicts() {
    let supported = ["a", "b", "c"].map(str::to_owned).to_vec();
    let mut detector = ConfigurableEnforcedStyle::new("EnforcedStyle", "a", supported).unwrap();
    detector.ambiguous_style_detected(&["a".into(), "b".into()]);
    assert_eq!(detector.detected_style().unwrap(), ["a", "b"]);
    detector.unexpected_style_detected("b");
    assert_eq!(detector.detected_style().unwrap(), ["b"]);
    detector.correct_style_detected();
    assert!(detector.no_acceptable_style());
    detector.conflicting_styles_detected();
    assert!(detector.no_acceptable_style());
    assert_eq!(detector.alternative_style(), Err(StyleError::NotBinary));
}

#[test]
fn binary_alternative_and_unknown_style_validation_match_rubocop() {
    let mut detector =
        ConfigurableEnforcedStyle::new("EnforcedStyle", "a", vec!["a".into(), "b".into()]).unwrap();
    assert_eq!(detector.style(), "a");
    assert_eq!(detector.supported_styles(), ["a", "b"]);
    assert_eq!(detector.alternative_style(), Ok("b"));
    assert!(detector.style_configured());
    assert_eq!(detector.style_parameter_name(), "EnforcedStyle");
    detector.opposite_style_detected();
    assert_eq!(detector.detected_style().unwrap(), ["b"]);
    assert!(ConfigurableEnforcedStyle::new("EnforcedStyle", "x", vec!["a".into()]).is_err());

    detector.unrecognized_style_detected();
    assert!(detector.no_acceptable_style());
}

#[test]
fn naming_numbering_and_hash_key_formats_cover_all_styles() {
    assert!(valid_naming("@snake_case?", "snake_case"));
    assert!(!valid_naming("camelCase", "snake_case"));
    assert!(valid_naming("camelCase", "camelCase"));
    assert!(valid_numbering("thing_1", "snake_case"));
    assert!(!valid_numbering("thing1", "snake_case"));
    assert!(valid_numbering("thing1", "normalcase"));
    assert!(!valid_numbering("thing1", "non_integer"));
    assert!(valid_numbering("_1", "non_integer"));
    assert!(hash_key(true, true));
    assert!(!hash_key(true, false));
    assert_eq!(max_parameter_name(), "Max");
}
