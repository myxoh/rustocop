// Ported from RuboCop 1.87.0:
// spec/rubocop/cop/severity_spec.rb
// Spec SHA-256: e03becdcfa419c66013678f6b30d395cbe77e28ceec4ec94cd8f4dde6a0906b7

use std::str::FromStr;

use super::severity::Severity;

#[test]
fn exposes_names_codes_levels_and_ordering() {
    let severities = [
        Severity::Info,
        Severity::Refactor,
        Severity::Convention,
        Severity::Warning,
        Severity::Error,
        Severity::Fatal,
    ];
    for (index, severity) in severities.iter().copied().enumerate() {
        assert_eq!(severity.name(), Severity::NAMES[index]);
        assert_eq!(severity.to_string(), Severity::NAMES[index]);
        assert_eq!(severity.level(), index + 1);
        assert_eq!(severity.code(), ['I', 'R', 'C', 'W', 'E', 'F'][index]);
        assert_eq!(severity.display(), Severity::NAMES[index]);
        assert!(severity.equivalent(severity));
        assert_eq!(severity.compare(severity), std::cmp::Ordering::Equal);
        assert_eq!(severity.hash_value(), severity.hash_value());
        if let Some(next) = severities.get(index + 1) {
            assert!(severity < *next);
        }
    }
}

#[test]
fn constructs_from_names_and_codes_and_rejects_unknown_values() {
    for (name, code, expected) in [
        ("info", "I", Severity::Info),
        ("refactor", "R", Severity::Refactor),
        ("convention", "C", Severity::Convention),
        ("warning", "W", Severity::Warning),
        ("error", "E", Severity::Error),
        ("fatal", "F", Severity::Fatal),
    ] {
        assert_eq!(Severity::from_str(name).unwrap(), expected);
        assert_eq!(Severity::from_str(code).unwrap(), expected);
    }
    assert_eq!(
        Severity::from_str("unknown").unwrap_err(),
        "Unknown severity: unknown"
    );
    assert_eq!(Severity::name_from_code("W"), "warning");
    assert_eq!(Severity::name_from_code("warning"), "warning");
}
