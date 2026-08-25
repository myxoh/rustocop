use super::ParenthesesCorrector;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};
use crate::rubocop::cop::corrector::Corrector;

#[test]
fn correct_removes_both_parentheses_and_repairs_ternary_adjacency() {
    let source = "(condition)? yes : no";
    let parsed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    let ternary = parsed.ast().unwrap();
    let condition = ternary.condition().unwrap();
    assert!(ParenthesesCorrector::ternary_condition(condition));
    assert!(ParenthesesCorrector::next_char_is_question_mark(condition));
    let buffer = parsed.buffer();
    let mut corrector = Corrector::new(&buffer);
    ParenthesesCorrector::correct(&mut corrector, condition);
    assert_eq!(corrector.rewrite().unwrap(), "condition ? yes : no");
}

#[test]
fn chained_close_parenthesis_detection_distinguishes_comments() {
    for (source, expected) in [
        ("foo(\n  value # note\n).bar", true),
        ("foo(\n  value\n) # note", false),
    ] {
        let parsed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
        let send = parsed
            .ast()
            .unwrap()
            .each_node(&["send"])
            .into_iter()
            .find(|node| node.parenthesized())
            .unwrap();
        assert_eq!(
            ParenthesesCorrector::chained_after_close_paren(send, &parsed.buffer()),
            expected
        );
    }
}
