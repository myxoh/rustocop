// Relevant examples ported from rubocop-ast 1.49.1:
// spec/rubocop/ast/send_node_spec.rb
// Spec SHA-256: a9d9b8f8c2d4e94f2f9b8297abac812f73ebd9a4c6874f81488f9fab64bf083d

use super::method_dispatch_node::{DefModifierArgument, MacroScope, MethodDispatch};
use super::method_identifier_predicates::{MethodIdentifier, ReceiverKind};

fn dispatch<'a>(
    name: &'a str,
    receiver: ReceiverKind,
    arguments: usize,
    scope: MacroScope,
) -> MethodDispatch<'a> {
    MethodDispatch::new(
        MethodIdentifier::new(name, receiver, Some(name)),
        name,
        Some(name),
        None,
        arguments,
        name.ends_with('='),
        false,
        false,
        false,
        scope,
        None,
    )
}

#[test]
fn macro_scope_matches_the_upstream_node_pattern() {
    assert!(MacroScope::Root.in_macro_scope());
    assert!(MacroScope::ClassLike.in_macro_scope());
    assert!(MacroScope::Wrapper(Box::new(MacroScope::ClassLike)).in_macro_scope());
    assert!(MacroScope::IfBody(Box::new(MacroScope::Root)).in_macro_scope());
    assert!(!MacroScope::IfCondition(Box::new(MacroScope::Root)).in_macro_scope());
    assert!(!MacroScope::Other.in_macro_scope());
    assert!(dispatch(
        "attr_reader",
        ReceiverKind::Implicit,
        1,
        MacroScope::ClassLike
    )
    .macro_call());
    assert!(!dispatch("attr_reader", ReceiverKind::Other, 1, MacroScope::ClassLike).macro_call());
}

#[test]
fn access_modifier_command_and_connector_predicates_match() {
    let bare = dispatch("private", ReceiverKind::Implicit, 0, MacroScope::ClassLike);
    assert!(bare.access_modifier());
    assert!(bare.bare_access_modifier());
    assert!(bare.special_modifier());
    assert!(bare.command("private"));
    let non_bare = dispatch("public", ReceiverKind::Implicit, 1, MacroScope::Root);
    assert!(non_bare.non_bare_access_modifier());
    assert!(!non_bare.special_modifier());

    for (connector, dot, double_colon, safe_navigation) in [
        (Some("."), true, false, false),
        (Some("::"), false, true, false),
        (Some("&."), false, false, true),
    ] {
        let call = MethodDispatch::new(
            MethodIdentifier::new("bar", ReceiverKind::Other, Some("bar")),
            "foo.bar",
            Some("bar"),
            connector,
            0,
            false,
            false,
            false,
            false,
            MacroScope::Other,
            None,
        );
        assert_eq!(call.dot(), dot);
        assert_eq!(call.double_colon(), double_colon);
        assert_eq!(call.safe_navigation(), safe_navigation);
    }
}

#[test]
fn call_shape_lambda_operator_and_def_modifier_predicates_match() {
    let setter = dispatch("bar=", ReceiverKind::Other, 1, MacroScope::Other);
    assert!(setter.setter_method());
    assert!(setter.assignment());
    let implicit_call = MethodDispatch::new(
        MethodIdentifier::new("call", ReceiverKind::Other, None),
        "foo.()",
        None,
        Some("."),
        1,
        false,
        true,
        false,
        false,
        MacroScope::Other,
        None,
    );
    assert!(implicit_call.implicit_call());
    assert!(implicit_call.block_literal());
    assert_eq!(implicit_call.selector(), None);

    let lambda = MethodDispatch::new(
        MethodIdentifier::new("lambda", ReceiverKind::Implicit, Some("lambda")),
        "lambda",
        Some("lambda"),
        None,
        0,
        false,
        true,
        false,
        false,
        MacroScope::Root,
        None,
    );
    assert!(lambda.lambda());

    let literal = MethodDispatch::new(
        MethodIdentifier::new("lambda", ReceiverKind::Implicit, Some("->")),
        "->",
        Some("->"),
        None,
        0,
        false,
        true,
        true,
        true,
        MacroScope::Root,
        None,
    );
    assert!(literal.lambda_literal());

    let unary = MethodDispatch::new(
        MethodIdentifier::new("!", ReceiverKind::Other, Some("!")),
        "!foo",
        Some("!"),
        None,
        0,
        false,
        false,
        false,
        true,
        MacroScope::Other,
        None,
    );
    assert!(unary.unary_operation());

    let binary = MethodDispatch::new(
        MethodIdentifier::new("+", ReceiverKind::Other, Some("+")),
        "foo + bar",
        Some("+"),
        None,
        1,
        false,
        false,
        false,
        false,
        MacroScope::Other,
        Some(DefModifierArgument::Dispatch {
            implicit_receiver: true,
            argument: Box::new(DefModifierArgument::Definition),
        }),
    );
    assert!(binary.binary_operation());
    assert!(binary.arithmetic_operation());
    assert!(binary.def_modifier_present());
    assert!(binary.def_modifier().is_some());
}

#[test]
fn receiver_predicates_delegate_to_identifier_semantics() {
    assert!(dispatch("bar", ReceiverKind::SelfValue, 0, MacroScope::Other).self_receiver());
    assert!(dispatch("bar", ReceiverKind::Constant, 0, MacroScope::Other).const_receiver());
    assert_eq!(
        dispatch("bar", ReceiverKind::Other, 0, MacroScope::Other).method_name(),
        "bar"
    );
}
