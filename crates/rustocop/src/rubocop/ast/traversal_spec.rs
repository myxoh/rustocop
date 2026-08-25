// Consolidated behavioral port from rubocop-ast 1.49.1:
// spec/rubocop/ast/traversal_spec.rb
// Spec SHA-256: 83cfa5999a351257de34b5d8f8be06780d03900760cc3d2418b7fa44278f9424

use super::node::core::{Ast, NodeRef, NodeValue};
use super::processed_source::{ParserEngine, ProcessedSource};
use super::traversal::{validate_shape, Traversal, KNOWN_NODE_TYPES};

#[derive(Default)]
struct Visitor {
    calls: Vec<String>,
}
impl Traversal for Visitor {
    fn on_arg(&mut self, node: NodeRef<'_>) {
        self.calls.push(format!("arg:{:?}", node.symbol_child(0)));
        self.walk_children(node);
    }
    fn on_block_pass(&mut self, node: NodeRef<'_>) {
        self.calls
            .push(format!("block_pass:{}", node.child_nodes().len()));
        self.walk_children(node);
    }
    fn on_itblock(&mut self, node: NodeRef<'_>) {
        self.calls.push("itblock".into());
        self.walk_children(node);
    }
    fn on_int(&mut self, node: NodeRef<'_>) {
        self.calls.push("int".into());
        self.walk_children(node);
    }
}

#[test]
fn invokes_overridden_callbacks_and_recurses_in_depth_first_order() {
    let mut ast = Ast::new("");
    let arg = ast.add_node("arg", vec![NodeValue::Symbol("x".into())], None);
    let integer = ast.add_node("int", vec![NodeValue::Integer(42)], None);
    let pass = ast.add_node("block_pass", vec![NodeValue::Node(arg)], None);
    let block = ast.add_node(
        "itblock",
        vec![NodeValue::Node(pass), NodeValue::Node(integer)],
        None,
    );
    let mut visitor = Visitor::default();
    visitor.walk(Some(ast.node(block)));
    assert_eq!(
        visitor.calls,
        ["itblock", "block_pass:1", "arg:Some(\"x\")", "int"]
    );
}

#[test]
fn invokes_argument_callback_for_each_argument() {
    let mut ast = Ast::new("");
    let first = ast.add_node("arg", vec![NodeValue::Symbol("x".into())], None);
    let second = ast.add_node("arg", vec![NodeValue::Symbol("y".into())], None);
    let args = ast.add_node(
        "args",
        vec![NodeValue::Node(first), NodeValue::Node(second)],
        None,
    );
    let mut visitor = Visitor::default();
    visitor.walk(Some(ast.node(args)));
    assert_eq!(visitor.calls, ["arg:Some(\"x\")", "arg:Some(\"y\")"]);
}

#[test]
fn invokes_block_pass_callback_for_anonymous_and_named_forwarding() {
    let mut ast = Ast::new("");
    let named = ast.add_node("lvar", vec![NodeValue::Symbol("block".into())], None);
    let anonymous = ast.add_node("block_pass", Vec::new(), None);
    let named_pass = ast.add_node("block_pass", vec![NodeValue::Node(named)], None);
    let root = ast.add_node(
        "begin",
        vec![NodeValue::Node(anonymous), NodeValue::Node(named_pass)],
        None,
    );
    let mut visitor = Visitor::default();
    visitor.walk(Some(ast.node(root)));
    assert_eq!(visitor.calls, ["block_pass:0", "block_pass:1"]);
}

#[test]
fn nil_walk_is_a_noop_and_unknown_nodes_use_generic_recursion() {
    let mut ast = Ast::new("");
    let integer = ast.add_node("int", vec![NodeValue::Integer(1)], None);
    let future = ast.add_node("future", vec![NodeValue::Node(integer)], None);
    let mut visitor = Visitor::default();
    visitor.walk(None);
    visitor.walk(Some(ast.node(future)));
    assert_eq!(visitor.calls, ["int"]);
}

#[test]
fn knows_every_parser_node_type_in_the_pinned_version() {
    assert_eq!(KNOWN_NODE_TYPES.len(), 137);
    for kind in [
        "send",
        "csend",
        "itblock",
        "match_pattern_p",
        "forwarded_kwrestarg",
        "__ENCODING__",
    ] {
        assert!(KNOWN_NODE_TYPES.contains(&kind));
    }
}

#[test]
fn debug_shape_validation_rejects_too_few_and_too_many_literal_children() {
    let mut ast = Ast::new("");
    let few = ast.add_node("int", Vec::new(), None);
    let many = ast.add_node(
        "int",
        vec![NodeValue::Integer(1), NodeValue::Integer(2)],
        None,
    );
    let valid = ast.add_node("int", vec![NodeValue::Integer(1)], None);
    assert_eq!(validate_shape(ast.node(few)).unwrap_err().actual, 0);
    assert_eq!(validate_shape(ast.node(many)).unwrap_err().actual, 2);
    assert!(validate_shape(ast.node(valid)).is_ok());
}

#[test]
fn traverses_every_node_in_the_upstream_code_example_corpus() {
    #[derive(Default)]
    struct Counter(usize);
    impl Traversal for Counter {
        fn on_node(&mut self, _node: NodeRef<'_>) {
            self.0 += 1;
        }
    }

    let examples = include_str!("../../../../../spec/upstream/rubocop-ast-1.49.1/spec/rubocop/ast/fixtures/code_examples.rb");
    let mut checked = 0;
    for example in examples.split("#----\n") {
        let source = format!("foo=bar=baz=nil; {example}");
        let processed = ProcessedSource::new(&source, 3.4, None, ParserEngine::Prism).unwrap();
        let Some(root) = processed.ast() else {
            continue;
        };
        let expected = root.each_node(&[]).len();
        let mut counter = Counter::default();
        counter.walk(Some(root));
        assert_eq!(counter.0, expected, "failed to traverse {example}");
        checked += 1;
    }
    assert!(
        checked > 700,
        "the complete upstream traversal corpus must be exercised"
    );
}
