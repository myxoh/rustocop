// RuboCop 1.87.0
// Source: lib/rubocop/cop/force.rb
// Source SHA-256: 4a40d2b70c8f91d998aa99054e18be0033f313dc79115b0cb6dec617f781da1c

use std::fmt;

pub(crate) trait ForceCop<Arguments> {
    fn cop_name(&self) -> &str;
    fn run_hook(&mut self, method_name: &str, arguments: &Arguments) -> Option<Result<(), String>>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HookError {
    pub(crate) joining_cop: String,
    pub(crate) source: String,
}

impl HookError {
    pub(crate) fn joining_cop(&self) -> &str {
        &self.joining_cop
    }
}

impl fmt::Display for HookError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.source)
    }
}

pub(crate) struct Force<Cop> {
    name: String,
    cops: Vec<Cop>,
}

impl<Cop> Force<Cop> {
    pub(crate) fn initialize(cops: Vec<Cop>) -> Self {
        Self {
            name: String::new(),
            cops,
        }
    }

    pub(crate) fn new(class_name: &str, cops: Vec<Cop>) -> Self {
        Self {
            name: force_name(class_name).to_owned(),
            cops,
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }
    pub(crate) fn cops(&self) -> &[Cop] {
        &self.cops
    }

    pub(crate) fn run_hook<Arguments>(
        &mut self,
        method_name: &str,
        arguments: &Arguments,
    ) -> Result<(), HookError>
    where
        Cop: ForceCop<Arguments>,
    {
        for cop in &mut self.cops {
            let name = cop.cop_name().to_owned();
            if let Some(result) = cop.run_hook(method_name, arguments) {
                result.map_err(|source| HookError {
                    joining_cop: name,
                    source,
                })?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ForceRegistry {
    subclasses: Vec<String>,
}

impl ForceRegistry {
    pub(crate) fn inherited(&mut self, subclass: impl Into<String>) {
        self.subclasses.push(subclass.into());
    }

    pub(crate) fn all(&self) -> &[String] {
        &self.subclasses
    }
}

pub(crate) fn force_name(class_name: &str) -> &str {
    class_name.rsplit("::").next().unwrap_or(class_name)
}
