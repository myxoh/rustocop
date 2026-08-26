use super::alignment::*;

fn item(range: std::ops::Range<usize>, line: usize, column: usize) -> AlignmentItem {
    AlignmentItem {
        source_range: range,
        line,
        column,
        begins_its_line: true,
    }
}

#[test]
fn bad_alignment_checks_only_the_first_item_per_line_and_records_column_delta() {
    let mut alignment = Alignment::new(vec!["  one".into(), "    two".into()], None, None);
    let offenses = alignment.check_alignment(&[item(2..5, 1, 2), item(6..9, 2, 4)], None);
    assert_eq!(offenses.len(), 1);
    assert_eq!(alignment.column_delta, -2);
    assert_eq!(alignment.configured_indentation_width(), 2);
    assert_eq!(alignment.indentation(&item(0..1, 1, 2)), "    ");
}

#[test]
fn containment_and_display_width_follow_source_ranges_and_unicode_columns() {
    let alignment = Alignment::new(vec!["é value".into()], Some(4), None);
    assert_eq!(alignment.display_column(&item(0..1, 1, 1)), 1);
    assert!(alignment.within(&(2..3), &(1..4)));
    assert!(!alignment.end_of_line_comment(1));
}
