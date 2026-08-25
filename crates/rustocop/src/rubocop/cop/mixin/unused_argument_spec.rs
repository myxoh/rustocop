use std::ops::Range;

use super::unused_argument::{Scope, UnusedArgument, Variable};

#[derive(Default)]
struct Cop(Vec<(Range<usize>, String, String)>);

impl UnusedArgument for Cop {
    fn message(&self, variable: &Variable) -> String {
        format!("unused {}", variable.declaration_source)
    }
    fn add_offense(&mut self, range: Range<usize>, message: String, source: &str) {
        self.0.push((range, message, source.into()));
    }
}

#[test]
fn leaving_scope_checks_every_variable_but_reports_only_unreferenced_expected_arguments() {
    let variable = |name: &str, unused, referenced| Variable {
        declaration_range: 2..2 + name.len(),
        declaration_source: name.into(),
        should_be_unused: unused,
        referenced,
    };
    let scope = Scope {
        variables: vec![
            variable("used", false, true),
            variable("_expected", true, false),
            variable("missing", false, false),
        ],
    };
    let mut cop = Cop::default();
    cop.after_leaving_scope(&scope);
    assert_eq!(cop.0, [(2..9, "unused missing".into(), "missing".into())]);
}
