// Ported from RuboCop 1.87.0:
// spec/rubocop/cop/force_spec.rb
// Spec SHA-256: 86d91716e9bb5f5149e6fed45ce7f86bcb6a6d419a0c8d103eea14fc50fd1d90

use super::force::{force_name, Force, ForceCop, ForceRegistry};

#[derive(Clone, Debug)]
struct Cop {
    name: &'static str,
    calls: Vec<String>,
    responds: bool,
}

impl ForceCop<String> for Cop {
    fn cop_name(&self) -> &str {
        self.name
    }
    fn run_hook(&mut self, method_name: &str, argument: &String) -> Option<Result<(), String>> {
        if !self.responds {
            return None;
        }
        self.calls.push(format!("{method_name}:{argument}"));
        Some(Ok(()))
    }
}

#[test]
fn force_name_returns_the_class_name_without_namespace() {
    assert_eq!(force_name("RuboCop::Cop::VariableForce"), "VariableForce");
}

#[test]
fn run_hook_invokes_every_responding_cop_and_wraps_the_joining_cop() {
    let cops = vec![
        Cop {
            name: "One",
            calls: Vec::new(),
            responds: true,
        },
        Cop {
            name: "Two",
            calls: Vec::new(),
            responds: true,
        },
    ];
    let mut force = Force::new("RuboCop::Cop::Force", cops);
    force.run_hook("message", &"foo".to_owned()).unwrap();
    assert_eq!(force.name(), "Force");
    assert!(force.cops().iter().all(|cop| cop.calls == ["message:foo"]));
}

#[test]
fn initialize_and_inherited_keep_instance_cops_separate_from_subclass_registration() {
    let force = Force::initialize(vec![Cop {
        name: "One",
        calls: Vec::new(),
        responds: false,
    }]);
    assert_eq!(force.cops().len(), 1);
    let mut registry = ForceRegistry::default();
    registry.inherited("FirstForce");
    registry.inherited("SecondForce");
    assert_eq!(registry.all(), ["FirstForce", "SecondForce"]);
}
