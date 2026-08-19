use super::super::catalog_cop::{custom, replace, report};
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
        replace(
            "Style/RedundantCondition",
            "condition ? true : false",
            "condition",
            "Use the condition directly.",
        ),
        report(
            "Style/IfWithBooleanLiteralBranches",
            "if predicate?\n  true",
            "Use a boolean expression instead of an if with boolean branches.",
        ),
        custom("Style/OneLineConditional", one_line_conditional),
        custom("Lint/UnreachableCode", unreachable_code),
        custom("Lint/LiteralAsCondition", literal_condition),
    ]
}
