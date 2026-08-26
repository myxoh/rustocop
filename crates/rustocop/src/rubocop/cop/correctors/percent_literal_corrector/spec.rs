use std::collections::BTreeMap;

use super::PercentLiteralCorrector;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};
use crate::rubocop::cop::corrector::Corrector;

fn translation() -> PercentLiteralCorrector {
    PercentLiteralCorrector::initialize(BTreeMap::from([("default".into(), "[]".into())]), None)
}

fn rewrite(source: &str, character: char) -> String {
    let parsed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    let buffer = parsed.buffer();
    let mut corrector = Corrector::new(&buffer);
    translation().correct(&mut corrector, parsed.ast().unwrap(), character);
    corrector.rewrite().unwrap()
}

#[test]
fn corrects_single_and_multiline_words_with_cached_delimiters() {
    for (source, character, expected) in [
        ("[\"one\", \"two\"]", 'w', "%w[one two]"),
        ("[\n  \"one\",\n  \"two\"\n]", 'w', "%w[\n  one\n  two\n]"),
        ("[:one, :two]", 'i', "%i[one two]"),
    ] {
        assert_eq!(rewrite(source, character), expected);
    }
}

#[test]
fn escaping_and_balanced_delimiter_rules_match_rubocop() {
    let translation = translation();
    assert_eq!(
        translation.substitute_escaped_delimiters("(balanced)".into(), ('(', ')')),
        "(balanced)"
    );
    assert_eq!(
        translation.substitute_escaped_delimiters("unbalanced)".into(), ('(', ')')),
        "unbalanced\\)"
    );
    assert_eq!(
        translation.end_content("[\n  word\n  ]"),
        Some("\n  ".into())
    );
}
