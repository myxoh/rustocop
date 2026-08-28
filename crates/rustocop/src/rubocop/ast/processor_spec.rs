// Behavioral port of ast 2.4.3's AST::Processor::Mixin examples.
// Upstream spec: spec/ast_spec.rb
// Spec MD5: 926aeb0be45f717fce7acff22dce7374

use super::node::core::{Ast, NodeId, NodeValue};
use super::processor::Processor;

#[derive(Default)]
struct ArithmeticProcessor {
    missing: Vec<String>,
}

impl Processor for ArithmeticProcessor {
    fn on_array(&mut self, ast: &mut Ast, node: NodeId) -> Option<NodeId> {
        let children = ast
            .node(node)
            .child_nodes()
            .into_iter()
            .map(|child| child.id())
            .collect::<Vec<_>>();
        let children = self
            .process_all(ast, &children)
            .into_iter()
            .map(NodeValue::Node)
            .collect();
        Some(ast.updated(node, None, Some(children)))
    }

    fn on_int(&mut self, ast: &mut Ast, node: NodeId) -> Option<NodeId> {
        let value = ast.node(node).integer_child(0)?;
        (value == 2).then(|| ast.updated(node, None, Some(vec![NodeValue::Integer(4)])))
    }

    fn handler_missing(&mut self, ast: &mut Ast, node: NodeId) -> Option<NodeId> {
        self.missing.push(ast.node(node).kind().to_string());
        None
    }
}

#[test]
fn nil_and_missing_handlers_preserve_the_upstream_process_contract() {
    let mut ast = Ast::new("");
    let unknown = ast.add_node("future", Vec::new(), None);
    let mut processor = ArithmeticProcessor::default();

    assert_eq!(processor.process(&mut ast, None), None);
    assert_eq!(processor.process(&mut ast, Some(unknown)), Some(unknown));
    assert_eq!(processor.missing, ["future"]);
}

#[test]
fn callbacks_and_process_all_propagate_rewritten_children_to_the_parent() {
    let mut ast = Ast::new("[1, 2]");
    let one = ast.add_node("int", vec![NodeValue::Integer(1)], Some(1..2));
    let two = ast.add_node("int", vec![NodeValue::Integer(2)], Some(4..5));
    let array = ast.add_node(
        "array",
        vec![NodeValue::Node(one), NodeValue::Node(two)],
        Some(0..6),
    );
    let mut processor = ArithmeticProcessor::default();

    let rewritten = processor.process(&mut ast, Some(array)).unwrap();
    let children = ast.node(rewritten).child_nodes();
    assert_eq!(children[0].integer_child(0), Some(1));
    assert_eq!(children[1].integer_child(0), Some(4));
    assert_eq!(ast.node(rewritten).source_range(), Some(0..6));
    assert_eq!(processor.missing, Vec::<String>::new());
}
