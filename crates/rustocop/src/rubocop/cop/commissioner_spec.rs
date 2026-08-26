// Port of RuboCop 1.87.0 spec/rubocop/cop/commissioner_spec.rb.
// Spec SHA-256: 6cb506d0d78fbbcdb0621850cc986eed14f08a33bc7dd1e75661f75bfc7dec94

use std::cell::RefCell;
use std::rc::Rc;

use super::commissioner::{CallbackDescriptor, Commissioner as CallbackCommissioner};
use super::framework::{Commissioner, CopRuntime, Finding, ForceRuntime};
use super::severity::Severity;
use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

#[derive(Default)]
struct State {
    events: Vec<String>,
    findings: Vec<Finding>,
}

struct FakeCop {
    state: Rc<RefCell<State>>,
    restrictions: Vec<&'static str>,
    callbacks: Option<&'static [&'static str]>,
    panic_on_int: bool,
}

impl CopRuntime for FakeCop {
    fn name(&self) -> &str {
        "Fake/FakeCop"
    }
    fn on_new_investigation(&mut self) {
        self.state.borrow_mut().events.push("new".into());
    }
    fn on_other_file(&mut self) {
        self.state.borrow_mut().events.push("other".into());
    }
    fn on_node(&mut self, node: NodeRef<'_>) {
        if self.panic_on_int && node.kind() == "int" {
            panic!("callback failed");
        }
        self.state
            .borrow_mut()
            .events
            .push(format!("on_{}", node.kind()));
    }
    fn after_node(&mut self, node: NodeRef<'_>) {
        self.state
            .borrow_mut()
            .events
            .push(format!("after_{}", node.kind()));
    }
    fn take_findings(&mut self) -> Vec<Finding> {
        std::mem::take(&mut self.state.borrow_mut().findings)
    }
    fn callbacks_needed(&self) -> Option<&[&str]> {
        self.callbacks
    }
    fn restrict_on_send(&self) -> &[&str] {
        &self.restrictions
    }
}

fn source(code: &str) -> ProcessedSource<'_> {
    ProcessedSource::new(code, 3.4, None, ParserEngine::Prism).unwrap()
}

fn fake(state: Rc<RefCell<State>>) -> FakeCop {
    FakeCop {
        state,
        restrictions: Vec::new(),
        callbacks: None,
        panic_on_int: false,
    }
}

#[test]
fn returns_all_offenses_found_by_the_cops() {
    let state = Rc::new(RefCell::new(State::default()));
    state.borrow_mut().findings.push(Finding::new(
        "Fake/FakeCop",
        0..1,
        "bad",
        Severity::Convention,
        false,
    ));
    let mut commissioner = Commissioner::new(vec![Box::new(fake(state))]);
    let report = commissioner.investigate_report(&source("1"));
    assert_eq!(report.offenses().len(), 1);
    assert_eq!(report.cops, ["Fake/FakeCop"]);
    assert_eq!(report.offenses_per_cop.len(), 1);
    assert_eq!(report.correctors, [None]);
}

#[test]
fn traverses_ast_and_invokes_specific_callbacks() {
    static CALLBACKS: &[&str] = &["on_def", "on_int", "after_int", "after_def"];
    let state = Rc::new(RefCell::new(State::default()));
    let mut cop = fake(state.clone());
    cop.callbacks = Some(CALLBACKS);
    let mut commissioner = Commissioner::new(vec![Box::new(cop)]);
    commissioner.investigate_processed(&source("def method\n1\nend\n"));
    let events = &state.borrow().events;
    assert_eq!(events.iter().filter(|event| *event == "on_def").count(), 1);
    assert_eq!(events.iter().filter(|event| *event == "on_int").count(), 1);
    assert!(!events.contains(&"after_int".into()));
    assert_eq!(
        events.iter().filter(|event| *event == "after_def").count(),
        1
    );
}

#[test]
fn unrestricted_cops_receive_all_send_and_csend_calls() {
    static CALLBACKS: &[&str] = &["on_send", "on_csend"];
    let state = Rc::new(RefCell::new(State::default()));
    let mut cop = fake(state.clone());
    cop.callbacks = Some(CALLBACKS);
    let mut commissioner = Commissioner::new(vec![Box::new(cop)]);
    commissioner.investigate_processed(&source("foo; var = bar; var&.baz"));
    let events = &state.borrow().events;
    assert_eq!(events.iter().filter(|event| *event == "on_send").count(), 2);
    assert_eq!(
        events.iter().filter(|event| *event == "on_csend").count(),
        1
    );
}

#[test]
fn restricted_cops_receive_only_named_send_calls() {
    static CALLBACKS: &[&str] = &["on_send", "on_csend", "after_send", "after_csend"];
    let state = Rc::new(RefCell::new(State::default()));
    let mut cop = fake(state.clone());
    cop.callbacks = Some(CALLBACKS);
    cop.restrictions = vec!["bar"];
    let mut commissioner = Commissioner::new(vec![Box::new(cop)]);
    commissioner.investigate_processed(&source("foo; var = bar; var&.baz"));
    assert_eq!(state.borrow().events, ["new", "on_send", "after_send"]);
}

#[test]
fn restrictions_apply_to_both_send_and_csend() {
    static CALLBACKS: &[&str] = &["on_send", "on_csend", "after_send", "after_csend"];
    let state = Rc::new(RefCell::new(State::default()));
    let mut cop = fake(state.clone());
    cop.callbacks = Some(CALLBACKS);
    cop.restrictions = vec!["bar", "baz"];
    let mut commissioner = Commissioner::new(vec![Box::new(cop)]);
    commissioner.investigate_processed(&source("foo; var = bar; var&.baz"));
    let events = &state.borrow().events;
    assert!(events.contains(&"on_send".into()));
    assert!(events.contains(&"on_csend".into()));
    assert!(events.contains(&"after_send".into()));
    assert!(events.contains(&"after_csend".into()));
}

#[test]
fn stores_errors_raised_by_cops_with_source_location() {
    let state = Rc::new(RefCell::new(State::default()));
    let mut cop = fake(state);
    cop.panic_on_int = true;
    let mut commissioner = Commissioner::new(vec![Box::new(cop)]);
    commissioner.investigate_processed(&source("def method\n1\nend\n"));
    assert_eq!(commissioner.errors().len(), 1);
    assert_eq!(commissioner.errors()[0].line, Some(2));
    assert_eq!(commissioner.errors()[0].column, Some(0));
}

#[test]
fn raise_error_reraises_callback_failure() {
    let state = Rc::new(RefCell::new(State::default()));
    let mut cop = fake(state);
    cop.panic_on_int = true;
    let mut commissioner = Commissioner::with_runtime(vec![Box::new(cop)], vec![], true, false);
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        commissioner.investigate_processed(&source("1"));
    }))
    .is_err());
}

#[test]
fn raise_cop_error_reraises_callback_failure() {
    let state = Rc::new(RefCell::new(State::default()));
    let mut cop = fake(state);
    cop.panic_on_int = true;
    let mut commissioner = Commissioner::with_runtime(vec![Box::new(cop)], vec![], false, true);
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        commissioner.investigate_processed(&source("1"));
    }))
    .is_err());
}

struct FakeForce(Rc<RefCell<usize>>);
impl ForceRuntime for FakeForce {
    fn investigate(&mut self, _: &ProcessedSource<'_>) {
        *self.0.borrow_mut() += 1;
    }
}

#[test]
fn passes_processed_source_to_cops_and_forces() {
    let state = Rc::new(RefCell::new(State::default()));
    let force_calls = Rc::new(RefCell::new(0));
    let mut commissioner = Commissioner::with_runtime(
        vec![Box::new(fake(state.clone()))],
        vec![Box::new(FakeForce(force_calls.clone()))],
        false,
        false,
    );
    commissioner.investigate_processed(&source("1"));
    assert!(state.borrow().events.contains(&"new".into()));
    assert_eq!(*force_calls.borrow(), 1);
}

#[test]
fn invalid_source_only_invokes_on_other_file() {
    let state = Rc::new(RefCell::new(State::default()));
    let mut commissioner = Commissioner::new(vec![Box::new(fake(state.clone()))]);
    commissioner.investigate_processed(&source("("));
    assert_eq!(state.borrow().events, ["other"]);
}

#[test]
fn callback_dispatch_and_argument_invocation_expose_the_same_selected_cops() {
    static CALLBACKS: &[&str] = &["on_send"];
    let state = Rc::new(RefCell::new(State::default()));
    let mut cop = fake(state);
    cop.callbacks = Some(CALLBACKS);
    cop.restrictions = vec!["target"];
    let runtime = Commissioner::new(vec![Box::new(cop)]);
    let mut commissioner = CallbackCommissioner::initialize(
        runtime,
        vec![CallbackDescriptor {
            cop: "Fake/FakeCop".into(),
            callbacks_needed: vec!["on_send".into()],
            restrict_on_send: vec!["target".into()],
        }],
        false,
        false,
    );
    let processed = source("target");
    let node = processed.ast().unwrap();
    assert_eq!(commissioner.on_callback("on_send", node), ["Fake/FakeCop"]);
    assert_eq!(
        commissioner.invoke_with_argument("on_send", &["Fake/FakeCop".into()], &node),
        [("Fake/FakeCop".into(), "on_send".into())]
    );
    let report = commissioner.investigate(&processed);
    let views = report.cop_reports();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].cop, "Fake/FakeCop");
}
