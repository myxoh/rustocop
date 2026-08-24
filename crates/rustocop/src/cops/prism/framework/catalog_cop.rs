use super::*;

pub(super) fn custom(name: &'static str, check: fn(&mut CopContext<'_, '_>)) -> Box<dyn Cop> {
    Box::new(CatalogCop {
        name,
        rule: Rule::Custom(check),
    })
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
