// Relevant examples ported from rubocop-ast 1.49.1:
// spec/rubocop/ast/send_node_spec.rb
// Spec SHA-256: a9d9b8f8c2d4e94f2f9b8297abac812f73ebd9a4c6874f81488f9fab64bf083d

use super::method_identifier_predicates::{MethodIdentifier, ReceiverKind};

fn method(name: &str) -> MethodIdentifier<'_> {
    MethodIdentifier::new(name, ReceiverKind::Other, Some(name))
}

#[test]
fn matches_operator_and_nonmutating_method_sets() {
    assert!(method("+").operator_method());
    assert!(method("!").operator_method());
    assert!(!method("bar=").operator_method());
    assert!(method("+").nonmutating_binary_operator_method());
    assert!(!method("<<").nonmutating_binary_operator_method());
    assert!(method("!").nonmutating_unary_operator_method());
    assert!(method("!").nonmutating_operator_method());
    assert!(!method("bar").nonmutating_operator_method());
    assert!(method("reverse").nonmutating_array_method());
    assert!(!method("push").nonmutating_array_method());
    assert!(method("slice").nonmutating_hash_method());
    assert!(!method("delete").nonmutating_hash_method());
    assert!(method("squeeze").nonmutating_string_method());
    assert!(!method("lstrip!").nonmutating_string_method());
}

#[test]
fn matches_identifier_categories_and_receiver_shapes() {
    assert!(method(">=").comparison_method());
    assert!(!method("!").comparison_method());
    assert!(method("bar=").assignment_method());
    assert!(method("[]=").assignment_method());
    assert!(!method("==").assignment_method());
    assert!(method("each_slice").enumerator_method());
    assert!(method("all?").enumerable_method());
    assert!(method("bar?").predicate_method());
    assert!(method("bar!").bang_method());
    assert!(method("Integer").camel_case_method());
    assert!(method("bar").method("bar"));

    assert!(MethodIdentifier::new("bar", ReceiverKind::SelfValue, Some("bar")).self_receiver());
    assert!(MethodIdentifier::new("bar", ReceiverKind::Constant, Some("bar")).const_receiver());
    assert!(!MethodIdentifier::new("bar", ReceiverKind::Implicit, Some("bar")).self_receiver());
}

#[test]
fn distinguishes_keyword_and_symbol_negation() {
    let keyword = MethodIdentifier::new("!", ReceiverKind::Other, Some("not"));
    let bang = MethodIdentifier::new("!", ReceiverKind::Other, Some("!"));
    let implicit = MethodIdentifier::new("!", ReceiverKind::Implicit, Some("!"));
    assert!(keyword.negation_method());
    assert!(keyword.prefix_not());
    assert!(!keyword.prefix_bang());
    assert!(bang.prefix_bang());
    assert!(!bang.prefix_not());
    assert!(!implicit.negation_method());
}
