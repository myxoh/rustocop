use super::CheckLineBreakable;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

#[test]
fn chooses_the_last_element_within_the_limit_for_parenthesized_calls() {
    let source = "method(first_argument, second_argument, third_argument)";
    let parsed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    let check = CheckLineBreakable::new(&parsed, Some(20));
    let node = parsed.ast().unwrap();
    let break_after = check.extract_breakable_node(node, 20).unwrap();
    assert_eq!(break_after.source(), Some("first_argument"));
}

#[test]
fn trailing_unbraced_hash_pairs_are_processed_as_individual_arguments() {
    let parsed =
        ProcessedSource::new("method(1, one: 2, two: 3)", 3.4, None, ParserEngine::Prism).unwrap();
    let check = CheckLineBreakable::new(&parsed, Some(10));
    let args = check.process_args(parsed.ast().unwrap().arguments());
    assert_eq!(
        args.iter().map(|node| node.kind()).collect::<Vec<_>>(),
        ["int", "pair", "pair"]
    );
}
