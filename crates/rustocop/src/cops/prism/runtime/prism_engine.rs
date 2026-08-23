use super::*;

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
        target_ruby_version: RubyVersion,
        cop_config: Arc<CopConfig>,
    ) -> Inspection {
        let parsed = parse(source.as_bytes());
        let mut context = Context::new(autocorrect, path, target_ruby_version, cop_config);
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
