use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(DuplicateMethods),
        Box::new(AccessModifierDeclarations),
    ]
}
