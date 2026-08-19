use super::*;

#[test]
fn expands_departments_without_enabling_default_disabled_cops() {
    let selection = CopSelection::only("Style");
    assert!(selection.enabled("Style/HashSyntax"));
    assert!(!selection.enabled("Style/Copyright"));
    assert!(!selection.enabled("Style/Documentation"));
}

#[test]
fn explicitly_enables_default_disabled_cops() {
    let selection = CopSelection::only("Style/Documentation");
    assert!(selection.enabled("Style/Documentation"));
    assert!(!selection.enabled("Style/HashSyntax"));
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
    let config = CopConfig::from_source(
        "Style/PercentLiteralDelimiters:\n  PreferredDelimiters:\n    '%x': '[]'\n",
    );

    assert_eq!(
        config
            .map("Style/PercentLiteralDelimiters", "PreferredDelimiters")
            .and_then(|values| values.get("%x"))
            .map(String::as_str),
        Some("[]")
    );
}
