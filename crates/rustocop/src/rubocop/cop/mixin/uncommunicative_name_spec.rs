use super::uncommunicative_name::*;

fn checker() -> UncommunicativeName {
    UncommunicativeName::new(vec!["x".into()], vec!["bad".into()], false, 3)
}

#[test]
fn check_preserves_underscore_and_rest_argument_range_rules() {
    let offenses = checker().check(
        ParameterOwner::Method,
        &[
            Argument {
                name: Some("_".into()),
                begin: 0,
                kind: ArgumentKind::Argument,
            },
            Argument {
                name: Some("_x".into()),
                begin: 4,
                kind: ArgumentKind::Argument,
            },
            Argument {
                name: Some("Ab1".into()),
                begin: 8,
                kind: ArgumentKind::RestArgument,
            },
            Argument {
                name: None,
                begin: 20,
                kind: ArgumentKind::KeywordRestArgument,
            },
        ],
    );
    assert_eq!(offenses.len(), 2);
    assert!(offenses.iter().all(|offense| offense.range == (8..12)));
    assert_eq!(
        offenses[0].message,
        "Only use lowercase characters for method parameter."
    );
    assert_eq!(
        offenses[1].message,
        "Do not end method parameter with a number."
    );
}

#[test]
fn every_applicable_rule_reports_independently_in_ruby_order() {
    let offenses = checker().check(
        ParameterOwner::Block,
        &[Argument {
            name: Some("_bad".into()),
            begin: 2,
            kind: ArgumentKind::Argument,
        }],
    );
    assert_eq!(offenses.len(), 1);
    assert_eq!(offenses[0].range, 2..6);
    assert_eq!(
        offenses[0].message,
        "Do not use bad as a name for a block parameter."
    );

    let short = checker().issue_offenses(ParameterOwner::Block, 0..1, "a");
    assert_eq!(
        short[0].message,
        "Block parameter must be at least 3 characters long."
    );
}

#[test]
fn configured_values_are_exposed_without_reinterpretation() {
    let checker = checker();
    assert_eq!(checker.allowed_names(), &["x"]);
    assert_eq!(checker.forbidden_names(), &["bad"]);
    assert!(!checker.allow_nums());
    assert_eq!(checker.min_length(), 3);
    assert!(checker.uppercase("aB"));
    assert!(checker.ends_with_num("a2"));
    assert!(checker.long_enough("abc"));
}
