use super::hash_alignment_styles::*;

fn pair(key: &str, key_col: usize, operator_col: usize, value_col: usize) -> PairLayout {
    PairLayout {
        key_source: key.into(),
        key_column: key_col,
        key_end_column: key_col + key.len(),
        pair_column: key_col,
        operator_column: operator_col,
        operator_end_column: operator_col + 2,
        value_column: value_col,
        delimiter: " => ".into(),
        first_line: key_col,
        last_line: key_col,
        hash_rocket: true,
        value_on_new_line: false,
        value_omission: false,
        begins_its_line: true,
    }
}

#[test]
fn key_alignment_uses_each_pairs_own_separator_and_value_geometry() {
    let first = pair("a", 2, 5, 9);
    let current = pair("long", 4, 11, 15);
    assert!(KeyAlignment.checkable_layout(&[first.clone(), current.clone()]));
    assert_eq!(
        KeyAlignment.deltas_for_first_pair(&first),
        Deltas {
            key: None,
            separator: Some(-1),
            value: Some(-1)
        }
    );
    assert_eq!(KeyAlignment.deltas(&first, &current).key, Some(-2));
    let mut inline = current.clone();
    inline.begins_its_line = false;
    assert_eq!(KeyAlignment.deltas(&first, &inline), Deltas::default());
}

#[test]
fn table_alignment_uses_max_key_and_delimiter_widths() {
    let mut second = pair("long", 2, 8, 12);
    second.first_line = 3;
    second.last_line = 3;
    let pairs = [pair("a", 2, 5, 9), second];
    assert!(ValueAlignment.checkable_layout(&pairs));
    assert_eq!(TableAlignment.max_key_width(&pairs), 4);
    assert_eq!(TableAlignment.max_delimiter_width(&pairs), 4);
    assert_eq!(
        TableAlignment.hash_rocket_delta(&pairs[0], &pairs[1], &pairs),
        -1
    );
}

#[test]
fn separator_and_keyword_splat_strategies_preserve_omissions_and_line_gate() {
    let first = pair("long", 2, 8, 12);
    let mut current = pair("a", 5, 8, 12);
    current.value_omission = true;
    assert_eq!(SeparatorAlignment.value_delta(&first, &current), 0);
    assert_eq!(KeywordSplatAlignment.deltas(&first, &current).key, Some(-3));
    current.begins_its_line = false;
    assert_eq!(
        KeywordSplatAlignment.deltas(&first, &current),
        Deltas::default()
    );
}
