use ruby_prism::{Location, Node};
use std::cmp::Reverse;
use std::ops::Range;
use std::sync::Arc;

use super::correction_engine::{accepted_corrections, apply_edits, Correction, Edit};
use super::{CopContext, CopPolicy};
use crate::config::{CopConfig, RubyVersion};

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
    autocorrect: bool,
    path: Arc<str>,
    target_ruby_version: RubyVersion,
    cop_config: Arc<CopConfig>,
    findings: Vec<Finding>,
    corrections: Vec<Correction>,
}

impl Context {
    pub(super) fn new(
        autocorrect: bool,
        path: impl Into<Arc<str>>,
        target_ruby_version: RubyVersion,
        cop_config: Arc<CopConfig>,
    ) -> Self {
        Self {
            autocorrect,
            path: path.into(),
            target_ruby_version,
            cop_config,
            findings: Vec::new(),
            corrections: Vec::new(),
        }
    }

    pub(super) fn target_ruby_version(&self) -> RubyVersion {
        self.target_ruby_version
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
        CopContext::new(
            Reporter {
                cop_name,
                context: self,
            },
            source,
            ancestors,
        )
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

    fn replace_many(
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
        let finding_index = self.findings.len();
        self.findings.push(Finding {
            cop_name,
            message,
            correctable,
            corrected: false,
            start_offset: offense.start,
            end_offset: offense.end,
        });
        if self.autocorrect {
            if let Some(edits) = correction {
                self.corrections.push(Correction {
                    finding_index,
                    edits,
                });
            }
        }
    }

    fn replace_indirectly(
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

    pub(super) fn finish(mut self, source: &str) -> Inspection {
        let (accepted, subsumed) = accepted_corrections(source, self.corrections);
        let mut edits = Vec::new();
        for correction in accepted {
            self.findings[correction.finding_index].corrected = true;
            edits.extend(correction.edits);
        }
        for finding_index in subsumed {
            self.findings[finding_index].corrected = true;
        }
        self.findings.sort_by_key(|finding| {
            (
                finding.start_offset,
                Reverse(finding.end_offset),
                finding.cop_name,
            )
        });
        Inspection {
            findings: self.findings,
            corrected_source: apply_edits(source, edits),
        }
    }
}

#[cfg(test)]
#[path = "diagnostic_tests.rs"]
mod tests;
