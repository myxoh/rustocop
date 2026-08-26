use super::MultilineLiteralBraceLayout;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

#[test]
fn symmetrical_style_follows_the_opening_brace_geometry() {
    for (source, offense) in [
        ("[one,\n two\n]", true),
        ("[one,\n two]", false),
        ("[\n one,\n two]", true),
        ("[\n one,\n two\n]", false),
    ] {
        let parsed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
        let check = MultilineLiteralBraceLayout::new("symmetrical", &parsed);
        assert_eq!(
            check.check_brace_layout(parsed.ast().unwrap()).is_some(),
            offense
        );
    }
}

#[test]
fn empty_implicit_and_single_line_literals_are_ignored() {
    let parsed = ProcessedSource::new("[]", 3.4, None, ParserEngine::Prism).unwrap();
    let check = MultilineLiteralBraceLayout::new("new_line", &parsed);
    assert!(check.ignored_literal(parsed.ast().unwrap()));
    assert!(check.check_brace_layout(parsed.ast().unwrap()).is_none());
}
