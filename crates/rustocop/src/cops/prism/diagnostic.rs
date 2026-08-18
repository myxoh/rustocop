use ruby_prism::Location;
use std::ops::Range;
use std::sync::Arc;

use crate::config::{CopConfig, RubyVersion};

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

struct Edit {
    range: Range<usize>,
    replacement: String,
}

pub(crate) struct Context {
    autocorrect: bool,
    target_ruby_version: RubyVersion,
    cop_config: Arc<CopConfig>,
    findings: Vec<Finding>,
    edits: Vec<Edit>,
}

/// A diagnostic context already scoped to one cop. Rule helpers use this
/// instead of accepting and forwarding a separate cop-name argument.
pub(super) struct Reporter<'context> {
    cop_name: &'static str,
    context: &'context mut Context,
}

impl Context {
    pub(super) fn new(
        autocorrect: bool,
        target_ruby_version: RubyVersion,
        cop_config: Arc<CopConfig>,
    ) -> Self {
        Self {
            autocorrect,
            target_ruby_version,
            cop_config,
            findings: Vec::new(),
            edits: Vec::new(),
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
            Some(Edit {
                range: edit.offsets(),
                replacement: replacement.into(),
            }),
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
        edit: Option<Edit>,
    ) {
        let correctable = edit.is_some();
        let corrected = self.autocorrect && correctable;
        if self.autocorrect {
            self.edits.extend(edit);
        }
        self.findings.push(Finding {
            cop_name,
            message,
            correctable,
            corrected,
            start_offset: offense.start,
            end_offset: offense.end,
        });
    }

    pub(super) fn finish(mut self, source: &str) -> Inspection {
        self.findings
            .sort_by_key(|finding| (finding.start_offset, finding.end_offset, finding.cop_name));
        Inspection {
            findings: self.findings,
            corrected_source: apply_edits(source, self.edits),
        }
    }
}

impl Reporter<'_> {
    pub(super) fn target_ruby_version(&self) -> RubyVersion {
        self.context.target_ruby_version()
    }

    pub(super) fn config_value(&self, key: &str) -> Option<&str> {
        self.context.config_value(self.cop_name, key)
    }

    pub(super) fn related_config_value(&self, cop_name: &str, key: &str) -> Option<&str> {
        self.context.config_value(cop_name, key)
    }

    pub(super) fn report(&mut self, message: impl Into<String>, offense: impl ByteRange) {
        self.context.report(self.cop_name, message, offense);
    }

    pub(super) fn replace(
        &mut self,
        message: impl Into<String>,
        offense: impl ByteRange,
        edit: impl ByteRange,
        replacement: impl Into<String>,
    ) {
        self.context
            .replace(self.cop_name, message, offense, edit, replacement);
    }

    pub(super) fn remove(
        &mut self,
        message: impl Into<String>,
        offense: impl ByteRange,
        edit: impl ByteRange,
    ) {
        self.context.remove(self.cop_name, message, offense, edit);
    }

    pub(super) fn insert(
        &mut self,
        message: impl Into<String>,
        offense: impl ByteRange,
        offset: usize,
        text: impl Into<String>,
    ) {
        self.context
            .insert(self.cop_name, message, offense, offset, text);
    }
}

fn apply_edits(source: &str, mut edits: Vec<Edit>) -> String {
    edits.sort_by_key(|edit| (edit.range.start, edit.range.end));

    let mut accepted = Vec::with_capacity(edits.len());
    let mut previous_end = 0;
    for edit in edits {
        if edit.range.start < previous_end || edit.range.end > source.len() {
            continue;
        }
        previous_end = edit.range.end;
        accepted.push(edit);
    }

    let mut corrected = source.to_string();
    for edit in accepted.into_iter().rev() {
        corrected.replace_range(edit.range, &edit.replacement);
    }
    corrected
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(autocorrect: bool) -> Context {
        Context::new(
            autocorrect,
            RubyVersion::default(),
            Arc::new(CopConfig::default()),
        )
    }

    #[test]
    fn reports_uncorrectable_findings_without_changing_source() {
        let mut context = context(true);
        context.report("Lint/Example", "Example offense.", 1..3);

        let inspection = context.finish("abcd");

        assert_eq!(inspection.corrected_source, "abcd");
        assert!(!inspection.findings[0].correctable);
        assert!(!inspection.findings[0].corrected);
    }

    #[test]
    fn records_correctability_without_applying_disabled_corrections() {
        let mut context = context(false);
        context.replace("Style/Example", "Example offense.", (1, 3), 1..3, "X");

        let inspection = context.finish("abcd");

        assert_eq!(inspection.corrected_source, "abcd");
        assert!(inspection.findings[0].correctable);
        assert!(!inspection.findings[0].corrected);
    }

    #[test]
    fn applies_each_correction_intent() {
        let mut context = context(true);
        context.insert("Layout/Example", "Insert.", (0, 1), 1, " ");
        context.replace("Style/Example", "Replace.", (1, 2), (1, 2), "B");
        context.remove("Style/Example", "Remove.", 3..4, 3..4);

        let inspection = context.finish("abcd");

        assert_eq!(inspection.corrected_source, "a Bc");
        assert!(inspection.findings.iter().all(|finding| finding.corrected));
    }

    #[test]
    fn reporter_scopes_every_intent_to_one_cop() {
        let mut context = context(true);
        {
            let mut reporter = context.reporter("Style/Example");
            reporter.report("Report.", 0..1);
            reporter.replace("Replace.", 1..2, 1..2, "B");
            reporter.insert("Insert.", 2..3, 2, "!");
        }

        let inspection = context.finish("abc");

        assert_eq!(inspection.corrected_source, "aB!c");
        assert!(inspection
            .findings
            .iter()
            .all(|finding| finding.cop_name == "Style/Example"));
    }
}
