use super::{SpaceSide, SurroundingSpace};
use crate::rubocop::ast::source::{SourceBuffer, SourceRange};
use crate::rubocop::ast::token::Token;

#[test]
fn selects_exact_left_and_right_whitespace_ranges() {
    let buffer = SourceBuffer::new("name  ( value )");
    let check = SurroundingSpace::new(&buffer, false);
    let token = Token::new(SourceRange::new(&buffer, 6, 7), "tLPAREN", "(");
    assert_eq!(
        check
            .side_space_range(token.pos(), SpaceSide::Left, false)
            .source(),
        "  "
    );
    assert_eq!(
        check
            .side_space_range(token.pos(), SpaceSide::Right, false)
            .source(),
        " "
    );
}

#[test]
fn empty_spacing_predicates_and_commands_are_independent() {
    let buffer = SourceBuffer::new("[ ]");
    let left = Token::new(SourceRange::new(&buffer, 0, 1), "tLBRACK", "[");
    let right = Token::new(SourceRange::new(&buffer, 2, 3), "tRBRACK", "]");
    let check = SurroundingSpace::new(&buffer, false);
    assert!(check.space_between(&left, &right));
    assert!(!check.no_character_between(&left, &right));
    assert!(check
        .empty_offenses("space", &left, &right, "%<command>s spaces")
        .is_empty());
    assert_eq!(
        check.empty_offenses("no_space", &left, &right, "%<command>s spaces")[0].message,
        "Do not use spaces"
    );
}
