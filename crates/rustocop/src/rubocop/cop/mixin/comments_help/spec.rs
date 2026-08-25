use std::collections::BTreeMap;

use super::CommentsHelp;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

#[test]
fn collects_comments_until_the_next_sibling_boundary() {
    let source = "first = 1 # one\n# before second\nsecond = 2\n";
    let parsed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    let root = parsed.ast().unwrap();
    let first = root.child_nodes()[0];
    let help = CommentsHelp::new(&parsed);
    assert!(help.contains_comments(first));
    assert_eq!(help.comments_in_range(first).len(), 2);
    assert_eq!(help.find_end_line(first), 3);
}

#[test]
fn expands_to_associated_leading_comments_and_detects_disable_overlap() {
    let source = "# explanation\nvalue = 1\n";
    let parsed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    let node = parsed.ast().unwrap();
    let help = CommentsHelp::with_disabled_line_ranges(
        &parsed,
        BTreeMap::from([("Lint/Test".into(), std::iter::once(1..3).collect())]),
    );
    let range = help.source_range_with_comment(node).unwrap();
    assert!(range.source().contains("# explanation"));
    assert!(help.comments_contain_disables(node, "Lint/Test"));
    assert!(!help.comments_contain_disables(node, "Lint/Other"));
}
