use super::*;
use crate::config::AutocorrectMode;
use std::collections::HashSet;

pub struct Engine {
    registry: Registry,
    enabled_cops: HashSet<&'static str>,
}

impl Engine {
    pub fn new(enabled: &dyn Fn(&str) -> bool, legacy_cops: &[&'static str]) -> Self {
        let registry = Registry::enabled(enabled);
        let enabled_cops = registry
            .cops
            .iter()
            .map(|cop| cop.name())
            .chain(legacy_cops.iter().copied().filter(|cop| enabled(cop)))
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
        let has_unrecoverable_parse_errors = parsed
            .errors()
            .any(|error| !is_context_only_parse_error(error.message()));
        for error in parsed.errors() {
            for cop in self
                .registry
                .phases
                .parse_errors
                .iter()
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
            .map(|index| &self.registry.cops[*index])
        {
            if has_unrecoverable_parse_errors
                && !matches!(cop.name(), "Lint/Syntax" | "Naming/HeredocDelimiterNaming")
            {
                continue;
            }
            cop.on_source(source, &mut context);
        }
        if has_unrecoverable_parse_errors && self.registry.phases.recovered_nodes.is_empty() {
            return context.finish(source);
        }
        let mut investigation_states: Vec<Box<dyn Any>> = self
            .registry
            .cops
            .iter()
            .map(|cop| {
                let mut state = cop.investigation_state();
                cop.on_new_investigation(state.as_mut());
                state
            })
            .collect();
        let mut runner = Runner {
            registry: &self.registry,
            context: &mut context,
            source,
            ancestors: Vec::new(),
            investigation_states: &mut investigation_states,
            node_cops: if has_unrecoverable_parse_errors {
                &self.registry.phases.recovered_nodes
            } else {
                &self.registry.phases.nodes
            },
        };
        runner.visit(&parsed.node());
        drop(runner);
        if !has_unrecoverable_parse_errors && !self.registry.phases.compatibility_nodes.is_empty() {
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
                    self.registry.cops[*index].on_compatibility_investigation(
                        &processed_source,
                        &mut context,
                        investigation_states[*index].as_mut(),
                    );
                }
                if let Some(root) = processed_source.ast() {
                    for node in root.each_node(&[]) {
                        for index in &self.registry.phases.compatibility_nodes {
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
        context.finish(source)
    }
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
