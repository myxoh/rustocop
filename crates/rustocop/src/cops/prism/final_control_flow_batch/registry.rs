use super::super::catalog_cop::{custom, report};
use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        report(
            "Lint/NoReturnInBeginEndBlocks",
            "begin\n  return",
            "Do not return from an explicit `begin` block.",
        ),
        report(
            "Lint/RescueType",
            "rescue '",
            "Rescue an exception class rather than a string literal.",
        ),
        custom("Lint/DuplicateBranch", identical_branches),
        Box::new(UnreachableCode),
        custom("Lint/LiteralAsCondition", literal_condition),
    ]
}
