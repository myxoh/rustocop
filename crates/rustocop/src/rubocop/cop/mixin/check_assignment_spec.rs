use super::check_assignment::{extract_rhs, CheckAssignment};
use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

#[derive(Default)]
struct Recorder {
    calls: Vec<(String, String)>,
}

impl CheckAssignment for Recorder {
    fn check_assignment(&mut self, node: NodeRef<'_>, rhs: NodeRef<'_>) {
        self.calls.push((node.kind().into(), rhs.kind().into()));
    }
}

fn parse(source: &str) -> ProcessedSource<'_> {
    ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap()
}

#[test]
fn assignment_callbacks_pass_the_assignment_expression_as_rhs() {
    let source = parse("value = call(1)");
    let node = source.ast().unwrap();
    let rhs = extract_rhs(node).unwrap();
    assert_eq!(rhs.method_name(), Some("call"));
    let mut recorder = Recorder::default();
    recorder.on_lvasgn(node);
    assert_eq!(recorder.calls, [("lvasgn".into(), "send".into())]);
}

#[test]
fn send_callback_uses_last_argument_and_ignores_non_calls() {
    let source = parse("object.value = 1");
    let node = source.ast().unwrap();
    let mut recorder = Recorder::default();
    recorder.on_send(node);
    assert_eq!(recorder.calls.len(), 1);
    assert_eq!(recorder.calls[0].1, "int");
}
