use super::Branch;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

#[test]
fn sibling_if_bodies_are_exclusive_but_the_condition_always_runs() {
    let parsed = ProcessedSource::new(
        "if ready\n  yes\nelse\n  no\nend",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let conditional = parsed.ast().unwrap();
    let yes = conditional.if_branch().unwrap();
    let no = conditional.else_branch().unwrap();
    let yes_branch = Branch::of(yes, None).unwrap();
    let no_branch = Branch::of(no, None).unwrap();
    assert!(yes_branch.exclusive_with(Some(no_branch)));
    assert!(!yes_branch.always_run());
    assert_eq!(Branch::branch_type("CaseMatch"), "case_match");
}

#[test]
fn equality_hash_and_ancestor_contracts_are_stable() {
    let parsed =
        ProcessedSource::new("if ready\n  work\nend", 3.4, None, ParserEngine::Prism).unwrap();
    let work = parsed.ast().unwrap().if_branch().unwrap();
    let first = Branch::of(work, None).unwrap();
    let second = Branch::of(work, None).unwrap();
    assert!(first.equivalent(Some(second)));
    assert_eq!(first.hash(), second.hash());
    assert_eq!(first.each_ancestor(true).len(), 1);
}
