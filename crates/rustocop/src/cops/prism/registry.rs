use super::*;

pub(super) struct Registry {
    pub(super) cops: Vec<Box<dyn Cop>>,
}

pub(super) fn cop_names() -> Vec<&'static str> {
    let mut names = Registry::enabled(&|_| true)
        .cops
        .into_iter()
        .map(|cop| cop.name())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    names
}
