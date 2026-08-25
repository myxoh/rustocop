use regex::Regex;

use super::MethodComplexity;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

fn score(_: crate::rubocop::ast::node::core::NodeRef<'_>) -> usize {
    1
}

#[test]
fn measures_counted_nodes_and_formats_the_cop_specific_message() {
    let source = "def work\n  if ready\n    run\n  end\nend";
    let parsed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    let check = MethodComplexity::new(
        1,
        &["if"],
        &[],
        &[],
        "Method %<method>s complexity is %<complexity>d/%<max>d",
        false,
        score,
    );
    let offense = check.on_def(parsed.ast().unwrap()).unwrap();
    assert_eq!(offense.complexity, 2);
    assert_eq!(offense.message, "Method work complexity is 2/1");
}

#[test]
fn allowed_names_and_define_method_blocks_follow_the_same_gates() {
    let parsed = ProcessedSource::new(
        "define_method(:generated) { if ready then run end }",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let patterns = [Regex::new("generated").unwrap()];
    let check = MethodComplexity::new(0, &["if"], &[], &patterns, "", false, score);
    let node = parsed.ast().unwrap();
    assert_eq!(check.define_method(node), Some("generated"));
    assert!(check.on_block(node).is_none());
}
