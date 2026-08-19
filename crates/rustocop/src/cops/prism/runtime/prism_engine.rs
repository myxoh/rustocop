use super::*;
use std::cmp::Reverse;
use std::thread;

pub struct Engine {
    registry: Registry,
}

impl Engine {
    pub fn new(enabled: &dyn Fn(&str) -> bool) -> Self {
        Self {
            registry: Registry::enabled(enabled),
        }
    }

    pub(crate) fn implements(&self, cop_name: &str) -> bool {
        self.registry.cops.iter().any(|cop| cop.name() == cop_name)
    }

    pub fn inspect(
        &self,
        path: &str,
        source: &str,
        autocorrect: bool,
        cop_parallelism: Parallelism,
        target_ruby_version: RubyVersion,
        cop_config: Arc<CopConfig>,
    ) -> Inspection {
        let source_workers = source_worker_count(cop_parallelism, self.registry.source_cops.len());
        if source_workers == 0 {
            return self.inspect_sequential(
                path,
                source,
                autocorrect,
                target_ruby_version,
                cop_config,
            );
        }

        debug_assert!(!autocorrect, "cop-parallel inspection is detection-only");
        let (tree_inspection, source_inspections) = thread::scope(|scope| {
            let handles = (0..source_workers)
                .map(|worker| {
                    let cop_config = cop_config.clone();
                    scope.spawn(move || {
                        let mut context =
                            Context::new(false, path, target_ruby_version, cop_config);
                        for index in self
                            .registry
                            .source_cops
                            .iter()
                            .copied()
                            .skip(worker)
                            .step_by(source_workers)
                        {
                            self.registry.cops[index].on_source(source, &mut context);
                        }
                        context.finish(source)
                    })
                })
                .collect::<Vec<_>>();
            let tree_inspection =
                self.inspect_tree_phases(path, source, target_ruby_version, cop_config);
            let source_inspections: Vec<_> = handles
                .into_iter()
                .map(|handle| handle.join().expect("cop worker panicked"))
                .collect();
            (tree_inspection, source_inspections)
        });
        merge_inspections(
            source,
            std::iter::once(tree_inspection).chain(source_inspections),
        )
    }

    fn inspect_sequential(
        &self,
        path: &str,
        source: &str,
        autocorrect: bool,
        target_ruby_version: RubyVersion,
        cop_config: Arc<CopConfig>,
    ) -> Inspection {
        let parsed = parse(source.as_bytes());
        let mut context = Context::new(autocorrect, path, target_ruby_version, cop_config);
        for index in &self.registry.source_cops {
            self.registry.cops[*index].on_source(source, &mut context);
        }
        inspect_parse_errors(&self.registry, &parsed, source, &mut context);
        inspect_nodes(&self.registry, &parsed, source, &mut context);
        context.finish(source)
    }

    fn inspect_tree_phases(
        &self,
        path: &str,
        source: &str,
        target_ruby_version: RubyVersion,
        cop_config: Arc<CopConfig>,
    ) -> Inspection {
        let parsed = parse(source.as_bytes());
        let mut context = Context::new(false, path, target_ruby_version, cop_config);
        inspect_parse_errors(&self.registry, &parsed, source, &mut context);
        inspect_nodes(&self.registry, &parsed, source, &mut context);
        context.finish(source)
    }
}

fn inspect_parse_errors(
    registry: &Registry,
    parsed: &ruby_prism::ParseResult<'_>,
    source: &str,
    context: &mut Context,
) {
    for error in parsed.errors() {
        for index in &registry.parse_error_cops {
            registry.cops[*index].on_parse_error(&error, source, context);
        }
    }
}

fn inspect_nodes(
    registry: &Registry,
    parsed: &ruby_prism::ParseResult<'_>,
    source: &str,
    context: &mut Context,
) {
    let mut runner = Runner {
        registry,
        node_cops: &registry.node_cops,
        context,
        source,
        ancestors: Vec::new(),
    };
    runner.visit(&parsed.node());
}

fn merge_inspections(
    source: &str,
    inspections: impl IntoIterator<Item = Inspection>,
) -> Inspection {
    let mut findings = inspections
        .into_iter()
        .flat_map(|inspection| inspection.findings)
        .collect::<Vec<_>>();
    findings.sort_by_key(|finding| {
        (
            finding.start_offset,
            Reverse(finding.end_offset),
            finding.cop_name,
        )
    });
    Inspection {
        findings,
        corrected_source: source.to_string(),
    }
}

fn source_worker_count(parallelism: Parallelism, source_cop_count: usize) -> usize {
    let requested = match parallelism {
        Parallelism::Sequential => 0,
        Parallelism::Automatic => thread::available_parallelism().map_or(1, usize::from),
        Parallelism::Fixed(jobs) => jobs,
    };
    requested.saturating_sub(1).min(source_cop_count)
}
