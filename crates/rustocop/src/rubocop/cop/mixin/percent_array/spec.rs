use super::PercentArray;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

#[test]
fn bracket_correction_preserves_percent_literal_whitespace() {
    let parsed = ProcessedSource::new("%w[  one   two  ]", 3.4, None, ParserEngine::Prism).unwrap();
    let check = PercentArray::new(&parsed, "brackets", 0, false);
    let node = parsed.ast().unwrap();
    assert_eq!(check.whitespace_leading(node), "  ");
    assert_eq!(check.whitespace_between(node), "   ");
    assert_eq!(check.whitespace_trailing(node), "  ");
    assert_eq!(
        check
            .check_percent_array(node)
            .unwrap()
            .replacement
            .unwrap(),
        "[  \"one\",   \"two\"  ]"
    );
}

#[test]
fn comments_and_small_arrays_are_allowed_in_bracket_form() {
    let parsed = ProcessedSource::new(
        "[\"one\", # note\n \"two\"]",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let check = PercentArray::new(&parsed, "percent", 3, false);
    let node = parsed.ast().unwrap();
    assert!(check.comments_in_array(node));
    assert!(check.allowed_bracket_array(node));
    assert!(check.check_bracketed_array(node, 'w').is_none());
}
