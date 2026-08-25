// Ported from rubocop-ast 1.49.1:
// spec/rubocop/ast/token_spec.rb
// Spec SHA-256: 46c1f07eb32401449aafa28a2d3d72ccf5dcfe1035ebfea6a87e0f5fe01afa9d

use super::source::{SourceBuffer, SourceRange};
use super::token::Token;

#[test]
fn exposes_position_text_type_debugging_and_spacing() {
    let buffer = SourceBuffer::new("é [ 1 ]");
    let token = Token::new(SourceRange::new(&buffer, 2, 3), "tLBRACK", "[");
    assert_eq!(
        Token::initialize(SourceRange::new(&buffer, 2, 3), "tLBRACK", "["),
        token
    );
    assert_eq!(
        Token::from_parser_token(("tLBRACK", ("[", SourceRange::new(&buffer, 2, 3)))),
        token
    );
    assert_eq!(token.pos().source(), "[");
    assert_eq!(token.kind(), "tLBRACK");
    assert_eq!(token.token_type(), "tLBRACK");
    assert_eq!(token.text(), "[");
    assert_eq!(token.line(), 1);
    assert_eq!(token.column(), 2);
    assert_eq!(token.begin_pos(), 2);
    assert_eq!(token.end_pos(), 3);
    assert!(token.space_before());
    assert!(token.space_after());
    assert_eq!(token.to_string(), "[[1, 2], tLBRACK, \"[\"]");
    assert_eq!(token.display(), token.to_string());

    let left = SourceRange::new(&buffer, 0, 4);
    let right = SourceRange::new(&buffer, 2, 6);
    assert_eq!(left.intersect(right).source(), "[ ");
}

#[test]
#[allow(clippy::cognitive_complexity)] // The macro enumerates all upstream token predicates.
fn matches_every_upstream_type_predicate() {
    let buffer = SourceBuffer::new("x");
    let range = SourceRange::new(&buffer, 0, 1);
    macro_rules! assert_predicate {
        ($kind:literal, $method:ident) => {
            assert!(Token::new(range, $kind, "x").$method(), $kind);
            assert!(!Token::new(range, "other", "x").$method(), $kind);
        };
    }
    assert_predicate!("tCOMMENT", comment);
    assert_predicate!("tSEMI", semicolon);
    assert_predicate!("tLBRACK", left_array_bracket);
    assert_predicate!("tLBRACK2", left_ref_bracket);
    assert_predicate!("tRBRACK", right_bracket);
    assert_predicate!("tLBRACE", left_brace);
    assert_predicate!("tLCURLY", left_curly_brace);
    assert_predicate!("tLAMBEG", left_curly_brace);
    assert_predicate!("tRCURLY", right_curly_brace);
    assert_predicate!("tLPAREN", left_parens);
    assert_predicate!("tLPAREN2", left_parens);
    assert_predicate!("tRPAREN", right_parens);
    assert_predicate!("tCOMMA", comma);
    assert_predicate!("tDOT", dot);
    assert_predicate!("tDOT2", regexp_dots);
    assert_predicate!("tDOT3", regexp_dots);
    assert_predicate!("kRESCUE_MOD", rescue_modifier);
    assert_predicate!("kEND", end);
    assert_predicate!("tEQL", equal_sign);
    assert_predicate!("tOP_ASGN", equal_sign);
    assert_predicate!("tNL", new_line);
    assert!(Token::new(range, "tLBRACK", "[").left_bracket());
    assert!(Token::new(range, "tLBRACK2", "[").left_bracket());
}
