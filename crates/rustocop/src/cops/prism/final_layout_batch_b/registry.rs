use super::super::catalog_cop::{custom, replace};
use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        custom(
            "Layout/HeredocArgumentClosingParenthesis",
            heredoc_parenthesis,
        ),
        replace(
            "Layout/SpaceInsideArrayPercentLiteral",
            "%w[ ",
            "%w[",
            "Space inside percent array detected.",
        ),
        custom("Layout/RescueEnsureAlignment", end_alignment),
        custom("Layout/HashAlignment", hash_alignment),
        custom("Layout/SpaceAroundOperators", operator_spacing),
        custom("Layout/HeredocIndentation", heredoc_indentation),
        replace(
            "Layout/SpaceAroundKeyword",
            "! defined?",
            "!defined?",
            "Space around keyword detected.",
        ),
        custom(
            "Layout/MultilineMethodCallIndentation",
            continuation_indentation,
        ),
        replace(
            "Layout/SpaceInsidePercentLiteralDelimiters",
            "%w( ",
            "%w(",
            "Space inside percent literal delimiters detected.",
        ),
    ]
}
