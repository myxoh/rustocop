use super::*;

#[test]
fn expands_departments_without_enabling_default_disabled_cops() {
    let selection = CopSelection::only("Style");
    let config = CopConfig::default();
    assert!(selection.enabled("Style/KeywordParametersOrder", &config));
    assert!(!selection.enabled("Style/Copyright", &config));
}

#[test]
fn explicitly_enables_default_disabled_cops() {
    let selection = CopSelection::only("Style/Copyright");
    let config = CopConfig::default();
    assert!(selection.enabled("Style/Copyright", &config));
    assert!(!selection.enabled("Style/KeywordParametersOrder", &config));
}

#[test]
fn default_selection_honors_pinned_enabled_and_new_cop_settings() {
    let selection = CopSelection::default_enabled();
    let config = CopConfig::default();

    assert!(selection.enabled("Style/KeywordParametersOrder", &config));
    assert!(!selection.enabled("Lint/ConstantResolution", &config));
    assert!(!selection.enabled("Lint/AmbiguousOperatorPrecedence", &config));
}

#[test]
fn new_cops_setting_enables_pending_but_not_disabled_cops() {
    let selection = CopSelection::default_enabled();
    let config = CopConfig::from_source("AllCops:\n  NewCops: enable\n");

    assert!(selection.enabled("Lint/AmbiguousOperatorPrecedence", &config));
    assert!(!selection.enabled("Lint/ConstantResolution", &config));
}

#[test]
fn explicit_cop_enabled_setting_overrides_pinned_defaults() {
    let selection = CopSelection::default_enabled();
    let config = CopConfig::from_source(
        "Lint/ConstantResolution:\n  Enabled: true\nStyle/KeywordParametersOrder:\n  Enabled: false\n",
    );

    assert!(selection.enabled("Lint/ConstantResolution", &config));
    assert!(!selection.enabled("Style/KeywordParametersOrder", &config));
}

#[test]
fn disabled_by_default_enables_only_explicitly_configured_cops() {
    let selection = CopSelection::default_enabled();
    let config = CopConfig::from_source(
        "AllCops:\n  DisabledByDefault: true\nStyle/FrozenStringLiteralComment:\n",
    );

    assert!(selection.enabled("Style/FrozenStringLiteralComment", &config));
    assert!(!selection.enabled("Style/KeywordParametersOrder", &config));
}

#[test]
fn enabled_by_default_still_respects_explicit_disabling() {
    let selection = CopSelection::default_enabled();
    let config = CopConfig::from_source(
        "AllCops:\n  EnabledByDefault: true\nStyle/KeywordParametersOrder:\n  Enabled: false\n",
    );

    assert!(selection.enabled("Lint/ConstantResolution", &config));
    assert!(!selection.enabled("Style/KeywordParametersOrder", &config));
}

#[test]
fn reads_scoped_cop_configuration_values() {
    let config = CopConfig::from_source(
        "Style/Example:\n  EnforcedStyle: custom\nOther/Rule:\n  Allowed: false\n",
    );

    assert_eq!(
        config.value("Style/Example", "EnforcedStyle"),
        Some("custom")
    );
    assert_eq!(config.value("Other/Rule", "Allowed"), Some("false"));
    assert_eq!(config.value("Style/Example", "Allowed"), None);
}

#[test]
fn preserves_symbol_map_entries_separately_from_string_entries() {
    let config = CopConfig::from_source(
        "Style/InvertibleUnlessCondition:\n  InverseMethods:\n    :include?: :exclude?\n    plain?: ignored?\n",
    );

    assert_eq!(
        config
            .map("Style/InvertibleUnlessCondition", "InverseMethods")
            .and_then(|values| values.get("include?").map(String::as_str)),
        Some("exclude?")
    );
    let symbols = config
        .symbol_map("Style/InvertibleUnlessCondition", "InverseMethods")
        .unwrap();
    assert_eq!(
        symbols.get("include?").map(String::as_str),
        Some("exclude?")
    );
    assert!(!symbols.contains_key("plain?"));
}

#[test]
fn reads_typed_scalars_and_block_or_inline_lists() {
    let config = CopConfig::from_source(
            "Style/Example:\n  Enabled: true\n  Max: 12\n  AllowedMethods:\n    - map\n    - 'each'\n  AllowedPatterns: [foo, 'bar']\n  PreferredMethods:\n    intern: to_sym\n",
        );

    assert_eq!(config.bool("Style/Example", "Enabled"), Some(true));
    assert_eq!(config.usize("Style/Example", "Max"), Some(12));
    assert_eq!(
        config.values("Style/Example", "AllowedMethods"),
        ["map", "each"]
    );
    assert_eq!(
        config.values("Style/Example", "AllowedPatterns"),
        ["foo", "bar"]
    );
    assert_eq!(
        config
            .map("Style/Example", "PreferredMethods")
            .and_then(|methods| methods.get("intern"))
            .map(String::as_str),
        Some("to_sym")
    );
}

#[test]
fn reads_serialized_regexp_lists() {
    let config = CopConfig::from_source(
        "Lint/Example:\n  AllowedPatterns:\n  - \"$regexp\": min\n    options: 0\n",
    );

    assert!(config.patterns("Lint/Example", "AllowedPatterns")[0].is_match("minutes"));
}

#[test]
fn preserves_block_scalars_blank_lines_and_quoted_hash_values() {
    let config = CopConfig::from_source(
            "Style/Copyright:\n  Notice: |\n    Copyright 2026\n\n    Acme Inc\n  AutocorrectNotice: '# Copyright 2026'\n",
        );

    assert_eq!(
        config.value("Style/Copyright", "Notice"),
        Some("Copyright 2026\n\nAcme Inc\n")
    );
    assert_eq!(
        config.value("Style/Copyright", "AutocorrectNotice"),
        Some("# Copyright 2026")
    );
}

#[test]
fn unquotes_nested_map_keys() {
    let config = CopConfig::from_source("Style/Example:\n  PreferredDelimiters:\n    '%x': '[]'\n");

    assert_eq!(
        config
            .map("Style/Example", "PreferredDelimiters")
            .and_then(|values| values.get("%x"))
            .map(String::as_str),
        Some("[]")
    );
}

#[test]
fn merges_user_configuration_over_pinned_rubocop_defaults() {
    let defaults = CopConfig::default();
    assert_eq!(
        defaults
            .map("Style/PercentLiteralDelimiters", "PreferredDelimiters")
            .and_then(|values| values.get("%r"))
            .map(String::as_str),
        Some("{}")
    );

    let configured = CopConfig::from_source(
        "Style/PercentLiteralDelimiters:\n  PreferredDelimiters:\n    default: '[]'\n",
    );
    let delimiters = configured
        .map("Style/PercentLiteralDelimiters", "PreferredDelimiters")
        .unwrap();
    assert_eq!(delimiters.get("default").map(String::as_str), Some("[]"));
    assert_eq!(delimiters.get("%r").map(String::as_str), None);
}
