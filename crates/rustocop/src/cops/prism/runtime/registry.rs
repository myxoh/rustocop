use super::*;

pub(super) struct Registry {
    pub(super) cops: Vec<Box<dyn Cop>>,
    pub(super) source_cops: Vec<usize>,
    pub(super) node_cops: Vec<usize>,
    pub(super) parse_error_cops: Vec<usize>,
    pub(super) recovered_node_cops: Vec<usize>,
}

impl Registry {
    pub(super) fn new(cops: Vec<Box<dyn Cop>>) -> Self {
        let source_cops = cop_indices(&cops, |phase| phase.visits_source());
        let node_cops = cop_indices(&cops, |phase| phase.visits_nodes());
        let parse_error_cops = cop_indices(&cops, |phase| phase.visits_parse_errors());
        let recovered_node_cops = cops
            .iter()
            .enumerate()
            .filter_map(|(index, cop)| cop.visits_recovered_nodes().then_some(index))
            .collect();
        Self {
            cops,
            source_cops,
            node_cops,
            parse_error_cops,
            recovered_node_cops,
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
