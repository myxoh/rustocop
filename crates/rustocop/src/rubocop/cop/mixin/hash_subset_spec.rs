use super::hash_subset::*;

fn comparison(method: &str, left: Operand, right: Operand) -> SubsetBody {
    SubsetBody {
        method_name: method.into(),
        receiver: left,
        first_argument: right,
        argument_count: 1,
        negated: false,
    }
}

fn send(outer: &str, body: SubsetBody) -> HashSubsetSend {
    HashSubsetSend {
        method_name: outer.into(),
        selector_range: 5..11,
        block_end: 30,
        block: Some(HashSubsetBlock {
            key_argument: Operand::local("key"),
            value_argument: Operand::local("value"),
            body,
        }),
    }
}

#[test]
fn except_and_slice_semantics_follow_outer_method_and_negation() {
    let except = HashSubset {
        active_support_extensions_enabled: false,
        preference: HashSubsetPreference::Except,
    };
    let reject_equal = send(
        "reject",
        comparison(
            "==",
            Operand::local("key"),
            Operand::literal(":foo", OperandKind::Symbol),
        ),
    );
    let offense = except.on_send(&reject_equal).unwrap();
    assert_eq!(offense.range, 5..30);
    assert_eq!(offense.replacement, "except(:foo)");
    assert_eq!(offense.message, "Use `except(:foo)` instead.");

    let slice = HashSubset {
        preference: HashSubsetPreference::Slice,
        ..except
    };
    assert!(slice.on_send(&reject_equal).is_none());
}

#[test]
fn active_support_methods_require_the_key_and_never_the_value_or_a_range() {
    let subset = HashSubset {
        active_support_extensions_enabled: true,
        preference: HashSubsetPreference::Except,
    };
    let include = comparison(
        "include?",
        Operand::literal("keys", OperandKind::Other),
        Operand::local("key"),
    );
    assert!(subset.extracts_hash_subset(send("reject", include.clone()).block.as_ref().unwrap()));

    let value = comparison("include?", Operand::local("value"), Operand::local("key"));
    assert!(!subset.extracts_hash_subset(send("reject", value).block.as_ref().unwrap()));

    let range = comparison(
        "include?",
        Operand::literal("1..5", OperandKind::Range),
        Operand::local("key"),
    );
    assert!(!subset.extracts_hash_subset(send("reject", range).block.as_ref().unwrap()));
}

#[test]
fn key_source_preserves_literals_splats_and_percent_array_decoration() {
    let subset = HashSubset {
        active_support_extensions_enabled: false,
        preference: HashSubsetPreference::Except,
    };
    assert_eq!(
        subset.except_key_source(&Operand::literal(":foo", OperandKind::Symbol)),
        ":foo"
    );
    assert_eq!(subset.except_key_source(&Operand::local("keys")), "*keys");
    let array = Operand::array(
        vec![
            Operand::literal("foo", OperandKind::Symbol),
            Operand::literal("bar", OperandKind::String),
            Operand::literal("#{name}", OperandKind::DynamicSymbol),
        ],
        true,
    );
    assert_eq!(
        subset.except_key_source(&array),
        ":foo, 'bar', :\"#{name}\""
    );
}

#[test]
fn equality_comparisons_are_safe_only_for_symbol_and_string_keys() {
    let subset = HashSubset {
        active_support_extensions_enabled: false,
        preference: HashSubsetPreference::Except,
    };
    let literal = send(
        "reject",
        comparison(
            "==",
            Operand::local("key"),
            Operand::literal("foo", OperandKind::String),
        ),
    );
    assert!(subset.on_send(&literal).is_some());
    let dynamic = send(
        "reject",
        comparison("==", Operand::local("key"), Operand::local("other")),
    );
    assert!(subset.on_csend(&dynamic).is_none());
}
