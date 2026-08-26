use super::MultilineLiteralBraceCorrector;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};
use crate::rubocop::cop::corrector::Corrector;

fn rewrite(source: &str) -> String {
    let parsed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    let node = parsed
        .ast()
        .unwrap()
        .each_node(&["array"])
        .into_iter()
        .next()
        .unwrap();
    let buffer = parsed.buffer();
    let mut corrector = Corrector::new(&buffer);
    MultilineLiteralBraceCorrector::correct(&mut corrector, node, &parsed);
    corrector.rewrite().unwrap()
}

#[test]
fn moves_braces_and_preserves_trailing_comments_and_commas() {
    for (source, expected) in [
        ("[1]", "[1\n]"),
        ("[\n  1\n]", "[\n  1]"),
        ("[\n  1, # hi\n]", "[\n  1,] # hi"),
    ] {
        assert_eq!(rewrite(source), expected);
    }
}

#[test]
fn declines_comment_moves_that_would_swallow_argument_syntax() {
    let source = "foo([\n  1 # hi\n])";
    assert_eq!(rewrite(source), source);
}
