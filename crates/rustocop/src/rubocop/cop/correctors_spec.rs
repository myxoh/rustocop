use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};
use crate::rubocop::ast::source::{SourceBuffer, SourceRange};

use super::corrector::Corrector;
use super::correctors::{
    ArgumentKind, ConditionCorrector, EmptyLineCorrector, EmptyLineStyle, PunctuationCorrector,
    RequireLibraryCorrector, StringLiteralCorrector, StringStyle, UnusedArgCorrector,
};

#[test]
fn punctuation_and_empty_line_operations_match_upstream() {
    let source = SourceBuffer::new("a ,b");
    let mut corrector = Corrector::new(&source);
    PunctuationCorrector::remove_space(&mut corrector, SourceRange::new(&source, 1, 2));
    PunctuationCorrector::swap_comma(&mut corrector, Some(SourceRange::new(&source, 2, 3)));
    assert_eq!(corrector.rewrite().unwrap(), "ab");

    let source = SourceBuffer::new("a\nb");
    let mut corrector = Corrector::new(&source);
    EmptyLineCorrector::correct(
        &mut corrector,
        EmptyLineStyle::EmptyLines,
        SourceRange::new(&source, 2, 3),
    );
    assert_eq!(corrector.rewrite().unwrap(), "a\n\nb");
}

#[test]
fn string_literal_styles_and_interpolation_guard_match_upstream() {
    let source = SourceBuffer::new("\"abc\"");
    let mut corrector = Corrector::new(&source);
    StringLiteralCorrector::correct(
        &mut corrector,
        source.source_range(),
        "abc",
        false,
        StringStyle::SingleQuotes,
    );
    assert_eq!(corrector.rewrite().unwrap(), "'abc'");

    let source = SourceBuffer::new("'a\\nb'");
    let mut corrector = Corrector::new(&source);
    StringLiteralCorrector::correct(
        &mut corrector,
        source.source_range(),
        "a\nb",
        false,
        StringStyle::DoubleQuotes,
    );
    assert_eq!(corrector.rewrite().unwrap(), "\"a\\nb\"");
}

#[test]
fn unused_arguments_skip_keywords_rename_values_and_remove_blockargs() {
    let source = SourceBuffer::new("a, &block");
    let mut corrector = Corrector::new(&source);
    UnusedArgCorrector::correct(
        &mut corrector,
        ArgumentKind::Block,
        SourceRange::new(&source, 3, 9),
        SourceRange::new(&source, 4, 9),
        None,
    );
    assert_eq!(corrector.rewrite().unwrap(), "a");

    let source = SourceBuffer::new("arg");
    let mut corrector = Corrector::new(&source);
    UnusedArgCorrector::correct(
        &mut corrector,
        ArgumentKind::Positional,
        source.source_range(),
        source.source_range(),
        None,
    );
    assert_eq!(corrector.rewrite().unwrap(), "_arg");

    let source = SourceBuffer::new("a, &block");
    let mut corrector = Corrector::new(&source);
    UnusedArgCorrector::correct_for_blockarg_type(&mut corrector, SourceRange::new(&source, 3, 9));
    assert_eq!(corrector.rewrite().unwrap(), "a");
}

#[test]
fn require_and_negative_condition_rewrites_match_upstream() {
    assert_eq!(
        RequireLibraryCorrector::require_statement("set"),
        "require 'set'\n"
    );
    let source = SourceBuffer::new("unless !ready");
    let mut corrector = Corrector::new(&source);
    ConditionCorrector::correct_negative_condition(
        &mut corrector,
        SourceRange::new(&source, 0, 6),
        "if",
        SourceRange::new(&source, 7, 13),
        "ready",
    );
    assert_eq!(corrector.rewrite().unwrap(), "if ready");

    let processed =
        ProcessedSource::new("unless (!ready)\nend", 3.4, None, ParserEngine::Prism).unwrap();
    let condition = ConditionCorrector::negated_condition(processed.ast().unwrap()).unwrap();
    assert_eq!(condition.method_name(), Some("!"));
}
