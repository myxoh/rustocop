// Ported from RuboCop 1.87.0:
// spec/rubocop/cop/badge_spec.rb
// Spec SHA-256: 766b24d07ea636bbc2d270874b9e3635fb349bb49e962d7bd1ef4fd7d95ff999

use std::collections::HashSet;

use super::badge::Badge;

#[test]
fn assigns_department_and_cop_name_at_any_depth() {
    for (parts, department, name) in [
        (vec!["Foo"], None, "Foo"),
        (vec!["Foo", "Bar"], Some("Foo"), "Bar"),
        (vec!["Foo", "Bar", "Baz"], Some("Foo/Bar"), "Baz"),
        (vec!["Foo", "Bar", "Baz", "Qux"], Some("Foo/Bar/Baz"), "Qux"),
    ] {
        let badge = Badge::new(&parts);
        assert_eq!(badge.department(), department);
        assert_eq!(badge.department_name(), department);
        assert_eq!(badge.cop_name(), name);
    }
}

#[test]
fn parses_identifiers_and_class_names_like_rubocop() {
    for (identifier, expected) in [
        ("bar", "Bar"),
        ("Bar", "Bar"),
        ("snake_case/example", "SnakeCase/Example"),
        ("Foo/Bar/Baz/Qux", "Foo/Bar/Baz/Qux"),
    ] {
        assert_eq!(Badge::parse(identifier).to_string(), expected);
    }
    for (class_name, expected) in [
        ("Foo", "Foo"),
        ("Foo::Bar", "Foo/Bar"),
        ("RuboCop::Cop::Foo", "Cop/Foo"),
        ("RuboCop::Cop::Foo::Bar", "Foo/Bar"),
        ("RuboCop::Cop::Foo::Bar::Baz", "Foo/Bar/Baz"),
    ] {
        assert_eq!(Badge::for_class(class_name).to_string(), expected);
    }
}

#[test]
fn compares_matches_qualifies_and_adds_departments() {
    let first = Badge::new(&["Foo", "Bar"]);
    let second = Badge::new(&["Foo", "Bar"]);
    assert_eq!(HashSet::from([first.clone(), second.clone()]).len(), 1);

    let unqualified = Badge::parse("Bar");
    let qualified = Badge::parse("Department/Bar");
    assert!(!unqualified.qualified());
    assert!(qualified.qualified());
    assert!(unqualified.matches(&qualified));
    assert_eq!(
        unqualified.with_department("Deep/Department").to_string(),
        "Deep/Department/Bar"
    );
    assert!(first.equivalent(&second));
    assert_eq!(first.hash_value(), second.hash_value());
    assert_eq!(qualified.display(), "Department/Bar");
}

#[test]
fn camel_cases_the_upstream_examples() {
    assert_eq!(Badge::camel_case("lint"), "Lint");
    assert_eq!(Badge::camel_case("foo_bar"), "FooBar");
    assert_eq!(Badge::camel_case("rspec"), "RSpec");
}
