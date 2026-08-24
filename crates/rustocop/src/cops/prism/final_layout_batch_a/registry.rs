use super::super::catalog_cop::{custom, replace};
use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(EmptyLineAfterGuardClause),
        custom(
            "Layout/LineEndStringConcatenationIndentation",
            align_continuation,
        ),
        Box::new(SpaceInsideBlockBraces),
        replace(
            "Layout/SpaceInsideHashLiteralBraces",
            "{  ",
            "{ ",
            "Extra space inside hash braces detected.",
        ),
        replace(
            "Layout/LineContinuationLeadingSpace",
            "\n .",
            "\n.",
            "Line continuation should not have leading space.",
        ),
    ]
}
