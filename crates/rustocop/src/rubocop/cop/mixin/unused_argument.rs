// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/unused_argument.rb
// Source SHA-256: 7bf5d074e5e5f80801550f8b77cac79a58a3ff44d7471c149b2262ef89c3a048

use std::ops::Range;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Variable {
    pub(crate) declaration_range: Range<usize>,
    pub(crate) declaration_source: String,
    pub(crate) should_be_unused: bool,
    pub(crate) referenced: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Scope {
    pub(crate) variables: Vec<Variable>,
}

pub(crate) trait UnusedArgument {
    fn message(&self, variable: &Variable) -> String;
    fn add_offense(&mut self, range: Range<usize>, message: String, declaration_source: &str);

    fn after_leaving_scope(&mut self, scope: &Scope) {
        for variable in &scope.variables {
            self.check_argument(variable);
        }
    }

    fn check_argument(&mut self, variable: &Variable) {
        if variable.should_be_unused || variable.referenced {
            return;
        }
        let message = self.message(variable);
        self.add_offense(
            variable.declaration_range.clone(),
            message,
            &variable.declaration_source,
        );
    }
}
