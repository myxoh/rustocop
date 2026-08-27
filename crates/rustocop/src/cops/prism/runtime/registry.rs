use super::*;
use std::collections::HashMap;

pub(super) struct Registry {
    pub(super) cops: Vec<Box<dyn Cop>>,
    pub(super) phases: PhasePlan,
}

pub(super) struct PhasePlan {
    pub(super) source: Vec<usize>,
    pub(super) nodes: Vec<usize>,
    pub(super) compatibility_nodes: Vec<usize>,
    pub(super) compatibility_nodes_by_kind: HashMap<&'static str, Vec<usize>>,
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
        let mut compatibility_nodes_by_kind = HashMap::<&'static str, Vec<usize>>::new();
        for index in &compatibility_nodes {
            for interest in cops[*index].compatibility_node_interests() {
                match interest {
                    CompatibilityNodeInterest::Callback(callback) => {
                        let kind = callback
                            .strip_prefix("on_")
                            .expect("compatibility callback must start with on_");
                        compatibility_nodes_by_kind
                            .entry(kind)
                            .or_default()
                            .push(*index);
                    }
                    CompatibilityNodeInterest::Kinds(kinds) => {
                        for kind in *kinds {
                            compatibility_nodes_by_kind
                                .entry(kind)
                                .or_default()
                                .push(*index);
                        }
                    }
                }
            }
        }
        for indices in compatibility_nodes_by_kind.values_mut() {
            indices.sort_unstable();
            indices.dedup();
        }
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
            compatibility_nodes_by_kind,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn each_builtin_cop_has_exactly_one_runtime_implementation() {
        let registry = Registry::enabled(&|_| true);
        let mut implementations = BTreeMap::<&str, usize>::new();
        for cop in &registry.cops {
            *implementations.entry(cop.name()).or_default() += 1;
        }
        let duplicates = implementations
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .collect::<Vec<_>>();

        assert!(
            duplicates.is_empty(),
            "built-in cops with multiple runtime implementations: {duplicates:?}"
        );
    }
}
