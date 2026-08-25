// RuboCop 1.87.0
// Source: lib/rubocop/cop/team.rb
// Source SHA-256: e7d2a5c11c922d13bc693fde2d9ae225d41317ecbb46f4a3f5356c63ef2fc840
// Ported contract: spec/rubocop/cop/team_spec.rb
// Spec SHA-256: 2ff4bf11a7654fa824c3929c35c2a1edef4d83096aeaddf2bd1bb52dde41de09

use std::collections::{BTreeMap, BTreeSet};

use crate::rubocop::ast::processed_source::{sha1_hex, ProcessedSource};
use crate::rubocop::ast::source::SourceBuffer;

use super::corrector::{CorrectionError, Corrector};
use super::framework::{
    AutocorrectMode, CopError, CopRuntime, Finding, Team as RuntimeTeam, TeamInvestigation,
    TeamResult,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TeamOptions {
    pub(crate) autocorrect: Option<bool>,
    pub(crate) debug: Option<bool>,
    pub(crate) stdin: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CopDescriptor {
    pub(crate) name: String,
    pub(crate) joining_forces: Vec<String>,
    pub(crate) target_ruby_supported: bool,
    pub(crate) target_rails_supported: bool,
    pub(crate) config_valid: bool,
}

pub(crate) struct Correction<'corrector, 'buffer, 'source> {
    pub(crate) cop: String,
    pub(crate) corrector: Option<&'corrector Corrector<'buffer, 'source>>,
    pub(crate) incompatible_with: Vec<String>,
}

pub(crate) struct InvestigationResult<'buffer, 'source> {
    report: TeamResult,
    corrector: Option<Corrector<'buffer, 'source>>,
}

#[derive(Clone, Copy)]
pub(crate) struct Fragment<'fragment, 'source> {
    pub(crate) processed_source: &'fragment ProcessedSource<'source>,
    pub(crate) offset: isize,
}

impl<'buffer, 'source> InvestigationResult<'buffer, 'source> {
    pub(crate) fn report(&self) -> &TeamResult {
        &self.report
    }
    pub(crate) fn corrector(&self) -> Option<&Corrector<'buffer, 'source>> {
        self.corrector.as_ref()
    }
}

pub(crate) struct Team {
    runtime: RuntimeTeam,
    descriptors: Vec<CopDescriptor>,
    options: TeamOptions,
    ready: bool,
    errors: Vec<String>,
    warnings: Vec<String>,
    updated_source_file: bool,
}

impl Team {
    pub(crate) fn new(
        cops: Vec<Box<dyn CopRuntime>>,
        descriptors: Vec<CopDescriptor>,
        options: TeamOptions,
    ) -> Result<Self, String> {
        Self::initialize(cops, descriptors, options)
    }

    pub(crate) fn initialize(
        cops: Vec<Box<dyn CopRuntime>>,
        descriptors: Vec<CopDescriptor>,
        options: TeamOptions,
    ) -> Result<Self, String> {
        Self::validate_config(&descriptors)?;
        let mode = if options.autocorrect == Some(true) {
            AutocorrectMode::All
        } else {
            AutocorrectMode::None
        };
        Ok(Self {
            runtime: RuntimeTeam::new(cops, mode),
            descriptors,
            options,
            ready: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            updated_source_file: false,
        })
    }

    pub(crate) fn mobilize(
        cops: Vec<Box<dyn CopRuntime>>,
        descriptors: Vec<CopDescriptor>,
        options: TeamOptions,
    ) -> Result<Self, String> {
        Self::new(cops, Self::mobilize_cops(descriptors), options)
    }

    pub(crate) fn mobilize_cops(cop_classes: Vec<CopDescriptor>) -> Vec<CopDescriptor> {
        cop_classes
    }

    pub(crate) fn forces_for(cops: &[CopDescriptor]) -> BTreeMap<String, Vec<String>> {
        let mut needed = BTreeMap::<String, Vec<String>>::new();
        for cop in cops {
            for force in &cop.joining_forces {
                needed
                    .entry(force.clone())
                    .or_default()
                    .push(cop.name.clone());
            }
        }
        needed
    }

    pub(crate) fn autocorrect(&self) -> Option<bool> {
        self.options.autocorrect
    }

    pub(crate) fn autocorrect_enabled(&self) -> bool {
        self.autocorrect() == Some(true)
    }

    pub(crate) fn debug(&self) -> Option<bool> {
        self.options.debug
    }

    pub(crate) fn errors(&self) -> &[String] {
        &self.errors
    }

    pub(crate) fn warnings(&self) -> &[String] {
        &self.warnings
    }

    pub(crate) fn cops(&self) -> &[CopDescriptor] {
        &self.descriptors
    }

    pub(crate) fn inspect_file(&mut self, processed_source: &ProcessedSource<'_>) -> Vec<Finding> {
        self.investigate(processed_source).findings
    }

    pub(crate) fn investigate(&mut self, processed_source: &ProcessedSource<'_>) -> TeamResult {
        let result = self.investigate_with_corrector(processed_source);
        self.updated_source_file = result.updated_source.is_some();
        result
    }

    pub(crate) fn investigate_fragments(
        &mut self,
        fragments: &[Fragment<'_, '_>],
        original: &ProcessedSource<'_>,
    ) -> TeamResult {
        self.updated_source_file = false;
        let mut findings = Vec::new();
        let mut errors = Vec::new();
        let mut error_messages = Vec::new();
        let mut warning_messages = Vec::new();
        let original_buffer = original.buffer();
        let mut corrector = Corrector::new(&original_buffer);

        for fragment in fragments {
            let investigation = self.investigate_fragment(*fragment);
            findings.extend(
                investigation
                    .result
                    .findings
                    .into_iter()
                    .map(|finding| Self::translate_finding(finding, fragment.offset)),
            );
            errors.extend(investigation.result.errors);
            error_messages.extend(self.errors.iter().cloned());
            warning_messages.extend(self.warnings.iter().cloned());
            if let Some(plan) = investigation.correction {
                let _ = plan.apply_to_with_offset(&mut corrector, fragment.offset);
            }
        }

        self.errors = error_messages;
        self.warnings = warning_messages;
        let updated_source = if self.autocorrect_enabled() && !corrector.is_empty() {
            corrector.rewrite().ok()
        } else {
            None
        };
        self.updated_source_file = updated_source.is_some();
        TeamResult {
            findings,
            errors,
            updated_source,
        }
    }

    pub(crate) fn forces(&self) -> BTreeMap<String, Vec<String>> {
        Self::forces_for(&self.descriptors)
    }

    pub(crate) fn external_dependency_checksum(&self) -> String {
        let joined = self
            .runtime
            .external_dependency_checksum()
            .into_iter()
            .map(|(_, checksum)| checksum)
            .collect::<String>();
        sha1_hex(joined.as_bytes())
    }

    fn be_ready(&mut self) {
        if !self.ready {
            self.reset();
            self.ready = true;
        }
    }

    fn reset(&mut self) {
        self.errors.clear();
        self.warnings.clear();
    }

    fn investigate_partial(&mut self, processed_source: &ProcessedSource<'_>) -> TeamInvestigation {
        let buffer = processed_source.buffer();
        let path = processed_source
            .path()
            .map_or_else(String::new, |path| path.to_string_lossy().into_owned());
        let selected = self
            .roundup_relevant_cops(processed_source)
            .into_iter()
            .map(str::to_owned)
            .collect();
        self.runtime.investigate_with_selected_cops(
            &path,
            &buffer,
            processed_source.ast(),
            &selected,
        )
    }

    fn investigate_with_corrector(&mut self, processed_source: &ProcessedSource<'_>) -> TeamResult {
        self.investigate_with_correction(processed_source).result
    }

    fn investigate_with_correction(
        &mut self,
        processed_source: &ProcessedSource<'_>,
    ) -> TeamInvestigation {
        self.be_ready();
        let result = self.investigate_partial(processed_source);
        let path = processed_source
            .path()
            .map_or_else(String::new, |path| path.to_string_lossy().into_owned());
        self.process_errors(&path, &result.result.errors);
        self.ready = false;
        result
    }

    fn investigate_fragment(&mut self, fragment: Fragment<'_, '_>) -> TeamInvestigation {
        self.investigate_with_correction(fragment.processed_source)
    }

    fn translate_finding(mut finding: Finding, offset: isize) -> Finding {
        finding.location = finding.location.and_then(|range| {
            Some(range.start.checked_add_signed(offset)?..range.end.checked_add_signed(offset)?)
        });
        finding
    }

    fn collated_corrector<'original_buffer, 'original_source>(
        &self,
        corrections: &[Correction<'_, '_, '_>],
        offset: isize,
        original: &'original_buffer SourceBuffer<'original_source>,
    ) -> Option<Corrector<'original_buffer, 'original_source>> {
        if !self.autocorrect_enabled() {
            return None;
        }
        let correction = self.collate_corrections(corrections, offset, original);
        (!correction.is_empty()).then_some(correction)
    }

    fn collate_corrections<'original_buffer, 'original_source>(
        &self,
        report: &[Correction<'_, '_, '_>],
        offset: isize,
        original: &'original_buffer SourceBuffer<'original_source>,
    ) -> Corrector<'original_buffer, 'original_source> {
        let mut result = Corrector::new(original);
        self.each_corrector(report, |correction| {
            let _ = Self::merge_corrector(&mut result, correction, offset);
        });
        result
    }

    fn merge_corrector(
        corrector: &mut Corrector<'_, '_>,
        to_merge: &Corrector<'_, '_>,
        offset: isize,
    ) -> Result<(), CorrectionError> {
        if std::ptr::eq(corrector.source_buffer(), to_merge.source_buffer()) {
            corrector.transaction(|corrector| corrector.merge(to_merge));
            Ok(())
        } else {
            corrector.import(to_merge, offset)
        }
    }

    fn each_corrector<'corrector, 'buffer, 'source>(
        &self,
        report: &[Correction<'corrector, 'buffer, 'source>],
        mut callback: impl FnMut(&'corrector Corrector<'buffer, 'source>),
    ) {
        let mut skips = BTreeSet::new();
        for correction in report {
            let Some(corrector) = correction.corrector else {
                continue;
            };
            if skips.contains(&correction.cop) || corrector.is_empty() {
                continue;
            }
            callback(corrector);
            skips.extend(correction.incompatible_with.iter().cloned());
        }
    }

    fn suppress_clobbering<T>(operation: impl FnOnce() -> Result<T, String>) -> Option<T> {
        operation().ok()
    }

    fn roundup_relevant_cops(&self, _processed_source: &ProcessedSource<'_>) -> Vec<&str> {
        self.descriptors
            .iter()
            .filter(|cop| {
                self.support_target_ruby_version(cop) && self.support_target_rails_version(cop)
            })
            .map(|cop| cop.name.as_str())
            .collect()
    }

    fn support_target_ruby_version(&self, cop: &CopDescriptor) -> bool {
        cop.target_ruby_supported
    }

    fn support_target_rails_version(&self, cop: &CopDescriptor) -> bool {
        cop.target_rails_supported
    }

    fn process_errors(&mut self, file: &str, errors: &[CopError]) {
        for error in errors {
            let location = format!(
                "{file}{}{}",
                error
                    .line
                    .map_or_else(String::new, |line| format!(":{line}")),
                error
                    .column
                    .map_or_else(String::new, |column| format!(":{column}"))
            );
            self.handle_error(&error.message, &location, &error.cop_name);
        }
    }

    fn handle_warning(&mut self, error: &str, location: &str) {
        self.warnings
            .push(format!("{error} (from file: {location})"));
    }

    fn handle_error(&mut self, error: &str, location: &str, cop: &str) {
        self.errors.push(format!(
            "An error occurred while {cop} cop was inspecting {location}: {error}"
        ));
    }

    fn updated_source_file(&self) -> bool {
        self.updated_source_file
    }

    fn validate_config(cops: &[CopDescriptor]) -> Result<(), String> {
        cops.iter()
            .find(|cop| !cop.config_valid)
            .map_or(Ok(()), |cop| {
                Err(format!("invalid configuration for {}", cop.name))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rubocop::ast::processed_source::ParserEngine;
    use crate::rubocop::ast::source::SourceRange;
    use crate::rubocop::cop::framework::CorrectionPlan;
    use crate::rubocop::cop::severity::Severity;

    struct FragmentCorrectingCop {
        width: usize,
        replacement: &'static str,
        findings: Vec<Finding>,
        correction: Option<CorrectionPlan>,
    }

    impl CopRuntime for FragmentCorrectingCop {
        fn name(&self) -> &str {
            "Test/Fragment"
        }

        fn begin_investigation(&mut self, source: &SourceBuffer<'_>) {
            let width = self.width.min(source.len());
            self.findings = vec![Finding::new(
                self.name(),
                0..width,
                "fragment offense",
                Severity::Convention,
                true,
            )];
            let mut correction = CorrectionPlan::new();
            correction.replace(0..width, self.replacement);
            self.correction = Some(correction);
        }

        fn take_findings(&mut self) -> Vec<Finding> {
            std::mem::take(&mut self.findings)
        }

        fn take_correction(&mut self) -> Option<CorrectionPlan> {
            self.correction.take()
        }

        fn supports_autocorrect(&self) -> bool {
            true
        }
    }

    fn fragment_team(autocorrect: bool, width: usize) -> Team {
        Team::new(
            vec![Box::new(FragmentCorrectingCop {
                width,
                replacement: "X",
                findings: Vec::new(),
                correction: None,
            })],
            vec![CopDescriptor {
                name: "Test/Fragment".into(),
                joining_forces: Vec::new(),
                target_ruby_supported: true,
                target_rails_supported: true,
                config_valid: true,
            }],
            TeamOptions {
                autocorrect: Some(autocorrect),
                debug: Some(false),
                stdin: true,
            },
        )
        .unwrap()
    }

    #[test]
    fn ports_force_assembly_correction_skips_and_error_policy() {
        let descriptors = vec![
            CopDescriptor {
                name: "One".into(),
                joining_forces: vec!["VariableForce".into()],
                target_ruby_supported: true,
                target_rails_supported: true,
                config_valid: true,
            },
            CopDescriptor {
                name: "Two".into(),
                joining_forces: vec!["VariableForce".into()],
                target_ruby_supported: true,
                target_rails_supported: true,
                config_valid: true,
            },
        ];
        assert_eq!(
            Team::forces_for(&descriptors)["VariableForce"],
            vec!["One", "Two"]
        );
        let mut team = Team::new(
            Vec::new(),
            descriptors,
            TeamOptions {
                autocorrect: Some(true),
                debug: Some(false),
                stdin: false,
            },
        )
        .unwrap();
        let original = SourceBuffer::new("abcd");
        let mut first = Corrector::new(&original);
        first.replace(SourceRange::new(&original, 0, 1), "A");
        let mut second = Corrector::new(&original);
        second.replace(SourceRange::new(&original, 1, 2), "B");
        let corrections = [
            Correction {
                cop: "One".into(),
                corrector: Some(&first),
                incompatible_with: vec!["Two".into()],
            },
            Correction {
                cop: "Two".into(),
                corrector: Some(&second),
                incompatible_with: vec![],
            },
        ];
        assert_eq!(
            team.collated_corrector(&corrections, 0, &original)
                .unwrap()
                .rewrite()
                .unwrap(),
            "Abcd"
        );

        let fragment = SourceBuffer::new("bc");
        let mut fragment_corrector = Corrector::new(&fragment);
        fragment_corrector.replace(SourceRange::new(&fragment, 0, 2), "BC");
        let imported = [Correction {
            cop: "One".into(),
            corrector: Some(&fragment_corrector),
            incompatible_with: vec![],
        }];
        assert_eq!(
            team.collated_corrector(&imported, 1, &original)
                .unwrap()
                .rewrite()
                .unwrap(),
            "aBCd"
        );

        let mut overlapping = Corrector::new(&original);
        overlapping.replace(SourceRange::new(&original, 0, 2), "left");
        let clobbering = [
            Correction {
                cop: "One".into(),
                corrector: Some(&first),
                incompatible_with: vec![],
            },
            Correction {
                cop: "Three".into(),
                corrector: Some(&overlapping),
                incompatible_with: vec![],
            },
        ];
        assert_eq!(
            team.collated_corrector(&clobbering, 0, &original)
                .unwrap()
                .rewrite()
                .unwrap(),
            "Abcd"
        );
        assert_eq!(team.autocorrect(), Some(true));
        assert_eq!(team.debug(), Some(false));
        assert_eq!(team.cops().len(), 2);
        assert_eq!(
            team.external_dependency_checksum(),
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
        team.handle_warning("warning", "file.rb:1");
        team.handle_error("boom", "file.rb:2", "One");
        assert_eq!(team.warnings.len(), 1);
        assert_eq!(team.errors.len(), 1);
        assert!(Team::suppress_clobbering(|| Err::<(), _>("clobber".into())).is_none());
        assert!(!team.updated_source_file());
    }

    #[test]
    fn nil_options_remain_distinct_from_false_and_disable_correction() {
        let team = Team::new(
            Vec::new(),
            Vec::new(),
            TeamOptions {
                autocorrect: None,
                debug: None,
                stdin: false,
            },
        )
        .unwrap();
        let buffer = SourceBuffer::new("a");
        let mut corrector = Corrector::new(&buffer);
        corrector.replace(SourceRange::new(&buffer, 0, 1), "b");
        let corrections = [Correction {
            cop: "One".into(),
            corrector: Some(&corrector),
            incompatible_with: vec![],
        }];
        assert_eq!(team.autocorrect(), None);
        assert_eq!(team.debug(), None);
        assert!(team.collated_corrector(&corrections, 0, &buffer).is_none());
    }

    #[test]
    fn fragment_investigation_translates_offenses_and_corrections_to_the_original() {
        let original = ProcessedSource::new("a--b", 3.4, None, ParserEngine::Prism).unwrap();
        let left = ProcessedSource::new("a", 3.4, None, ParserEngine::Prism).unwrap();
        let right = ProcessedSource::new("b", 3.4, None, ParserEngine::Prism).unwrap();
        let fragments = [
            Fragment {
                processed_source: &left,
                offset: 0,
            },
            Fragment {
                processed_source: &right,
                offset: 3,
            },
        ];
        let mut team = fragment_team(true, 1);

        let result = team.investigate_fragments(&fragments, &original);

        assert_eq!(result.updated_source.as_deref(), Some("X--X"));
        assert_eq!(
            result
                .findings
                .iter()
                .map(|finding| finding.location.clone().unwrap())
                .collect::<Vec<_>>(),
            vec![0..1, 3..4]
        );
        assert!(team.updated_source_file());
    }

    #[test]
    fn fragment_investigation_suppresses_clobbering_imports_and_honors_autocorrect_mode() {
        let original = ProcessedSource::new("abc", 3.4, None, ParserEngine::Prism).unwrap();
        let left = ProcessedSource::new("ab", 3.4, None, ParserEngine::Prism).unwrap();
        let right = ProcessedSource::new("bc", 3.4, None, ParserEngine::Prism).unwrap();
        let fragments = [
            Fragment {
                processed_source: &left,
                offset: 0,
            },
            Fragment {
                processed_source: &right,
                offset: 1,
            },
        ];

        let corrected = fragment_team(true, 2).investigate_fragments(&fragments, &original);
        assert_eq!(corrected.updated_source.as_deref(), Some("Xc"));

        let uncorrected = fragment_team(false, 2).investigate_fragments(&fragments, &original);
        assert!(uncorrected.updated_source.is_none());
        assert_eq!(uncorrected.findings.len(), 2);
    }

    #[test]
    fn investigation_excludes_cops_outside_the_target_ruby_or_rails_contract() {
        let source = ProcessedSource::new("a", 3.4, None, ParserEngine::Prism).unwrap();
        let mut team = Team::new(
            vec![Box::new(FragmentCorrectingCop {
                width: 1,
                replacement: "X",
                findings: Vec::new(),
                correction: None,
            })],
            vec![CopDescriptor {
                name: "Test/Fragment".into(),
                joining_forces: Vec::new(),
                target_ruby_supported: false,
                target_rails_supported: true,
                config_valid: true,
            }],
            TeamOptions {
                autocorrect: Some(true),
                debug: Some(false),
                stdin: true,
            },
        )
        .unwrap();

        let result = team.investigate(&source);

        assert!(result.findings.is_empty());
        assert!(result.updated_source.is_none());
    }
}
