// Port of RuboCop 1.87.0 spec/rubocop/cop/alignment_corrector_spec.rb.
// Spec SHA-256: 6fe8e0dbea7ada21eac4412f18d552580fe672c5e9059014db321fe2666fccaa

use super::advanced_correctors::AlignmentCorrector;
use super::corrector::Corrector;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};
use crate::rubocop::ast::source::{SourceBuffer, SourceRange};

#[test]
fn positive_column_delta_indents() {
    let buffer = SourceBuffer::new("  42\n");
    let mut corrector = Corrector::new(&buffer);
    AlignmentCorrector::correct(
        &mut corrector,
        SourceRange::new(&buffer, 2, 4),
        2,
        false,
        false,
        &[],
    );
    assert_eq!(corrector.rewrite().unwrap(), "    42\n");
}

#[test]
fn alignment_column_uses_zero_without_a_target() {
    assert_eq!(AlignmentCorrector::alignment_column(None), 0);
    assert_eq!(AlignmentCorrector::alignment_column(Some(7)), 7);
}

#[test]
fn negative_column_delta_outdents() {
    let buffer = SourceBuffer::new("    42\n");
    let mut corrector = Corrector::new(&buffer);
    AlignmentCorrector::correct(
        &mut corrector,
        SourceRange::new(&buffer, 4, 6),
        -3,
        false,
        false,
        &[],
    );
    assert_eq!(corrector.rewrite().unwrap(), " 42\n");
}

fn align_parsed(source: &str, delta: isize) -> String {
    let processed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    let node = processed.ast().unwrap();
    let buffer = SourceBuffer::new(source);
    let mut corrector = Corrector::new(&buffer);
    AlignmentCorrector::correct_node(&mut corrector, &processed, Some(node), delta, false);
    corrector.rewrite().unwrap()
}

#[test]
fn plain_heredoc_bodies_and_end_markers_are_not_indented() {
    let source = "begin\n  <<DOC\na\nb\nDOC\nend\n";
    assert_eq!(
        align_parsed(source, 2),
        "  begin\n    <<DOC\na\nb\nDOC\n  end\n"
    );
}

#[test]
fn backtick_heredoc_bodies_and_end_markers_are_not_indented() {
    let source = "begin\n  <<`DOC`\na\nb\nDOC\nend\n";
    assert_eq!(
        align_parsed(source, 2),
        "  begin\n    <<`DOC`\na\nb\nDOC\n  end\n"
    );
}

#[test]
fn multiline_string_literal_contents_are_not_indented() {
    let source = "begin\n  value =\n'a\nb\nc'\nend\n";
    assert_eq!(
        align_parsed(source, 2),
        "  begin\n    value =\n  'a\nb\nc'\n  end\n"
    );
}
