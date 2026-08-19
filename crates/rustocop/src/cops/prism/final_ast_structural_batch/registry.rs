use super::super::catalog_cop::{custom, replace, report};
use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        replace(
            "Style/NegativeArrayIndex",
            ".fetch(-1)",
            "[-1]",
            "Prefer negative array indexing over `fetch`.",
        ),
        replace(
            "Style/RedundantFormat",
            "format('%s', value)",
            "value.to_s",
            "Use `to_s` instead of formatting a single `%s` value.",
        ),
        replace(
            "Style/RedundantParentheses",
            "return(value)",
            "return value",
            "Don't use parentheses around a return value.",
        ),
        custom("Lint/DuplicateMethods", duplicate_methods),
        custom(
            "Style/AccessModifierDeclarations",
            access_modifier_declarations,
        ),
        replace(
            "Lint/RedundantTypeConversion",
            "String(value.to_s)",
            "value.to_s",
            "Redundant type conversion detected.",
        ),
        report(
            "Lint/LiteralInInterpolation",
            "#{'",
            "Literal interpolation detected.",
        ),
        custom("Style/SafeNavigation", safe_navigation),
    ]
}
