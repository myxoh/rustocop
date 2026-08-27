use super::*;

pub(super) fn custom(name: &'static str, check: fn(&mut CopContext<'_, '_>)) -> Box<dyn Cop> {
    Box::new(CatalogCop {
        name,
        rule: Rule::Custom(check),
    })
}

pub(super) fn compatibility_custom(
    name: &'static str,
    check: fn(&mut CompatibilityCopContext<'_, '_, '_>),
) -> Box<dyn Cop> {
    Box::new(CompatibilityCatalogCop { name, check })
}

struct CompatibilityCatalogCop {
    name: &'static str,
    check: fn(&mut CompatibilityCopContext<'_, '_, '_>),
}

impl Cop for CompatibilityCatalogCop {
    fn name(&self) -> &'static str {
        self.name
    }

    fn phase(&self) -> CopPhase {
        CopPhase::CompatibilityNode
    }

    fn on_compatibility_investigation_with_prism<'processed, 'source>(
        &self,
        processed_source: &'processed crate::rubocop::ast::processed_source::ProcessedSource<
            'source,
        >,
        prism_result: &'processed ruby_prism::ParseResult<'source>,
        context: &mut Context,
        _state: &mut dyn Any,
    ) {
        let mut context = CompatibilityCopContext::new_with_prism(
            context,
            self.name(),
            processed_source,
            prism_result,
        );
        (self.check)(&mut context);
    }
}

struct CatalogCop {
    name: &'static str,
    rule: Rule,
}

enum Rule {
    Custom(fn(&mut CopContext<'_, '_>)),
}

impl Cop for CatalogCop {
    fn name(&self) -> &'static str {
        self.name
    }

    fn phase(&self) -> CopPhase {
        CopPhase::Source
    }

    fn on_source(&self, source: &str, context: &mut Context) {
        let mut context = context.cop_context(self.name, source, &[]);
        match self.rule {
            Rule::Custom(check) => check(&mut context),
        }
    }
}
