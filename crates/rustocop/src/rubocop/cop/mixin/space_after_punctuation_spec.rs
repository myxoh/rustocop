use super::space_after_punctuation::*;
use super::surrounding_space::SurroundingSpace;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};
use crate::rubocop::ast::source::{SourceBuffer, SourceRange};
use crate::rubocop::ast::token::Token;

#[test]
fn adjacent_token_scan_keeps_allowed_closers_and_right_curly_style_exceptions() {
    let parsed = ProcessedSource::new("call(one,two)", 3.4, None, ParserEngine::Prism).unwrap();
    let policy = SpaceAfterPunctuation {
        space_style_before_rcurly: "space".into(),
    };
    let offenses = policy.on_new_investigation(parsed.tokens(), |left, _| {
        (left.text == ",").then(|| "comma".into())
    });
    assert_eq!(offenses.len(), 1);
    assert_eq!(offenses[0].message, "Space missing after comma.");
    assert_eq!(policy.offset(), 1);
}

#[test]
fn surrounding_space_public_offense_and_empty_token_contracts_are_executable() {
    let spaced = SourceBuffer::new("a  b");
    let left = Token::new(SourceRange::new(&spaced, 0, 1), "tIDENTIFIER", "a");
    let right = Token::new(SourceRange::new(&spaced, 3, 4), "tIDENTIFIER", "b");
    let helper = SurroundingSpace::new(&spaced, false);
    assert_eq!(
        helper
            .no_space_offenses(
                Some(&left),
                Some(&right),
                "%{command} spaces.",
                false,
                false
            )
            .len(),
        2
    );

    let compact = SourceBuffer::new("ab");
    let left = Token::new(SourceRange::new(&compact, 0, 1), "tIDENTIFIER", "a");
    let right = Token::new(SourceRange::new(&compact, 1, 2), "tIDENTIFIER", "b");
    let helper = SurroundingSpace::new(&compact, false);
    assert_eq!(
        helper
            .space_offenses(
                Some(&left),
                Some(&right),
                "%{command} spaces.",
                false,
                false
            )
            .len(),
        2
    );
    assert!(helper.empty_brackets(&left, &right, &[left.clone(), right.clone()]));
}
