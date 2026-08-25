use super::{CallbackDescriptor, Commissioner};
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};
use crate::rubocop::cop::framework::Commissioner as RuntimeCommissioner;

#[test]
fn callback_initialization_separates_restricted_send_dispatch() {
    let descriptors = vec![
        CallbackDescriptor {
            cop: "One".into(),
            callbacks_needed: vec!["on_send".into()],
            restrict_on_send: Vec::new(),
        },
        CallbackDescriptor {
            cop: "Two".into(),
            callbacks_needed: vec!["on_send".into()],
            restrict_on_send: vec!["target".into()],
        },
    ];
    let commissioner = Commissioner::initialize(
        RuntimeCommissioner::new(Vec::new()),
        descriptors,
        false,
        false,
    );
    let parsed = ProcessedSource::new("target", 3.4, None, ParserEngine::Prism).unwrap();
    assert_eq!(
        commissioner.trigger_responding_cops("on_send", parsed.ast().unwrap()),
        ["One"]
    );
    assert_eq!(
        commissioner.trigger_restricted_cops("on_send", parsed.ast().unwrap()),
        ["Two"]
    );
}

#[test]
fn absorbed_errors_keep_cop_and_node_locations() {
    let mut commissioner = Commissioner::initialize(
        RuntimeCommissioner::new(Vec::new()),
        Vec::new(),
        false,
        false,
    );
    let parsed = ProcessedSource::new("value", 3.4, None, ParserEngine::Prism).unwrap();
    let result = commissioner
        .with_cop_error_handling::<()>("Lint/Test", parsed.ast(), Err("failed".into()))
        .unwrap();
    assert!(result.is_none());
}
