// Port of RuboCop 1.87.0 spec/rubocop/cop/variable_force_spec.rb.
// Spec SHA-256: d716261b8b3bb32d910dcf82b1e6c74fe7139e619e718788e8ee270d446dc582

use super::framework::{scan_variables, VariableTable};
use super::variable_force::{VariableForce, VariableForceHandler};
use crate::rubocop::ast::node::core::{Ast, NodeValue};
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn processing_an_undeclared_local_variable_does_not_fail() {
    let mut ast = Ast::new("foo");
    let local = ast.add_node("lvar", vec![NodeValue::Symbol("foo".into())], Some(0..3));
    ast.complete(local);
    assert!(scan_variables(ast.node(local)).variables().is_empty());
}

fn scan(source: &str) {
    let processed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    if let Some(root) = processed.ast() {
        let _ = scan_variables(root);
    }
}

#[test]
fn processing_an_empty_regexp_does_not_fail() {
    scan("// =~ \"\"");
}

#[test]
fn processing_a_regexp_with_regopt_does_not_fail() {
    scan("/\\x82/n =~ \"a\"");
}

#[test]
fn processing_a_multiline_capture_regexp_does_not_fail() {
    scan("/(\n pattern\n)/ =~ string\n");
}

#[test]
fn assignment_rhs_is_scanned_before_the_new_assignment() {
    let processed =
        ProcessedSource::new("foo = 1\nfoo = foo + 1\n", 3.4, None, ParserEngine::Prism).unwrap();
    let table = scan_variables(processed.ast().unwrap());
    let foo = table
        .variables()
        .into_iter()
        .find(|variable| variable.name == "foo")
        .unwrap();
    assert_eq!(foo.assignments.len(), 2);
    assert_eq!(foo.references.len(), 1);
    assert!(foo.assignment_used(0));
    assert!(!foo.assignment_used(1));
}

#[test]
fn blocks_capture_outer_variables_but_method_scopes_do_not() {
    let processed = ProcessedSource::new(
        "outer = 1\nitems.each { outer }\ndef call\n  outer\nend\n",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let table = scan_variables(processed.ast().unwrap());
    let outer = table
        .variables()
        .into_iter()
        .find(|variable| variable.name == "outer")
        .unwrap();
    assert_eq!(outer.references.len(), 1);
    assert!(outer.captured_by_block());
}

#[test]
fn argument_kinds_are_declared_in_their_method_scope() {
    let processed = ProcessedSource::new(
        "def call(positional, keyword:, optional: 1, &block)\n  positional\n  keyword\nend",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let table = scan_variables(processed.ast().unwrap());
    let variables = table.variables();
    assert!(variables
        .iter()
        .any(|variable| variable.name == "positional" && variable.argument()));
    assert!(variables
        .iter()
        .any(|variable| variable.name == "keyword" && variable.keyword_argument()));
    assert!(variables
        .iter()
        .any(|variable| variable.name == "optional" && variable.keyword_argument()));
    assert!(variables
        .iter()
        .any(|variable| variable.name == "block" && variable.argument()));
}

struct RecordingHandler(Rc<RefCell<Vec<String>>>);

impl VariableForceHandler for RecordingHandler {
    fn on_variable_force_event(&mut self, hook: &str, _table: &VariableTable) {
        self.0.borrow_mut().push(hook.into());
    }
}

#[test]
fn handlers_receive_the_public_force_lifecycle() {
    let processed = ProcessedSource::new("value = 1", 3.4, None, ParserEngine::Prism).unwrap();
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut force = VariableForce::new();
    force.add_handler(Box::new(RecordingHandler(events.clone())));
    force.investigate(processed.ast().unwrap());
    assert_eq!(
        events.borrow().first().map(String::as_str),
        Some("before_entering_scope")
    );
    assert_eq!(
        events.borrow().last().map(String::as_str),
        Some("after_leaving_scope")
    );
}
