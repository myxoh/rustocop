use super::builder::{
    build_node, node_class, s, string_value, NodeClass, SexpValue, EMIT_FORWARD_ARG,
    EMIT_MATCH_PATTERN,
};

#[test]
fn maps_every_specialized_builder_family_and_falls_back_to_node() {
    for (kind, class) in [
        ("and", NodeClass::And),
        ("kwoptarg", NodeClass::Argument),
        ("numblock", NodeClass::Block),
        ("defs", NodeClass::Definition),
        ("kwargs", NodeClass::Hash),
        ("erange", NodeClass::Range),
        ("zsuper", NodeClass::Super),
        ("while_post", NodeClass::While),
        ("future_node", NodeClass::Node),
    ] {
        assert_eq!(node_class(kind), class, "{kind}");
    }
    const {
        assert!(EMIT_FORWARD_ARG);
        assert!(EMIT_MATCH_PATTERN);
    }
    assert_eq!(super::builder::builder_features(), (true, true));
}

#[test]
fn builder_and_sexp_select_the_same_class_and_preserve_children() {
    let children = vec![SexpValue::Symbol("name".into()), SexpValue::Nil];
    assert_eq!(
        build_node("lvasgn", children.clone()),
        s("lvasgn", children)
    );
    assert_eq!(s("lvasgn", Vec::new()).class, NodeClass::Assignment);
}

#[test]
fn string_value_preserves_invalid_utf8_bytes() {
    let bytes = [0xff, 0xfe];
    assert_eq!(string_value(&bytes), bytes);
}
