use super::MultilineElementIndentation;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};
use crate::rubocop::ast::source::SourceRange;

#[test]
fn detects_consistent_and_brace_aligned_base_columns() {
    let parsed = ProcessedSource::new("  [\n    one\n  ]", 3.4, None, ParserEngine::Prism).unwrap();
    let buffer = parsed.buffer();
    let node = parsed.ast().unwrap();
    let first = node.first_node().unwrap();
    let opening = node.loc("begin").unwrap();
    let opening = SourceRange::new(&buffer, opening.0.start, opening.0.end);
    let check = MultilineElementIndentation::new(&buffer, "consistent", "align_braces", 2);
    let result = check.check_first(first, opening, None, 0);
    assert!(result.ambiguous);
    assert!(result.styles.contains(&"consistent".into()));
}

#[test]
fn extracts_nested_argument_literals_opened_on_the_parenthesis_line() {
    let parsed = ProcessedSource::new("call([\n  one\n])", 3.4, None, ParserEngine::Prism).unwrap();
    let buffer = parsed.buffer();
    let check = MultilineElementIndentation::new(&buffer, "consistent", "align_braces", 2);
    let nodes = check.each_argument_node(parsed.ast().unwrap(), "array");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].0.kind(), "array");
}
