// Ported from RuboCop 1.87.0:
// spec/rubocop/cop/range_help_spec.rb
// Spec SHA-256: f135c6ddb60bc97317bd5fa023ebcfbf8b9e0999440073e2a981669e5a41f28b

use super::range_help::{RangeHelp, Side, SurroundingSpace};
use crate::rubocop::ast::source::{SourceBuffer, SourceRange};

#[test]
fn range_with_surrounding_comma_works_with_each_side() {
    let buffer = SourceBuffer::new("raise \" ,Error, \"");
    let helper = RangeHelp::new(&buffer);
    let input = SourceRange::new(&buffer, 9, 14);

    assert_eq!(
        helper
            .range_with_surrounding_comma(input, Side::Both)
            .source(),
        ",Error,"
    );
    assert_eq!(
        helper
            .range_with_surrounding_comma(input, Side::Left)
            .source(),
        ",Error"
    );
    assert_eq!(
        helper
            .range_with_surrounding_comma(input, Side::Right)
            .source(),
        "Error,"
    );
}

#[test]
fn range_with_surrounding_space_works_with_each_side() {
    let buffer = SourceBuffer::new("f {  a(2) }");
    let helper = RangeHelp::new(&buffer);
    let input = SourceRange::new(&buffer, 5, 9);

    for (side, expected) in [
        (Side::Both, "  a(2) "),
        (Side::Left, "  a(2)"),
        (Side::Right, "a(2) "),
    ] {
        assert_eq!(
            helper
                .range_with_surrounding_space(
                    input,
                    SurroundingSpace {
                        side,
                        ..SurroundingSpace::default()
                    },
                )
                .source(),
            expected
        );
    }
}

#[test]
fn range_with_surrounding_space_matches_continuation_options() {
    let buffer = SourceBuffer::new("call  \\\n  argument");
    let helper = RangeHelp::new(&buffer);
    let input = SourceRange::new(&buffer, 10, 18);

    assert_eq!(
        helper
            .range_with_surrounding_space(
                input,
                SurroundingSpace {
                    side: Side::Left,
                    continuations: true,
                    ..SurroundingSpace::default()
                },
            )
            .source(),
        "\\\n  argument"
    );
}

#[test]
fn range_by_whole_lines_matches_upstream_cases() {
    let source = "puts 'example'\nputs 'another example'\n\nsomething_else\n";
    let buffer = SourceBuffer::new(source);
    let helper = RangeHelp::new(&buffer);
    let cases = [
        (5..14, "puts 'example'"),
        (0..14, "puts 'example'"),
        (0..15, "puts 'example'\nputs 'another example'"),
        (14..14, "puts 'example'"),
        (15..15, "puts 'another example'"),
        (5..28, "puts 'example'\nputs 'another example'"),
        (5..43, source.trim_end()),
    ];

    for (range, expected) in cases {
        let input = SourceRange::new(&buffer, range.start, range.end);
        assert_eq!(helper.range_by_whole_lines(input, false).source(), expected);
        assert_eq!(
            helper.range_by_whole_lines(input, true).source(),
            format!("{expected}\n")
        );
    }
}

#[test]
fn range_by_whole_lines_does_not_extend_past_end_of_source() {
    let source = "example\nwith\nno\nnewline_at_end";
    let buffer = SourceBuffer::new(source);
    let helper = RangeHelp::new(&buffer);
    let start = source.find("line_at_e").unwrap();
    let input = SourceRange::new(&buffer, start, start + "line_at_e".len());

    let output = helper.range_by_whole_lines(input, true);
    assert_eq!(output.source(), "newline_at_end");
    assert_eq!(output.end_pos(), source.len());
}

#[test]
fn source_range_and_column_helpers_match_rubocop_contracts() {
    let buffer = SourceBuffer::new("first\nsecond\n");
    let helper = RangeHelp::new(&buffer);
    assert_eq!(helper.range_between(1, 4).source(), "irs");
    assert_eq!(helper.source_range(&buffer, 2, 1, 3).source(), "eco");
    assert_eq!(
        helper.source_range_columns(&buffer, 2, 1..4).source(),
        "eco"
    );

    let left = helper.source_range(&buffer, 2, 1, 1);
    let right = helper.source_range(&buffer, 2, 4, 1);
    assert_eq!(helper.column_offset_between(right, left), 3);
}

#[test]
fn descending_source_range_columns_match_rubocops_empty_range_contract() {
    let buffer = SourceBuffer::new("first\n  second\n");
    let helper = RangeHelp::new(&buffer);

    let range = helper.source_range_columns(&buffer, 2, 2..0);

    assert_eq!(range.source(), "");
    assert_eq!((range.begin_pos(), range.end_pos()), (8, 6));
    assert_eq!(range.len(), 0);
}

#[test]
fn source_ranges_use_parser_character_offsets_for_unicode() {
    let buffer = SourceBuffer::new("éclair\n🍒 pie\n");
    let helper = RangeHelp::new(&buffer);

    assert_eq!(helper.source_range(&buffer, 1, 0, 1).source(), "é");
    assert_eq!(helper.source_range(&buffer, 2, 0, 1).source(), "🍒");
    assert_eq!(helper.source_range(&buffer, 2, 2, 3).source(), "pie");
}

#[test]
fn contents_arguments_ranges_and_bom_columns_match_rubocop_helpers() {
    let buffer = SourceBuffer::new("\u{feff}[é, two]");
    let helper = RangeHelp::new(&buffer);
    let begin = SourceRange::new(&buffer, 1, 2);
    let first = SourceRange::new(&buffer, 2, 3);
    let last = SourceRange::new(&buffer, 5, 8);
    let end = SourceRange::new(&buffer, 8, 9);

    assert_eq!(helper.contents_range(begin, end).source(), "é, two");
    assert_eq!(helper.arguments_range(first, last).source(), "é, two");
    assert_eq!(helper.add_range(last, first).source(), "é, two");
    assert_eq!(helper.effective_column(first), 1);
}

#[test]
fn range_with_comments_and_lines_matches_upstream_association_contract() {
    let source = "class A\n  # foo 1\n  def foo\n    # foo 2\n  end\n\n  # bar 1\n  def bar\n    # bar 2\n  end\n\n  # baz 1\n  def baz\n    # baz 2\n  end\nend\n";
    let buffer = SourceBuffer::new(source);
    let helper = RangeHelp::new(&buffer);
    let node_start = source[..source.find("def bar").unwrap()].chars().count();
    let node_end_byte = source.find("  end\n\n  # baz").unwrap() + "  end".len();
    let node_end = source[..node_end_byte].chars().count();
    let comment_start = source[..source.find("# bar 1").unwrap()].chars().count();
    let comment_end_byte = source.find("# bar 2").unwrap() + "# bar 2".len();
    let comment_end = source[..comment_end_byte].chars().count();
    let node = SourceRange::new(&buffer, node_start, node_end);
    let comments = [
        SourceRange::new(&buffer, comment_start, comment_start + "# bar 1".len()),
        SourceRange::new(&buffer, comment_end - "# bar 2".len(), comment_end),
    ];

    assert_eq!(
        helper
            .range_with_comments_and_lines(node, &comments)
            .source(),
        "  # bar 1\n  def bar\n    # bar 2\n  end\n"
    );
}
