// RuboCop 1.87.0
// Source: lib/rubocop/cop/commissioner.rb
// Source SHA-256: 152961ea12a39721a0ea78297f7bd6507111d3a02a3718dbcbcfe2afbd0b31dd

use std::collections::BTreeMap;

use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::processed_source::ProcessedSource;

use super::framework::{
    Commissioner as RuntimeCommissioner, CopError, CorrectionPlan, InvestigationReport,
};

const RESTRICTED_CALLBACKS: [&str; 4] = ["on_send", "on_csend", "after_send", "after_csend"];
type CallbackMap = BTreeMap<String, Vec<String>>;
type RestrictedCallbackMap = BTreeMap<String, CallbackMap>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CallbackDescriptor {
    pub(crate) cop: String,
    pub(crate) callbacks_needed: Vec<String>,
    pub(crate) restrict_on_send: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SourceInvestigationReport {
    report: InvestigationReport,
    processed_source: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CopReportView<'report> {
    pub(crate) cop: &'report str,
    pub(crate) offenses: &'report [super::framework::Finding],
    pub(crate) corrector: Option<&'report CorrectionPlan>,
}

impl SourceInvestigationReport {
    pub(crate) fn processed_source(&self) -> &str {
        &self.processed_source
    }
    pub(crate) fn cop_reports(&self) -> Vec<CopReportView<'_>> {
        self.report
            .cops
            .iter()
            .zip(&self.report.offenses_per_cop)
            .zip(&self.report.correctors)
            .map(|((cop, offenses), corrector)| CopReportView {
                cop,
                offenses,
                corrector: corrector.as_ref(),
            })
            .collect()
    }
    pub(crate) fn cops(&self) -> &[String] {
        &self.report.cops
    }
    pub(crate) fn offenses_per_cop(&self) -> &[Vec<super::framework::Finding>] {
        &self.report.offenses_per_cop
    }
    pub(crate) fn correctors(&self) -> &[Option<CorrectionPlan>] {
        &self.report.correctors
    }
    pub(crate) fn offenses(&self) -> Vec<super::framework::Finding> {
        self.report.offenses()
    }
    pub(crate) fn merge(self, investigation: Self) -> Self {
        Self {
            report: self.report.merge(investigation.report),
            processed_source: self.processed_source,
        }
    }
}

pub(crate) struct Commissioner {
    runtime: RuntimeCommissioner,
    descriptors: Vec<CallbackDescriptor>,
    callbacks: BTreeMap<String, Vec<String>>,
    restricted: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    errors: Vec<CopError>,
    raise_error: bool,
    raise_cop_error: bool,
}

impl Commissioner {
    pub(crate) fn errors(&self) -> &[CopError] {
        &self.errors
    }

    pub(crate) fn initialize(
        runtime: RuntimeCommissioner,
        descriptors: Vec<CallbackDescriptor>,
        raise_error: bool,
        raise_cop_error: bool,
    ) -> Self {
        let mut commissioner = Self {
            runtime,
            descriptors,
            callbacks: BTreeMap::new(),
            restricted: BTreeMap::new(),
            errors: Vec::new(),
            raise_error,
            raise_cop_error,
        };
        commissioner.initialize_callbacks();
        commissioner.reset();
        commissioner
    }

    pub(crate) fn investigate(
        &mut self,
        processed_source: &ProcessedSource<'_>,
    ) -> SourceInvestigationReport {
        self.reset();
        self.begin_investigation(processed_source, 0, processed_source);
        let report = self.runtime.investigate_report(processed_source);
        self.errors = report.errors.clone();
        SourceInvestigationReport {
            report,
            processed_source: processed_source.raw_source().to_owned(),
        }
    }

    pub(crate) fn begin_investigation(
        &mut self,
        _processed_source: &ProcessedSource<'_>,
        _offset: usize,
        _original: &ProcessedSource<'_>,
    ) {
        // RuntimeCommissioner performs the actual per-cop begin callbacks.
    }

    pub(crate) fn trigger_responding_cops(
        &self,
        callback: &str,
        _node: NodeRef<'_>,
    ) -> Vec<String> {
        self.callbacks.get(callback).cloned().unwrap_or_default()
    }

    pub(crate) fn on_callback(&self, callback: &str, node: NodeRef<'_>) -> Vec<String> {
        let mut cops = self.trigger_responding_cops(callback, node);
        cops.extend(self.trigger_restricted_cops(callback, node));
        cops
    }

    pub(crate) fn reset(&mut self) {
        self.errors.clear();
    }

    pub(crate) fn initialize_callbacks(&mut self) {
        self.callbacks = self.build_callbacks(&self.descriptors);
        let (callbacks, restricted) = self.restrict_callbacks(self.callbacks.clone());
        self.callbacks = callbacks;
        self.restricted = restricted;
    }

    pub(crate) fn build_callbacks(
        &self,
        cops: &[CallbackDescriptor],
    ) -> BTreeMap<String, Vec<String>> {
        let mut callbacks = BTreeMap::<String, Vec<String>>::new();
        for cop in cops {
            for callback in &cop.callbacks_needed {
                callbacks
                    .entry(callback.clone())
                    .or_default()
                    .push(cop.cop.clone());
            }
        }
        callbacks
    }

    pub(crate) fn restrict_callbacks(
        &self,
        mut callbacks: BTreeMap<String, Vec<String>>,
    ) -> (CallbackMap, RestrictedCallbackMap) {
        let mut restricted = BTreeMap::new();
        for callback in RESTRICTED_CALLBACKS {
            let cops = callbacks.get(callback).cloned().unwrap_or_default();
            let (unrestricted, map) = self.restricted_map(&cops);
            callbacks.insert(callback.into(), unrestricted);
            restricted.insert(callback.into(), map);
        }
        (callbacks, restricted)
    }

    pub(crate) fn trigger_restricted_cops(&self, event: &str, node: NodeRef<'_>) -> Vec<String> {
        let Some(name) = node.method_name() else {
            return Vec::new();
        };
        self.restricted
            .get(event)
            .and_then(|map| map.get(name))
            .cloned()
            .unwrap_or_default()
    }

    pub(crate) fn restricted_map(
        &self,
        cops: &[String],
    ) -> (Vec<String>, BTreeMap<String, Vec<String>>) {
        let mut unrestricted = Vec::new();
        let mut map = BTreeMap::<String, Vec<String>>::new();
        for cop_name in cops {
            let restrictions = self
                .descriptors
                .iter()
                .find(|cop| &cop.cop == cop_name)
                .map(|cop| cop.restrict_on_send.as_slice())
                .unwrap_or_default();
            if restrictions.is_empty() {
                unrestricted.push(cop_name.clone());
            } else {
                for name in restrictions {
                    map.entry(name.clone()).or_default().push(cop_name.clone());
                }
            }
        }
        (unrestricted, map)
    }

    pub(crate) fn invoke(&self, callback: &str, cops: &[String]) -> Vec<(String, String)> {
        cops.iter()
            .cloned()
            .map(|cop| (cop, callback.to_owned()))
            .collect()
    }

    pub(crate) fn invoke_with_argument<T>(
        &self,
        callback: &str,
        cops: &[String],
        _argument: &T,
    ) -> Vec<(String, String)> {
        self.invoke(callback, cops)
    }

    pub(crate) fn with_cop_error_handling<T>(
        &mut self,
        cop_name: &str,
        node: Option<NodeRef<'_>>,
        result: Result<T, String>,
    ) -> Result<Option<T>, CopError> {
        match result {
            Ok(value) => Ok(Some(value)),
            Err(message) => {
                if self.raise_error {
                    panic!("{message}");
                }
                let error = CopError {
                    cop_name: cop_name.into(),
                    message,
                    line: node.map(NodeRef::first_line),
                    column: node.map(NodeRef::column),
                };
                if self.raise_cop_error {
                    return Err(error);
                }
                self.errors.push(error);
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod spec;
