use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(DuplicateBranchCop),
        Box::new(UnreachableCode),
    ]
}
