use super::super::catalog_cop::{replace, report};
use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(DuplicateMethods),
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
