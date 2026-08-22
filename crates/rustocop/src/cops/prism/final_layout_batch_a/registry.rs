use super::super::catalog_cop::{custom, replace, report};
use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        custom("Layout/MultilineArrayBraceLayout", array_brace_layout),
        Box::new(EmptyLineAfterGuardClause),
        custom(
            "Layout/LineEndStringConcatenationIndentation",
            align_continuation,
        ),
        custom("Layout/MultilineAssignmentLayout", multiline_assignment),
        Box::new(SpaceInsideBlockBraces),
        replace(
            "Layout/SpaceInsideHashLiteralBraces",
            "{  ",
            "{ ",
            "Extra space inside hash braces detected.",
        ),
        custom("Layout/ArgumentAlignment", align_continuation),
        custom("Layout/FirstArrayElementIndentation", align_continuation),
        replace(
            "Layout/LineContinuationLeadingSpace",
            "\n .",
            "\n.",
            "Line continuation should not have leading space.",
        ),
        custom(
            "Layout/MultilineMethodCallBraceLayout",
            method_call_brace_layout,
        ),
        report(
            "Layout/MultilineBlockLayout",
            " { |",
            "Multi-line block argument must be on a separate line.",
        ),
    ]
}
