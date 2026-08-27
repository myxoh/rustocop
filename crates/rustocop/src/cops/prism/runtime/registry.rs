use super::*;

pub(super) struct Registry {
    pub(super) cops: Vec<Box<dyn Cop>>,
    pub(super) phases: PhasePlan,
}

pub(super) struct PhasePlan {
    pub(super) source: Vec<usize>,
    pub(super) nodes: Vec<usize>,
    pub(super) compatibility_nodes: Vec<usize>,
    pub(super) parse_errors: Vec<usize>,
    pub(super) recovered_nodes: Vec<usize>,
}

impl Registry {
    pub(super) fn new(cops: Vec<Box<dyn Cop>>) -> Self {
        let phases = PhasePlan::new(&cops);
        Self { cops, phases }
    }
}

impl PhasePlan {
    fn new(cops: &[Box<dyn Cop>]) -> Self {
        let source = cop_indices(cops, |phase| phase.visits_source());
        let nodes = cop_indices(cops, |phase| phase.visits_nodes());
        let compatibility_nodes = cop_indices(cops, |phase| phase.visits_compatibility_nodes());
        let parse_errors = cop_indices(cops, |phase| phase.visits_parse_errors());
        let recovered_nodes = cops
            .iter()
            .enumerate()
            .filter_map(|(index, cop)| cop.visits_recovered_nodes().then_some(index))
            .collect();
        Self {
            source,
            nodes,
            compatibility_nodes,
            parse_errors,
            recovered_nodes,
        }
    }
}

fn cop_indices(cops: &[Box<dyn Cop>], accepts: impl Fn(CopPhase) -> bool) -> Vec<usize> {
    cops.iter()
        .enumerate()
        .filter_map(|(index, cop)| accepts(cop.phase()).then_some(index))
        .collect()
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
