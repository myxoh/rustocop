use super::interpolation::Interpolation;
use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

#[derive(Default)]
struct Recorder(Vec<String>);

impl Interpolation for Recorder {
    fn on_interpolation(&mut self, node: NodeRef<'_>) {
        self.0.push(node.source().unwrap_or_default().into());
    }
}

#[test]
fn every_interpolated_literal_callback_visits_only_immediate_begin_children() {
    for source in [
        r#""before #{one} after #{two}""#,
        r#"%x(echo #{one})"#,
        r#":"value_#{one}""#,
        r#"/a#{one}/"#,
    ] {
        let parsed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
        let node = parsed
            .ast()
            .unwrap_or_else(|| panic!("failed to parse {source}"));
        let mut recorder = Recorder::default();
        match node.kind() {
            "dstr" => recorder.on_dstr(node),
            "xstr" => recorder.on_xstr(node),
            "dsym" => recorder.on_dsym(node),
            "regexp" => recorder.on_regexp(node),
            other => panic!("{other}"),
        }
        assert!(!recorder.0.is_empty());
    }
}
