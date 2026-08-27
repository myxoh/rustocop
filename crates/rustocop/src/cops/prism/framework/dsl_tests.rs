use std::collections::BTreeMap;

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

#[test]
fn cop_entry_macro_matchers_are_unique() {
    let source = include_str!("dsl.rs");
    let body = source
        .split_once("macro_rules! define_cop_entry {")
        .expect("define_cop_entry macro")
        .1
        .split_once("\n}\n\npub(super) use")
        .expect("end of define_cop_entry macro")
        .0;
    let matchers = body.lines().filter_map(|line| {
        line.trim()
            .strip_suffix("=> {")
            .map(str::trim)
            .filter(|matcher| matcher.starts_with('('))
            .map(|matcher| matcher.split_whitespace().collect::<Vec<_>>().join(" "))
    });
    let mut counts = BTreeMap::new();
    for matcher in matchers {
        *counts.entry(matcher).or_insert(0usize) += 1;
    }
    assert!(!counts.is_empty(), "no define_cop_entry matchers found");
    let duplicates = counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .collect::<Vec<_>>();

    assert!(
        duplicates.is_empty(),
        "duplicate define_cop_entry macro matchers: {duplicates:?}"
    );
}

#[test]
fn cop_entry_uses_only_the_canonical_compatibility_callback_vocabulary() {
    let source = include_str!("dsl.rs");
    for obsolete in [
        "rubocop_callbacks(",
        "recovery_rubocop_callbacks(",
        "stateful_rubocop_callbacks(",
    ] {
        assert!(
            !source.contains(obsolete),
            "obsolete define_cop_entry alias remains: {obsolete}"
        );
    }
}
