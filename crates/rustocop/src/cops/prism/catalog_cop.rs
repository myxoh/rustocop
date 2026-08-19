use super::*;

pub(super) fn replace(
    name: &'static str,
    old: &'static str,
    new: &'static str,
    message: &'static str,
) -> Box<dyn Cop> {
    Box::new(CatalogCop {
        name,
        rule: Rule::Replace { old, new, message },
    })
}

pub(super) fn report(
    name: &'static str,
    needle: &'static str,
    message: &'static str,
) -> Box<dyn Cop> {
    Box::new(CatalogCop {
        name,
        rule: Rule::Report { needle, message },
    })
}

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
    Replace {
        old: &'static str,
        new: &'static str,
        message: &'static str,
    },
    Report {
        needle: &'static str,
        message: &'static str,
    },
    Custom(fn(&mut CopContext<'_, '_>)),
}

impl Cop for CatalogCop {
    fn name(&self) -> &'static str {
        self.name
    }

    fn on_source(&self, source: &str, context: &mut Context) {
        let mut context = context.cop_context(self.name, source, &[]);
        match self.rule {
            Rule::Replace { old, new, message } => {
                for start in context.source_file().code_offsets(old) {
                    context.replace(
                        message,
                        start..start + old.len(),
                        start..start + old.len(),
                        new,
                    );
                }
            }
            Rule::Report { needle, message } => {
                for start in context.source_file().code_offsets(needle) {
                    context.report(message, start..start + needle.len());
                }
            }
            Rule::Custom(check) => check(&mut context),
        }
    }
}
