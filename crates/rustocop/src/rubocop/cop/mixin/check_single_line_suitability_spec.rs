use super::check_single_line_suitability::CheckSingleLineSuitability;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

#[test]
fn suitability_applies_length_comment_and_structural_safety_in_order() {
    let parsed = ProcessedSource::new("foo(\n  1\n)", 3.4, None, ParserEngine::Prism).unwrap();
    let check = CheckSingleLineSuitability {
        processed_source: &parsed,
        max_line_length: Some(20),
    };
    assert!(check.suitable_as_single_line(parsed.ast().unwrap()));
    assert_eq!(check.to_single_line("foo\n  &.bar"), "foo&.bar");

    let commented =
        ProcessedSource::new("foo(\n  1 # note\n)", 3.4, None, ParserEngine::Prism).unwrap();
    let check = CheckSingleLineSuitability {
        processed_source: &commented,
        max_line_length: None,
    };
    assert!(check.comment_within(commented.ast().unwrap()));
    assert!(!check.suitable_as_single_line(commented.ast().unwrap()));
}
