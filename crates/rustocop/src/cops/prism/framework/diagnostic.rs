use ruby_prism::{Location, Node};
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::sync::Arc;

use super::correction_engine::{accepted_corrections, apply_edits, Correction, Edit};
use super::{CopContext, CopPolicy};
use crate::config::{AutocorrectMode, CopConfig, RubyVersion, SourceEncoding};

#[path = "diagnostic/reporter.rs"]
mod reporter;
pub(super) use reporter::Reporter;

#[derive(Debug)]
pub struct Finding {
    pub cop_name: &'static str,
    pub message: String,
    pub correctable: bool,
    pub corrected: bool,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug)]
pub struct Inspection {
    pub findings: Vec<Finding>,
    pub corrected_source: String,
}

pub(super) trait ByteRange {
    fn offsets(self) -> Range<usize>;
}

impl ByteRange for Range<usize> {
    fn offsets(self) -> Range<usize> {
        self
    }
}

impl ByteRange for (usize, usize) {
    fn offsets(self) -> Range<usize> {
        self.0..self.1
    }
}

impl ByteRange for Location<'_> {
    fn offsets(self) -> Range<usize> {
        self.start_offset()..self.end_offset()
    }
}

impl ByteRange for &Location<'_> {
    fn offsets(self) -> Range<usize> {
        self.start_offset()..self.end_offset()
    }
}

pub(crate) struct Context {
    autocorrect: AutocorrectMode,
    ignore_disable_comments: bool,
    path: Arc<str>,
    target_ruby_version: RubyVersion,
    source_encoding: SourceEncoding,
    cop_config: Arc<CopConfig>,
    enabled_cops: HashSet<&'static str>,
    parser_warnings: Vec<(String, Range<usize>)>,
    line_starts: Vec<usize>,
    findings: Vec<Finding>,
    corrections: Vec<Correction>,
}

impl Context {
    pub(super) fn new(
        autocorrect: AutocorrectMode,
        ignore_disable_comments: bool,
        path: impl Into<Arc<str>>,
        target_ruby_version: RubyVersion,
        source_encoding: SourceEncoding,
        cop_config: Arc<CopConfig>,
    ) -> Self {
        Self {
            autocorrect,
            ignore_disable_comments,
            path: path.into(),
            target_ruby_version,
            source_encoding,
            cop_config,
            enabled_cops: HashSet::new(),
            parser_warnings: Vec::new(),
            line_starts: Vec::new(),
            findings: Vec::new(),
            corrections: Vec::new(),
        }
    }

    pub(super) fn target_ruby_version(&self) -> RubyVersion {
        self.target_ruby_version
    }

    pub(super) fn set_enabled_cops(&mut self, cops: impl Iterator<Item = &'static str>) {
        self.enabled_cops = cops.collect();
    }

    pub(super) fn set_parser_warnings<'pr>(
        &mut self,
        warnings: impl Iterator<Item = ruby_prism::Diagnostic<'pr>>,
    ) {
        self.parser_warnings = warnings
            .map(|warning| {
                let location = warning.location();
                (
                    warning.message().to_string(),
                    location.start_offset()..location.end_offset(),
                )
            })
            .collect();
    }

    pub(super) fn parser_warning_at(&self, offset: usize, message: &str) -> bool {
        self.parser_warnings
            .iter()
            .any(|(warning, range)| range.start == offset && warning.contains(message))
    }

    fn cop_enabled(&self, cop_name: &str) -> bool {
        self.enabled_cops.contains(cop_name)
    }

    pub(super) fn source_encoding(&self) -> SourceEncoding {
        self.source_encoding
    }

    fn config_value(&self, cop_name: &str, key: &str) -> Option<&str> {
        self.cop_config.value(cop_name, key)
    }

    pub(super) fn reporter(&mut self, cop_name: &'static str) -> Reporter<'_> {
        Reporter {
            cop_name,
            context: self,
        }
    }

    pub(super) fn cop_context<'context, 'pr>(
        &'context mut self,
        cop_name: &'static str,
        source: &'pr str,
        ancestors: &'pr [Node<'pr>],
    ) -> CopContext<'context, 'pr> {
        if self.line_starts.is_empty() {
            self.line_starts.push(0);
            self.line_starts.extend(
                source
                    .match_indices('\n')
                    .map(|(offset, _)| offset + 1)
                    .filter(|offset| *offset < source.len()),
            );
        }
        CopContext::new(
            Reporter {
                cop_name,
                context: self,
            },
            source,
            ancestors,
        )
    }

    fn line_index(&self, offset: usize) -> usize {
        self.line_starts
            .partition_point(|start| *start <= offset)
            .saturating_sub(1)
    }

    fn line_start_at(&self, index: usize) -> usize {
        self.line_starts.get(index).copied().unwrap_or(0)
    }

    pub(super) fn report(
        &mut self,
        cop_name: &'static str,
        message: impl Into<String>,
        offense: impl ByteRange,
    ) {
        self.record(cop_name, message.into(), offense.offsets(), None);
    }

    pub(super) fn replace(
        &mut self,
        cop_name: &'static str,
        message: impl Into<String>,
        offense: impl ByteRange,
        edit: impl ByteRange,
        replacement: impl Into<String>,
    ) {
        self.record(
            cop_name,
            message.into(),
            offense.offsets(),
            Some(vec![Edit {
                range: edit.offsets(),
                replacement: replacement.into(),
            }]),
        );
    }

    pub(super) fn replace_many(
        &mut self,
        cop_name: &'static str,
        message: impl Into<String>,
        offense: impl ByteRange,
        edits: Vec<(Range<usize>, String)>,
    ) {
        self.record(
            cop_name,
            message.into(),
            offense.offsets(),
            Some(
                edits
                    .into_iter()
                    .map(|(range, replacement)| Edit { range, replacement })
                    .collect(),
            ),
        );
    }

    pub(super) fn remove(
        &mut self,
        cop_name: &'static str,
        message: impl Into<String>,
        offense: impl ByteRange,
        edit: impl ByteRange,
    ) {
        self.replace(cop_name, message, offense, edit, "");
    }

    pub(super) fn insert(
        &mut self,
        cop_name: &'static str,
        message: impl Into<String>,
        offense: impl ByteRange,
        offset: usize,
        text: impl Into<String>,
    ) {
        self.replace(cop_name, message, offense, offset..offset, text);
    }

    fn record(
        &mut self,
        cop_name: &'static str,
        message: String,
        offense: Range<usize>,
        correction: Option<Vec<Edit>>,
    ) {
        let correctable = correction.is_some();
        self.record_with_correctability(cop_name, message, offense, correction, correctable);
    }

    fn record_with_correctability(
        &mut self,
        cop_name: &'static str,
        message: String,
        offense: Range<usize>,
        correction: Option<Vec<Edit>>,
        correctable: bool,
    ) {
        if !self.cop_config.cop_applies_to_path(cop_name, &self.path) {
            return;
        }
        if self.findings.iter().any(|finding| {
            finding.cop_name == cop_name
                && finding.message == message
                && finding.start_offset == offense.start
                && finding.end_offset == offense.end
        }) {
            return;
        }
        let finding_index = self.findings.len();
        self.findings.push(Finding {
            cop_name,
            message,
            correctable,
            corrected: false,
            start_offset: offense.start,
            end_offset: offense.end,
        });
        if self.autocorrect.enabled_for(&self.cop_config, cop_name) {
            if let Some(edits) = correction {
                self.corrections.push(Correction {
                    finding_index,
                    edits,
                    indirect: !correctable,
                });
            }
        }
    }

    pub(super) fn replace_indirectly(
        &mut self,
        cop_name: &'static str,
        message: impl Into<String>,
        offense: impl ByteRange,
        edit: impl ByteRange,
        replacement: impl Into<String>,
    ) {
        self.record_with_correctability(
            cop_name,
            message.into(),
            offense.offsets(),
            Some(vec![Edit {
                range: edit.offsets(),
                replacement: replacement.into(),
            }]),
            false,
        );
    }

    pub(super) fn replace_many_indirectly(
        &mut self,
        cop_name: &'static str,
        message: impl Into<String>,
        offense: impl ByteRange,
        edits: Vec<(Range<usize>, String)>,
    ) {
        self.record_with_correctability(
            cop_name,
            message.into(),
            offense.offsets(),
            Some(
                edits
                    .into_iter()
                    .map(|(range, replacement)| Edit { range, replacement })
                    .collect(),
            ),
            false,
        );
    }

    pub(super) fn finish(mut self, source: &str) -> Inspection {
        let disabled = if self.ignore_disable_comments {
            vec![false; self.findings.len()]
        } else {
            disabled_findings(source, &self.findings)
        };
        self.corrections
            .retain(|correction| !disabled[correction.finding_index]);
        let correction_findings = self
            .corrections
            .iter()
            .map(|correction| correction.finding_index)
            .collect::<HashSet<_>>();
        let correction_intents = self
            .corrections
            .iter()
            .map(|correction| {
                (
                    correction.finding_index,
                    correction
                        .edits
                        .iter()
                        .map(|edit| (edit.range.clone(), edit.replacement.clone()))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<HashMap<_, _>>();
        let (accepted, subsumed) = accepted_corrections(source, self.corrections);
        let mut edits = Vec::new();
        for correction in accepted {
            self.findings[correction.finding_index].corrected = true;
            edits.extend(correction.edits);
        }
        for (finding_index, finding) in self.findings.iter_mut().enumerate() {
            if !correction_findings.contains(&finding_index)
                && edits.iter().any(|edit| {
                    if edit.range.start == edit.range.end {
                        !edit.replacement.is_empty()
                            && finding.start_offset <= edit.range.start
                            && edit.range.start <= finding.end_offset
                    } else {
                        edit.range.start <= finding.start_offset
                            && finding.end_offset <= edit.range.end
                            || finding.start_offset <= edit.range.start
                                && edit.range.end <= finding.end_offset
                    }
                })
            {
                finding.corrected = true;
            }
        }
        for (finding_index, intents) in correction_intents {
            if !intents.is_empty()
                && intents.iter().all(|(range, replacement)| {
                    edits
                        .iter()
                        .any(|edit| edit.range == *range && edit.replacement == *replacement)
                })
            {
                self.findings[finding_index].corrected = true;
            }
        }
        for finding_index in subsumed {
            self.findings[finding_index].corrected = true;
        }
        self.findings = self
            .findings
            .into_iter()
            .enumerate()
            .filter_map(|(index, finding)| (!disabled[index]).then_some(finding))
            .collect();
        Inspection {
            findings: self.findings,
            corrected_source: apply_edits(source, edits),
        }
    }
}

#[derive(Clone, Default)]
struct DisabledState {
    all: bool,
    cops: Arc<HashSet<String>>,
}

impl DisabledState {
    fn update(&mut self, names: &[&str], disabled: bool) {
        for name in names {
            if name.eq_ignore_ascii_case("all") {
                self.all = disabled;
            } else {
                // RuboCop accepts both `Metrics` and the frequently used
                // `Metrics/` spelling as department-wide directives.
                let name = name.trim_end_matches('/');
                if disabled {
                    Arc::make_mut(&mut self.cops).insert(name.to_string());
                } else {
                    Arc::make_mut(&mut self.cops).remove(name);
                }
            }
        }
    }

    fn contains(&self, cop: &str) -> bool {
        self.all
            || self.cops.contains(cop)
            || cop
                .split_once('/')
                .is_some_and(|(department, _)| self.cops.contains(department))
    }
}

fn disabled_findings(source: &str, findings: &[Finding]) -> Vec<bool> {
    let directive_comments = ruby_prism::parse(source.as_bytes())
        .comments()
        .map(|comment| comment.location().start_offset())
        .collect::<HashSet<_>>();
    let mut state = DisabledState::default();
    let mut line_starts = Vec::new();
    let mut states = Vec::new();
    let mut offset = 0;
    for physical_line in source.split_inclusive('\n') {
        line_starts.push(offset);
        let line = physical_line.strip_suffix('\n').unwrap_or(physical_line);
        let mut line_state = state.clone();
        if let Some((comment_at, action, names)) = cop_directive(line)
            .filter(|(comment_at, _, _)| directive_comments.contains(&(offset + comment_at)))
        {
            match action {
                DirectiveAction::Disable | DirectiveAction::Enable => {
                    let disabled = action == DirectiveAction::Disable;
                    if line[..comment_at].trim().is_empty() {
                        if disabled {
                            state.update(&names, true);
                            line_state = state.clone();
                        } else {
                            line_state = state.clone();
                            state.update(&names, false);
                        }
                    } else {
                        line_state.update(&names, disabled);
                    }
                }
            }
        }
        states.push(line_state);
        offset += physical_line.len();
    }
    if states.is_empty() {
        line_starts.push(0);
        states.push(state);
    }

    findings
        .iter()
        .map(|finding| {
            let line = line_starts
                .partition_point(|start| *start <= finding.start_offset)
                .saturating_sub(1);
            states[line].contains(finding.cop_name)
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DirectiveAction {
    Disable,
    Enable,
}

fn cop_directive(line: &str) -> Option<(usize, DirectiveAction, Vec<&str>)> {
    let comment_at = line.find("# rubocop:")?;
    let directive = line[comment_at + "# rubocop:".len()..].trim_start();
    let (action, names) = if let Some(names) = directive_action_names(directive, "disable") {
        (DirectiveAction::Disable, names)
    } else if let Some(names) = directive_action_names(directive, "todo") {
        (DirectiveAction::Disable, names)
    } else if let Some(names) = directive_action_names(directive, "enable") {
        (DirectiveAction::Enable, names)
    } else {
        return None;
    };
    let names = names
        .split_once("--")
        .map_or(names, |(names, _reason)| names)
        .split(',')
        .flat_map(str::split_whitespace)
        .map(|name| name.split_once('(').map_or(name, |(cop, _reason)| cop))
        .map(|name| name.trim_end_matches(':'))
        .filter(|name| !name.is_empty())
        .collect::<Vec<_>>();
    Some((comment_at, action, names))
}

fn directive_action_names<'a>(directive: &'a str, action: &str) -> Option<&'a str> {
    let names = directive.strip_prefix(action)?;
    names
        .chars()
        .next()
        .is_none_or(char::is_whitespace)
        .then_some(names)
}

#[cfg(test)]
#[path = "diagnostic_tests.rs"]
mod tests;
