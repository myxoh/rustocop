fn positive_but_not_one(value: i32) -> Option<i32> {
    return_unless!(value > 0, None);
    return_if!(value == 1, None);
    Some(value)
}

#[test]
fn rubocop_style_guards_preserve_early_return_semantics() {
    assert_eq!(positive_but_not_one(-1), None);
    assert_eq!(positive_but_not_one(1), None);
    assert_eq!(positive_but_not_one(2), Some(2));
}
