// Ported from RuboCop 1.87.0:
// spec/rubocop/cop/offense_spec.rb
// Spec SHA-256: 7b71096ee4ce80463e77b7b82e44db00ca2c9bbbec1e6c53242b0bf386deea23

use super::offense::{Offense, OffenseStatus};
use super::severity::Severity;
use crate::rubocop::ast::source::{SourceBuffer, SourceRange};
use crate::rubocop::cop::corrector::Corrector;

#[test]
fn exposes_attributes_statuses_highlight_and_debug_output() {
    let buffer = SourceBuffer::new("a\n");
    let offense = Offense::new(
        Severity::Convention,
        SourceRange::new(&buffer, 0, 1),
        "message",
        "CopName",
        OffenseStatus::Corrected,
        Some(Corrector::new(&buffer)),
    );
    assert_eq!(offense.severity(), Severity::Convention);
    assert_eq!(offense.line(), 1);
    assert_eq!(offense.column(), 0);
    assert_eq!(offense.message(), "message");
    assert_eq!(offense.cop_name(), "CopName");
    assert_eq!(offense.status(), OffenseStatus::Corrected);
    assert!(offense.has_corrector());
    assert!(offense.corrector().is_some());
    assert!(offense.correctable());
    assert!(offense.corrected());
    assert_eq!(offense.highlighted_source(), "a");
    assert_eq!(offense.highlighted_area(), 0..1);
    assert_eq!(offense.size(), 1);
    assert_eq!(offense.length(), 1);
    assert_eq!(offense.display(), "C:  1:  1: message");
    assert_eq!(offense.to_string(), "C:  1:  1: message");

    let restored = Offense::marshal_load(offense.marshal_dump());
    assert!(offense.equivalent(&restored));
    assert_eq!(offense.compare(&restored), std::cmp::Ordering::Equal);
}

#[test]
fn status_predicates_match_every_rubocop_status_branch() {
    let buffer = SourceBuffer::new("a");
    let offense = |status| {
        Offense::new(
            Severity::Warning,
            SourceRange::new(&buffer, 0, 1),
            "message",
            "Cop",
            status,
            None,
        )
    };
    assert!(!offense(OffenseStatus::Unsupported).correctable());
    assert!(offense(OffenseStatus::CorrectedWithTodo).corrected());
    assert!(offense(OffenseStatus::CorrectedWithTodo).corrected_with_todo());
    assert!(offense(OffenseStatus::Disabled).disabled());
    assert!(offense(OffenseStatus::Todo).disabled());
    assert!(!offense(OffenseStatus::Uncorrected).corrected());
}

#[test]
fn compares_by_the_same_ordered_attribute_tuple() {
    let buffer = SourceBuffer::new("aaaaaa\nbbbbbb\ncccccc\ndddddd\neeeeee\nffffff");
    let offense = |line: usize, column: usize, cop: &str| {
        let begin = buffer.line_start(line) + column;
        Offense::new(
            Severity::Convention,
            SourceRange::new(&buffer, begin, begin + 1),
            "message",
            cop,
            OffenseStatus::Uncorrected,
            None,
        )
    };
    assert_eq!(offense(5, 5, "CopName"), offense(5, 5, "CopName"));
    assert!(offense(6, 5, "CopName") > offense(5, 5, "CopName"));
    assert!(offense(5, 6, "CopName") > offense(5, 5, "CopName"));
    assert!(offense(5, 5, "B") > offense(5, 5, "A"));
}

#[test]
fn highlights_first_line_and_supports_no_location() {
    let buffer = SourceBuffer::new("def foo\n  something\n  something_else\nend\n");
    let multiline = Offense::new(
        Severity::Convention,
        buffer.source_range(),
        "message % test",
        "CopName",
        OffenseStatus::Corrected,
        None,
    );
    assert_eq!(multiline.highlighted_source(), "def foo");
    assert!(multiline.to_string().ends_with("message % test"));

    let none = Offense::no_location(
        Severity::Convention,
        "message",
        "CopName",
        OffenseStatus::Uncorrected,
    );
    assert_eq!(none.location(), None);
    assert_eq!(none.line(), 1);
    assert_eq!(none.column(), 0);
    assert_eq!(none.source_line(), "");
    assert_eq!(none.column_length(), 0);
    assert_eq!(none.first_line(), 1);
    assert_eq!(none.last_line(), 1);
    assert_eq!(none.last_column(), 0);
    assert_eq!(none.column_range(), 0..0);
    assert_eq!(none.real_column(), 1);
    assert_eq!(none.highlighted_source(), "");
}
