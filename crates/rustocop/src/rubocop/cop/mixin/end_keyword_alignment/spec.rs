use super::EndKeywordAlignment;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

#[test]
fn reports_end_columns_against_the_configured_keyword_style() {
    let parsed = ProcessedSource::new(
        "class Example\n  work\n  end",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let buffer = parsed.buffer();
    let check = EndKeywordAlignment::new("keyword", &buffer);
    let offense = check.check_end_kw_in_node(parsed.ast().unwrap()).unwrap();
    assert_eq!(
        offense.message,
        "`end` at 3, 2 is not aligned with `class` at 1, 0."
    );
    assert_eq!(check.style_parameter_name(), "EnforcedStyleAlignWith");
}

#[test]
fn accepts_an_end_at_the_keyword_column_and_extracts_the_start_line() {
    let parsed = ProcessedSource::new(
        "  class Example\n    work\n  end",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let buffer = parsed.buffer();
    let check = EndKeywordAlignment::new("keyword", &buffer);
    let node = parsed.ast().unwrap();
    assert!(check.check_end_kw_in_node(node).is_none());
    assert_eq!(
        check.start_line_range(node).unwrap().source(),
        "class Example"
    );
}

#[test]
fn variable_alignment_uses_the_rhs_line_break_for_non_keyword_styles() {
    let parsed = ProcessedSource::new("value =\n  call", 3.4, None, ParserEngine::Prism).unwrap();
    let buffer = parsed.buffer();
    let check = EndKeywordAlignment::new("variable", &buffer);
    let assignment = parsed.ast().unwrap();
    let rhs = assignment.node_child(1).unwrap();
    assert!(!check.variable_alignment(
        crate::rubocop::ast::source::SourceRange::new(
            &buffer,
            assignment.source_range().unwrap().start,
            assignment.source_range().unwrap().end,
        ),
        rhs,
        "variable",
    ));
}
