use super::hash_shorthand_syntax::*;

fn pair(key: &str, value: Option<&str>) -> HashPair {
    HashPair {
        key_source: key.into(),
        value_source: value.map(str::to_owned),
        key_is_symbol: true,
        value_is_send_or_local: true,
        parent_is_hash: true,
        parent_has_braces: true,
        dispatch: None,
    }
}

#[test]
fn always_and_never_styles_follow_pair_callback_branches() {
    let always = HashShorthandSyntax::new(3.1, Some("always"));
    let explicit = pair("foo", Some("foo"));
    let offense = always.on_pair(0, &explicit, Some(&explicit)).unwrap();
    assert_eq!(offense.message, OMIT_HASH_VALUE_MSG);
    assert_eq!(offense.replacement, "foo:");
    assert!(always.on_pair(0, &pair("foo", None), None).is_none());
    assert!(always
        .on_pair(0, &pair("foo?", Some("foo?")), None)
        .is_none());

    let never = HashShorthandSyntax::new(3.1, Some("never"));
    let omitted = pair("foo", None);
    let offense = never.on_pair(0, &omitted, Some(&omitted)).unwrap();
    assert_eq!(offense.message, EXPLICIT_HASH_VALUE_MSG);
    assert_eq!(offense.replacement, "foo: foo");
}

#[test]
fn mixed_consistency_checks_distinguish_needed_omitted_and_omittable_values() {
    let syntax = HashShorthandSyntax::new(3.1, Some("consistent"));
    let hash = HashNode {
        pairs: vec![pair("foo", None), pair("bar", Some("other"))],
        hash_type: true,
    };
    let offenses = syntax.on_hash_for_mixed_shorthand(&hash);
    assert_eq!(offenses.len(), 1);
    assert_eq!(offenses[0].pair_index, 0);
    assert_eq!(offenses[0].message, DO_NOT_MIX_EXPLICIT_VALUE_MSG);

    let hash = HashNode {
        pairs: vec![pair("foo", None), pair("bar", Some("bar"))],
        hash_type: true,
    };
    let offenses = syntax.on_hash_for_mixed_shorthand(&hash);
    assert_eq!(offenses.len(), 1);
    assert_eq!(offenses[0].pair_index, 1);
    assert_eq!(offenses[0].message, DO_NOT_MIX_OMIT_VALUE_MSG);
}

#[test]
fn either_consistent_accepts_uniform_explicit_omittable_values() {
    let syntax = HashShorthandSyntax::new(3.1, Some("either_consistent"));
    let hash = HashNode {
        pairs: vec![pair("foo", Some("foo")), pair("bar", Some("bar"))],
        hash_type: true,
    };
    assert!(syntax.on_hash_for_mixed_shorthand(&hash).is_empty());
    assert!(HashShorthandSyntax::new(3.0, Some("consistent"))
        .on_hash_for_mixed_shorthand(&hash)
        .is_empty());
}

#[test]
fn modifier_context_and_parenthesis_guard_match_rubocop_order() {
    let syntax = HashShorthandSyntax::new(3.1, Some("always"));
    let dispatch = DispatchContext {
        method_name: "call".into(),
        send_type: true,
        hash_is_receiver: false,
        assignment_method: false,
        parenthesized: false,
        parent_parenthesized: false,
        modifier_form_ancestor: true,
        last_expression: false,
        requires_parentheses_context: true,
        selector: "call".into(),
        arguments: vec!["foo: foo".into()],
    };
    let mut explicit = pair("foo", Some("foo"));
    explicit.parent_has_braces = false;
    explicit.dispatch = Some(dispatch.clone());
    assert!(syntax.require_hash_value_for_around_hash_literal(&explicit));

    explicit.parent_has_braces = true;
    let offense = syntax.on_pair(0, &explicit, Some(&explicit)).unwrap();
    assert!(offense.add_parentheses);
    let def_node = syntax
        .def_node_that_require_parentheses(&explicit, Some(&explicit))
        .unwrap();
    assert_eq!(def_node.selector(), "call");
    assert_eq!(def_node.first_argument(), Some("foo: foo"));
    assert_eq!(def_node.last_argument(), Some("foo: foo"));
}
