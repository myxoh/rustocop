use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use super::exclude_limit::ExcludeLimit;

#[test]
fn transforms_parameter_names_and_aggregates_maxima_per_cop() {
    assert_eq!(ExcludeLimit::transform("Max"), "max");
    assert_eq!(ExcludeLimit::transform("MinDigits"), "min_digits");

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "rustocop-exclude-limit-{}-{unique}",
        std::process::id()
    ));
    let tracker = ExcludeLimit::new(Some(root.clone()));
    assert_eq!(tracker.tmp_dir(), Some(root.as_path()));
    tracker.record("Metrics/MethodLength", "Max", 10).unwrap();
    tracker.record("Metrics/MethodLength", "Max", 14).unwrap();
    tracker
        .record("Metrics/MethodLength", "CountComments", 1)
        .unwrap();
    assert_eq!(
        tracker.read_limits("Metrics/MethodLength").unwrap(),
        [("CountComments".to_owned(), 1), ("Max".to_owned(), 14)].into()
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_tmp_directory_or_cop_directory_is_a_noop() {
    let tracker = ExcludeLimit::default();
    tracker.record("Layout/LineLength", "Max", 80).unwrap();
    assert!(tracker.read_limits("Layout/LineLength").unwrap().is_empty());
    assert_eq!(tracker.cop_dir_for("Layout/LineLength"), None);
}
