use super::CodeLength;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

#[test]
fn skips_blank_and_comment_lines_and_reports_the_calculated_max() {
    let source = "def long\n  one\n\n  # note\n  two\nend\n";
    let parsed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    let check = CodeLength::new("Method", 1, false, Vec::new(), false);
    let offense = check
        .check_code_length(parsed.ast().unwrap(), &parsed)
        .unwrap()
        .unwrap();
    assert_eq!(offense.length, 2);
    assert_eq!(offense.message, "Method has too many lines. [2/1]");
    assert!(check.irrelevant_line("  # note"));
}

#[test]
fn lsp_location_ends_at_the_definition_name() {
    let source = "def example\n  work\nend";
    let parsed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    let node = parsed.ast().unwrap();
    let regular = CodeLength::new("Method", 0, true, Vec::new(), false);
    let lsp = CodeLength::new("Method", 0, true, Vec::new(), true);
    assert_eq!(regular.location(node).unwrap(), 0..source.len());
    assert_eq!(lsp.location(node).unwrap(), 0..11);
}
