use super::LambdaLiteralToMethodCorrector;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};
use crate::rubocop::cop::corrector::Corrector;

fn rewrite(source: &str) -> String {
    let parsed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    let block = parsed.ast().unwrap();
    let translation = LambdaLiteralToMethodCorrector::initialize(block).unwrap();
    let buffer = parsed.buffer();
    let mut corrector = Corrector::new(&buffer);
    translation.call(&mut corrector);
    corrector.rewrite().unwrap()
}

#[test]
fn translates_lambda_selector_arguments_and_delimiters_in_source_order() {
    for (source, expected) in [
        ("->(x) do x end", "lambda do |x| x end"),
        ("-> { 1 }", "lambda { 1 }"),
        (
            "->(first, second) { first }",
            "lambda { |first, second| first }",
        ),
    ] {
        assert_eq!(rewrite(source), expected);
    }
}

#[test]
fn source_shaped_position_helpers_track_the_prism_adapter() {
    let parsed = ProcessedSource::new("->(x) do x end", 3.4, None, ParserEngine::Prism).unwrap();
    let translation = LambdaLiteralToMethodCorrector::initialize(parsed.ast().unwrap()).unwrap();
    assert_eq!(translation.arguments_begin_pos(), Some(2));
    assert_eq!(translation.arguments_end_pos(), Some(5));
    assert_eq!(translation.selector_end(), Some(2));
    assert_eq!(translation.lambda_arg_string(), "x");
    assert!(!translation.arg_to_unparenthesized_call());
}
