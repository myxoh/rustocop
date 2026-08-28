use super::*;
use crate::config::AutocorrectMode;
use std::collections::HashSet;

pub struct Engine {
    registry: Registry,
    enabled_cops: HashSet<&'static str>,
}

impl Engine {
    pub fn new(
        enabled: &dyn Fn(&str) -> bool,
        registry_visible: &dyn Fn(&str) -> bool,
        legacy_cops: &[&'static str],
    ) -> Self {
        let registry = Registry::enabled(enabled);
        let enabled_cops = crate::cops::cop_names()
            .into_iter()
            .filter(|cop| registry_visible(cop))
            .chain(
                legacy_cops
                    .iter()
                    .copied()
                    .filter(|cop| registry_visible(cop)),
            )
            .collect();
        Self {
            registry,
            enabled_cops,
        }
    }

    pub(crate) fn implements(&self, cop_name: &str) -> bool {
        self.registry.cops.iter().any(|cop| cop.name() == cop_name)
    }

    pub fn inspect(
        &self,
        path: &str,
        source: &str,
        autocorrect: AutocorrectMode,
        ignore_disable_comments: bool,
        target_ruby_version: RubyVersion,
        source_encoding: SourceEncoding,
        cop_config: Arc<CopConfig>,
    ) -> Inspection {
        let parsed = parse(source.as_bytes());
        let active_cops = self
            .registry
            .cops
            .iter()
            .map(|cop| cop_config.cop_applies_to_path(cop.name(), path))
            .collect::<Vec<_>>();
        let mut context = Context::new(
            autocorrect,
            ignore_disable_comments,
            path,
            target_ruby_version,
            source_encoding,
            cop_config,
        );
        context.set_enabled_cops(self.enabled_cops.iter().copied());
        context.set_parser_warnings(parsed.warnings());
        context.set_parser_comments(parsed.comments());
        let has_unrecoverable_parse_errors = parsed
            .errors()
            .any(|error| !is_context_only_parse_error(error.message()));
        for error in parsed.errors() {
            for cop in self
                .registry
                .phases
                .parse_errors
                .iter()
                .filter(|index| active_cops[**index])
                .map(|index| &self.registry.cops[*index])
            {
                cop.on_parse_error(&error, source, &mut context);
            }
        }
        for cop in self
            .registry
            .phases
            .source
            .iter()
            .filter(|index| active_cops[**index])
            .map(|index| &self.registry.cops[*index])
        {
            if has_unrecoverable_parse_errors
                && !matches!(cop.name(), "Lint/Syntax" | "Naming/HeredocDelimiterNaming")
            {
                continue;
            }
            cop.on_source(source, &mut context);
        }
        let recovered_compatibility_source = has_unrecoverable_parse_errors
            && self
                .registry
                .phases
                .compatibility_nodes
                .iter()
                .any(|index| {
                    active_cops[*index]
                        && self.registry.cops[*index].name() == "Naming/HeredocDelimiterNaming"
                });
        if has_unrecoverable_parse_errors
            && self.registry.phases.recovered_nodes.is_empty()
            && !recovered_compatibility_source
        {
            return context.finish(source);
        }
        let mut investigation_states: Vec<Box<dyn Any>> = self
            .registry
            .cops
            .iter()
            .zip(&active_cops)
            .map(|(cop, active)| begin_investigation(cop.as_ref(), *active))
            .collect();
        let selected_node_cops = if has_unrecoverable_parse_errors {
            &self.registry.phases.recovered_nodes
        } else {
            &self.registry.phases.nodes
        };
        let active_node_cops = selected_node_cops
            .iter()
            .copied()
            .filter(|index| active_cops[*index])
            .collect::<Vec<_>>();
        let mut runner = Runner {
            registry: &self.registry,
            context: &mut context,
            source,
            ancestors: Vec::new(),
            investigation_states: &mut investigation_states,
            node_cops: &active_node_cops,
        };
        runner.visit(&parsed.node());
        drop(runner);
        if (!has_unrecoverable_parse_errors || recovered_compatibility_source)
            && self
                .registry
                .phases
                .compatibility_nodes
                .iter()
                .any(|index| active_cops[*index])
        {
            let processed_source =
                crate::rubocop::ast::processed_source::ProcessedSource::from_prism_result(
                    source,
                    target_ruby_version.as_f64(),
                    Some(path.into()),
                    crate::rubocop::ast::processed_source::ParserEngine::Default,
                    &parsed,
                );
            if let Ok(processed_source) = processed_source {
                for index in &self.registry.phases.compatibility_nodes {
                    if !active_cops[*index] {
                        continue;
                    }
                    if has_unrecoverable_parse_errors
                        && self.registry.cops[*index].name() != "Naming/HeredocDelimiterNaming"
                    {
                        continue;
                    }
                    self.registry.cops[*index].on_compatibility_investigation_with_prism(
                        &processed_source,
                        &parsed,
                        &mut context,
                        investigation_states[*index].as_mut(),
                    );
                }
                if !has_unrecoverable_parse_errors {
                    if let Some(root) = processed_source.ast() {
                        for node in root.each_node(&[]) {
                            let Some(indices) = self
                                .registry
                                .phases
                                .compatibility_nodes_by_kind
                                .get(node.kind())
                            else {
                                continue;
                            };
                            for index in indices {
                                if !active_cops[*index] {
                                    continue;
                                }
                                self.registry.cops[*index].on_compatibility_node_with_state(
                                    node,
                                    &processed_source,
                                    &mut context,
                                    investigation_states[*index].as_mut(),
                                );
                            }
                        }
                    }
                }
            }
        }
        context.finish(source)
    }
}

fn begin_investigation(cop: &dyn Cop, active: bool) -> Box<dyn Any> {
    let mut state = cop.investigation_state();
    if active {
        cop.on_new_investigation(state.as_mut());
    }
    state
}

fn is_context_only_parse_error(message: &str) -> bool {
    matches!(
        message,
        "Invalid break"
            | "Invalid next"
            | "Invalid redo"
            | "Invalid retry without rescue"
            | "Invalid yield"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingCop<'count>(&'count AtomicUsize);

    impl Cop for CountingCop<'_> {
        fn name(&self) -> &'static str {
            "Test/CountingCop"
        }

        fn on_new_investigation(&self, _state: &mut dyn Any) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn excluded_cops_do_not_receive_investigation_callbacks() {
        let calls = AtomicUsize::new(0);
        let cop = CountingCop(&calls);

        begin_investigation(&cop, false);
        assert_eq!(calls.load(Ordering::Relaxed), 0);

        begin_investigation(&cop, true);
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }
}
