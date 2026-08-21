use super::super::catalog_cop::{custom, replace, report};
use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        custom("Lint/DuplicateMethods", duplicate_methods),
        Box::new(AccessModifierDeclarations),
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
    ]
}
