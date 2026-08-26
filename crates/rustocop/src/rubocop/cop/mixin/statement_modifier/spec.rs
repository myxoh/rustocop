use super::StatementModifier;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

#[test]
fn converts_eligible_three_line_conditionals_to_modifier_source() {
    let parsed =
        ProcessedSource::new("if ready\n  work\nend", 3.4, None, ParserEngine::Prism).unwrap();
    let check = StatementModifier::new(&parsed, Some(80), "Style/IfUnlessModifier");
    let node = parsed.ast().unwrap();
    assert!(check.single_line_as_modifier(node));
    assert_eq!(check.to_modifier_form(node), "work if ready");
    assert_eq!(check.length_in_modifier_form(node), 13);
}

#[test]
fn assignments_and_disabling_comments_keep_their_safety_gates() {
    let parsed = ProcessedSource::new(
        "if value = next_value\n  work\nend",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let check = StatementModifier::new(&parsed, None, "Style/IfUnlessModifier");
    assert!(!check.single_line_as_modifier(parsed.ast().unwrap()));
    assert!(check.comment_disables_cop("# rubocop:disable Style/IfUnlessModifier"));
    assert!(!check.comment_disables_cop("# rubocop:disable Lint/Other"));
}
