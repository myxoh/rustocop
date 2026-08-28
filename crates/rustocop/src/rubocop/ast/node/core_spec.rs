// Consolidated behavioral port from rubocop-ast 1.49.1:
// spec/rubocop/ast/node_spec.rb
// Spec SHA-256: 8bc7277f33d3bc5d4b0e1f2821c3520b4700f5822ee60ed99a765e13613bde53

use super::super::processed_source::{ParserEngine, ProcessedSource};
use super::core::{Ast, NodeValue, RubyStringEncoding};

#[test]
fn assigns_parents_roots_completion_and_siblings() {
    let mut ast = Ast::new("a; b; c");
    let a = ast.add_node("lvar", vec![NodeValue::Symbol("a".into())], Some(0..1));
    let b = ast.add_node("lvar", vec![NodeValue::Symbol("b".into())], Some(3..4));
    let c = ast.add_node("lvar", vec![NodeValue::Symbol("c".into())], Some(6..7));
    let root = ast.add_node(
        "begin",
        vec![NodeValue::Node(a), NodeValue::Node(b), NodeValue::Node(c)],
        Some(0..7),
    );
    let b = ast.node(b);
    assert!(b.parent_exists());
    assert_eq!(b.parent().unwrap().id(), root);
    assert!(!b.root());
    assert_eq!(b.sibling_index(), Some(1));
    assert_eq!(b.left_sibling().unwrap().source(), Some("a"));
    assert_eq!(b.right_sibling().unwrap().source(), Some("c"));
    assert_eq!(b.left_siblings().len(), 1);
    assert_eq!(b.right_siblings().len(), 1);
    ast.complete(root);
    assert!(ast.node(root).complete());
    assert!(ast.node(a).complete());
}

#[test]
fn traverses_children_descendants_nodes_and_ancestors_depth_first() {
    let mut ast = Ast::new("[1, [2]]");
    let one = ast.add_node("int", vec![NodeValue::Integer(1)], Some(1..2));
    let two = ast.add_node("int", vec![NodeValue::Integer(2)], Some(5..6));
    let inner = ast.add_node("array", vec![NodeValue::Node(two)], Some(4..7));
    let root = ast.add_node(
        "array",
        vec![NodeValue::Node(one), NodeValue::Node(inner)],
        Some(0..8),
    );
    let root = ast.node(root);
    assert_eq!(root.child_nodes().len(), 2);
    assert_eq!(root.each_child_node(&["int"]).len(), 1);
    assert_eq!(
        root.descendants()
            .iter()
            .map(|node| node.kind())
            .collect::<Vec<_>>(),
        ["int", "array", "int"]
    );
    assert_eq!(root.each_descendant(&["int"]).len(), 2);
    assert_eq!(root.each_node(&["array"]).len(), 2);
    assert_eq!(
        ast.node(two)
            .ancestors()
            .iter()
            .map(|node| node.kind())
            .collect::<Vec<_>>(),
        ["array", "array"]
    );
}

#[test]
fn precomputed_hash_equivalence_preserves_squiggly_heredoc_dedent_buckets() {
    let source = "gem <<~A\n  rubocop\nA\ngem \"rubocop\\n\"\ngem <<~B\n  rubocop\nB\n";
    let parsed = parse(source);
    let arguments = parsed
        .ast()
        .unwrap()
        .each_node(&["send"])
        .into_iter()
        .filter_map(|send| send.first_argument())
        .collect::<Vec<_>>();

    assert!(arguments[0].structurally_equal(arguments[1]));
    assert!(!arguments[0].rubocop_hash_equivalent(arguments[1]));
    assert!(arguments[0].structurally_equal(arguments[2]));
    assert!(arguments[0].rubocop_hash_equivalent(arguments[2]));
}

#[test]
fn precomputed_hash_equivalence_decodes_squiggly_heredoc_escapes_before_dedent() {
    let source = "gem <<~A\n  rubo\\x63op\nA\ngem <<~B\n  rubocop\nB\n";
    let parsed = parse(source);
    let arguments = parsed
        .ast()
        .unwrap()
        .each_node(&["send"])
        .into_iter()
        .filter_map(|send| send.first_argument())
        .collect::<Vec<_>>();

    assert!(arguments[0].structurally_equal(arguments[1]));
    assert!(arguments[0].rubocop_hash_equivalent(arguments[1]));
}

#[test]
fn static_string_equality_preserves_bytes_without_special_casing_replacement_characters() {
    let source = "gem \"\\u{FFFD}\"\ngem \"�\"\ngem \"\\xFF\"\ngem \"\\xFF\"\n";
    let parsed = parse(source);
    let arguments = parsed
        .ast()
        .unwrap()
        .each_node(&["send"])
        .into_iter()
        .filter_map(|send| send.first_argument())
        .collect::<Vec<_>>();

    assert!(arguments[0].structurally_equal(arguments[1]));
    assert!(arguments[0].rubocop_hash_equivalent(arguments[1]));
    assert!(!arguments[0].structurally_equal(arguments[2]));
    assert!(!arguments[0].rubocop_hash_equivalent(arguments[2]));
    assert!(arguments[2].structurally_equal(arguments[3]));
    assert!(arguments[2].rubocop_hash_equivalent(arguments[3]));
}

#[test]
fn ruby_string_equality_and_hash_preserve_encoding_for_non_ascii_bytes() {
    let mut ast = Ast::new("");
    let utf8 = ast.add_node("str", vec![NodeValue::String("é".into())], None);
    ast.set_decoded_bytes(utf8, "é".as_bytes());
    ast.set_string_encoding(utf8, RubyStringEncoding::Utf8);
    let binary = ast.add_node("str", vec![NodeValue::String("é".into())], None);
    ast.set_decoded_bytes(binary, "é".as_bytes());
    ast.set_string_encoding(binary, RubyStringEncoding::Binary);

    assert!(!ast.node(utf8).structurally_equal(ast.node(binary)));
    assert!(!ast.node(utf8).rubocop_hash_equivalent(ast.node(binary)));

    let ascii_utf8 = ast.add_node("str", vec![NodeValue::String("gem".into())], None);
    ast.set_string_encoding(ascii_utf8, RubyStringEncoding::Utf8);
    let ascii_binary = ast.add_node("str", vec![NodeValue::String("gem".into())], None);
    ast.set_string_encoding(ascii_binary, RubyStringEncoding::Binary);

    assert!(ast
        .node(ascii_utf8)
        .structurally_equal(ast.node(ascii_binary)));
    assert!(ast
        .node(ascii_utf8)
        .rubocop_hash_equivalent(ast.node(ascii_binary)));
}

#[test]
fn binary_node_source_bytes_restore_the_original_source_spelling() {
    let prefix = "# encoding: ASCII-8BIT\n";
    let source = format!("{prefix}:caf\u{e0e9}");
    let start = prefix.chars().count();
    let mut ast = Ast::new(source);
    let symbol = ast.add_node(
        "sym",
        vec![NodeValue::Symbol("caf\u{e0e9}".into())],
        Some(start..start + 5),
    );

    assert_eq!(
        ast.node(symbol).source_bytes().as_deref(),
        Some(&b":caf\xe9"[..])
    );
}

#[test]
fn grouped_type_queries_match_all_rubocop_groups() {
    for (kind, group) in [
        ("defs", "any_def"),
        ("kwoptarg", "argument"),
        ("false", "boolean"),
        ("rational", "numeric"),
        ("xstr", "any_str"),
        ("dsym", "any_sym"),
        ("erange", "range"),
        ("csend", "call"),
        ("numblock", "any_block"),
        ("match_pattern_p", "any_match_pattern"),
    ] {
        let mut ast = Ast::new("");
        let node = ast.add_node(kind, vec![], None);
        assert!(ast.node(node).type_is(&[group]), "{kind}/{group}");
        assert!(ast.node(node).type_group_is(group), "{kind}/{group}");
    }
}

#[test]
fn updated_nodes_preserve_locations_and_replace_requested_parts() {
    let mut ast = Ast::new("value");
    let value = ast.add_node("lvar", vec![NodeValue::Symbol("value".into())], Some(0..5));
    ast.set_location(value, "name", 0..5, "value");
    let updated = ast.updated(
        value,
        Some("ivasgn"),
        Some(vec![NodeValue::Symbol("@value".into()), NodeValue::Nil]),
    );
    let updated = ast.node(updated);
    assert_eq!(updated.kind(), "ivasgn");
    assert_eq!(updated.source_range(), Some(0..5));
    assert!(updated.loc_is("name", "value"));
    assert_eq!(updated.children().len(), 2);
}

#[test]
fn structural_equality_uses_ruby_numeric_values_instead_of_storage_or_spelling() {
    let distinct_bignums = parse(
        "[99999999999999999999999999999999999999, 88899999999999999999999999999999999999]",
    );
    let nodes = distinct_bignums.ast().unwrap().child_nodes();
    assert!(!nodes[0].structurally_equal(nodes[1]));
    assert!(!nodes[0].rubocop_hash_equivalent(nodes[1]));

    for source in ["[1.0, 1e0]", "[1.0r, 1r]", "[1.0i, 1e0i]"] {
        let parsed = parse(source);
        let nodes = parsed.ast().unwrap().child_nodes();
        assert!(
            nodes[0].structurally_equal(nodes[1]),
            "numeric values should be equal for {source}"
        );
        assert!(
            nodes[0].rubocop_hash_equivalent(nodes[1]),
            "numeric hash inputs should be equal for {source}"
        );
    }
}

#[test]
fn source_line_and_location_helpers_use_character_offsets() {
    let mut ast = Ast::new("é\n  value\n");
    let node = ast.add_node("send", vec![], Some(4..9));
    ast.set_location(node, "begin", 3..4, "(");
    let node = ast.node(node);
    assert_eq!(node.source(), Some("value"));
    assert_eq!(node.first_line(), 2);
    assert_eq!(node.last_line(), 2);
    assert_eq!(node.line_count(), 1);
    assert_eq!(node.nonempty_line_count(), 1);
    assert_eq!(node.source_length(), 5);
    assert!(node.single_line());
    assert!(node.parenthesized_call());
}

#[test]
fn literal_assignment_conditional_keyword_and_loop_predicates_match_sets() {
    for kind in ["str", "int", "false", "nil", "array"] {
        let mut ast = Ast::new("");
        let id = ast.add_node(kind, vec![], None);
        assert!(ast.node(id).literal());
    }
    let mut ast = Ast::new("");
    let string = ast.add_node("str", vec![], None);
    assert!(ast.node(string).truthy_literal());
    assert!(ast.node(string).mutable_literal());
    let nil = ast.add_node("nil", vec![], None);
    assert!(ast.node(nil).falsey_literal());
    assert!(ast.node(nil).immutable_literal());
    let assignment = ast.add_node("op_asgn", vec![], None);
    assert!(ast.node(assignment).assignment());
    assert!(ast.node(assignment).assignment_or_similar());
    assert!(ast.node(assignment).shorthand_asgn());
    let conditional = ast.add_node("case", vec![], None);
    assert!(ast.node(conditional).conditional());
    let loop_node = ast.add_node("while_post", vec![], None);
    assert!(ast.node(loop_node).post_condition_loop());
    assert!(ast.node(loop_node).loop_keyword());
}

#[test]
fn pure_recursion_and_value_usage_follow_parent_context() {
    let mut ast = Ast::new("[1, foo]");
    let one = ast.add_node("int", vec![NodeValue::Integer(1)], Some(1..2));
    let foo = ast.add_node(
        "send",
        vec![NodeValue::Nil, NodeValue::Symbol("foo".into())],
        Some(4..7),
    );
    let array = ast.add_node(
        "array",
        vec![NodeValue::Node(one), NodeValue::Node(foo)],
        Some(0..8),
    );
    assert!(!ast.node(array).pure());
    assert!(!ast.node(array).value_used());
    assert!(!ast.node(one).value_used());
    assert!(!ast.node(foo).value_used());

    let mut ast = Ast::new("call(1)");
    let one = ast.add_node("int", vec![NodeValue::Integer(1)], Some(5..6));
    let _call = ast.add_node(
        "send",
        vec![
            NodeValue::Nil,
            NodeValue::Symbol("call".into()),
            NodeValue::Node(one),
        ],
        Some(0..7),
    );
    assert!(ast.node(one).value_used());
    assert!(ast.node(one).argument());
}

fn parse(source: &str) -> ProcessedSource<'_> {
    ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap()
}

#[test]
fn recursive_literal_predicates_match_operator_and_composite_rules() {
    for source in [
        "[1, :two, 'three']",
        "{a: 1, b: 2}",
        "1..3",
        "1 == 1",
        "[1, 2] * 3",
    ] {
        let processed = parse(source);
        let node = processed.ast().unwrap();
        assert!(node.recursive_literal(), "{source}");
    }
    for source in ["[1, call]", "1 + 2", "value == 1", "{a: value}"] {
        let processed = parse(source);
        let node = processed.ast().unwrap();
        assert!(!node.recursive_literal(), "{source}");
    }
    assert!(parse("[1, :two]").ast().unwrap().recursive_basic_literal());
    assert!(parse("[[1]]").ast().unwrap().recursive_basic_literal());
}

#[test]
fn constant_and_constructor_helpers_follow_rubocop_node_patterns() {
    let processed = parse("::Namespace::Name");
    let constant = processed.ast().unwrap();
    assert_eq!(constant.const_name().as_deref(), Some("Namespace::Name"));
    for source in [
        "Class.new {}",
        "Module.new {}",
        "Struct.new(:x) {}",
        "Data.define(:x)",
    ] {
        let processed = parse(source);
        assert!(processed.ast().unwrap().class_constructor(), "{source}");
    }
    assert!(parse("Struct.new(:x) {}")
        .ast()
        .unwrap()
        .struct_constructor());
    assert!(!parse("Struct.new(:x)").ast().unwrap().struct_constructor());
}

#[test]
fn class_module_proc_lambda_and_guard_helpers_return_matched_bodies() {
    let class = parse("class Example < Parent\n  work\nend");
    let class = class.ast().unwrap();
    assert_eq!(class.kind(), "class");
    assert_eq!(class.children().len(), 3);
    assert_eq!(
        class.class_definition().unwrap().method_name(),
        Some("work")
    );
    let module = parse("module Example\n  work\nend");
    let module = module.ast().unwrap();
    assert_eq!(
        module.module_definition().unwrap().method_name(),
        Some("work")
    );
    assert!(parse("proc { work }").ast().unwrap().proc_literal());
    assert!(parse("lambda { work }").ast().unwrap().lambda());
    assert!(parse("proc { work }").ast().unwrap().lambda_or_proc());
    assert!(parse("lambda { work }").ast().unwrap().lambda_or_proc());
    assert!(parse("Example = Class.new { work }")
        .ast()
        .unwrap()
        .node_child(2)
        .unwrap()
        .new_class_or_module_block());
    assert!(parse("return if ready")
        .ast()
        .unwrap()
        .node_child(1)
        .unwrap()
        .guard_clause());
}

#[test]
fn defined_and_parent_module_names_follow_nested_scope_boundaries() {
    let nested = parse(
        "module Foo\n  class << self\n    class Bar\n      attr_reader :config\n    end\n  end\nend",
    );
    let target = nested
        .ast()
        .unwrap()
        .each_descendant(&["send"])
        .into_iter()
        .find(|node| node.method_name() == Some("attr_reader"))
        .unwrap();
    assert_eq!(
        target.parent_module_name().as_deref(),
        Some("Foo::#<Class:Foo>::Bar")
    );

    let top = parse("def config; end");
    assert_eq!(
        top.ast().unwrap().parent_module_name().as_deref(),
        Some("Object")
    );

    let unknown = parse("module Foo\n  wrapper do\n    attr_reader :config\n  end\nend");
    let target = unknown
        .ast()
        .unwrap()
        .each_descendant(&["send"])
        .into_iter()
        .find(|node| node.method_name() == Some("attr_reader"))
        .unwrap();
    assert_eq!(target.parent_module_name(), None);

    let assigned = parse("Namespace::Widget = Class.new do\n  attr_reader :config\nend");
    let root = assigned.ast().unwrap();
    assert_eq!(
        root.defined_module_name().as_deref(),
        Some("Namespace::Widget")
    );
    let (namespace, name) = root.defined_module_parts().unwrap();
    assert_eq!(
        namespace.unwrap().const_name().as_deref(),
        Some("Namespace")
    );
    assert_eq!(name, "Widget");
    let target = root
        .each_descendant(&["send"])
        .into_iter()
        .find(|node| node.method_name() == Some("attr_reader"))
        .unwrap();
    assert_eq!(
        target.parent_module_name().as_deref(),
        Some("Namespace::Widget")
    );
}
