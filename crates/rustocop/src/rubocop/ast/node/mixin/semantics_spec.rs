use super::semantics::*;

#[test]
fn literal_binary_and_conditional_helpers_preserve_branch_order() {
    assert_eq!(basic_literal_value(&[42, 7]), Some(42));
    let tree = BinaryCondition::Operator(
        Box::new(BinaryCondition::Begin(Box::new(BinaryCondition::Operator(
            Box::new(BinaryCondition::Atom("a")),
            Box::new(BinaryCondition::Atom("b")),
        )))),
        Box::new(BinaryCondition::Atom("c")),
    );
    assert_eq!(tree.conditions(), [&"a", &"b", &"c"]);
    let lines = ConditionalLines {
        keyword_line: 1,
        condition_line: 2,
    };
    assert!(!lines.single_line_condition());
    assert!(lines.multiline_condition());
}

#[test]
fn constant_paths_match_absolute_relative_and_naming_semantics() {
    let path = ConstantPath::new(true, vec!["Foo", "Bar", "BAZ"]);
    assert_eq!(path.namespace().as_deref(), Some("::Foo::Bar"));
    assert_eq!(path.short_name(), Some("BAZ"));
    assert!(path.absolute());
    assert!(!path.relative());
    assert_eq!(path.each_path(), ["::", "::Foo", "::Foo::Bar"]);
    assert!(!path.module_name());
    assert!(!path.class_name());
    assert!(ConstantPath::new(false, vec!["CamelCase"]).module_name());
}

#[test]
fn modifier_numeric_and_parameterized_helpers_match() {
    assert!(modifier_form(false));
    assert!(!modifier_form(true));
    assert!(numeric_has_sign("+42"));
    assert!(numeric_has_sign("-1.0"));
    assert!(!numeric_has_sign("42"));

    let arguments = Parameterized::new(
        vec![
            (ArgumentKind::Other, "first"),
            (ArgumentKind::Splat, "rest"),
            (ArgumentKind::BlockPass, "block"),
        ],
        Some(')'),
    );
    assert!(arguments.parenthesized());
    assert!(arguments.has_arguments());
    assert_eq!(arguments.arguments().len(), 3);
    assert_eq!(arguments.first_argument(), Some(&"first"));
    assert_eq!(arguments.last_argument(), Some(&"block"));
    assert!(arguments.splat_argument());
    assert!(arguments.rest_argument());
    assert!(arguments.block_argument());

    let rest = Parameterized::new(vec![(ArgumentKind::RestArgument, "rest")], None);
    assert!(rest.rest_argument());
    let block = Parameterized::new(vec![(ArgumentKind::BlockArgument, "block")], None);
    assert!(block.block_argument());
}

#[test]
fn predicate_operator_strings_and_categories_match() {
    for (operator, source, logical, semantic) in [
        (PredicateOperator::LogicalAnd, "&&", true, false),
        (PredicateOperator::SemanticAnd, "and", false, true),
        (PredicateOperator::LogicalOr, "||", true, false),
        (PredicateOperator::SemanticOr, "or", false, true),
    ] {
        assert_eq!(operator.operator(), source);
        assert_eq!(operator.logical(), logical);
        assert_eq!(operator.semantic(), semantic);
    }
}
