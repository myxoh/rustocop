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
        for cop in self
            .registry
            .source_cops
            .iter()
            .map(|index| &self.registry.cops[*index])
        {
            cop.on_source(source, &mut context);
        }
        for error in parsed.errors() {
            for cop in self
                .registry
                .parse_error_cops
                .iter()
                .map(|index| &self.registry.cops[*index])
            {
                cop.on_parse_error(&error, source, &mut context);
            }
        }
        let mut runner = Runner {
            registry: &self.registry,
            context: &mut context,
            source,
            ancestors: Vec::new(),
        };
        runner.visit(&parsed.node());
        context.finish(source)
    }
}
