use ruby_prism::Location;
use std::ops::Range;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RubyVersion {
    major: u16,
    minor: u16,
}

impl RubyVersion {
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    pub fn parse(value: &str) -> Option<Self> {
        let mut parts = value.trim_matches(['\'', '"']).split('.');
        Some(Self::new(
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
        ))
    }

    pub(super) fn at_least(self, major: u16, minor: u16) -> bool {
        (self.major, self.minor) >= (major, minor)
    }
}

impl Default for RubyVersion {
    fn default() -> Self {
        Self::new(2, 7)
    }
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
    findings: Vec<Finding>,
    edits: Vec<Edit>,
}

impl Context {
    pub(super) fn new(autocorrect: bool, target_ruby_version: RubyVersion) -> Self {
        Self {
            autocorrect,
            target_ruby_version,
            findings: Vec::new(),
            edits: Vec::new(),
        }
    }

    pub(super) fn target_ruby_version(&self) -> RubyVersion {
        self.target_ruby_version
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

    #[test]
    fn reports_uncorrectable_findings_without_changing_source() {
        let mut context = Context::new(true, RubyVersion::default());
        context.report("Lint/Example", "Example offense.", 1..3);

        let inspection = context.finish("abcd");

        assert_eq!(inspection.corrected_source, "abcd");
        assert!(!inspection.findings[0].correctable);
        assert!(!inspection.findings[0].corrected);
    }

    #[test]
    fn records_correctability_without_applying_disabled_corrections() {
        let mut context = Context::new(false, RubyVersion::default());
        context.replace("Style/Example", "Example offense.", (1, 3), 1..3, "X");

        let inspection = context.finish("abcd");

        assert_eq!(inspection.corrected_source, "abcd");
        assert!(inspection.findings[0].correctable);
        assert!(!inspection.findings[0].corrected);
    }

    #[test]
    fn applies_each_correction_intent() {
        let mut context = Context::new(true, RubyVersion::default());
        context.insert("Layout/Example", "Insert.", (0, 1), 1, " ");
        context.replace("Style/Example", "Replace.", (1, 2), (1, 2), "B");
        context.remove("Style/Example", "Remove.", 3..4, 3..4);

        let inspection = context.finish("abcd");

        assert_eq!(inspection.corrected_source, "a Bc");
        assert!(inspection.findings.iter().all(|finding| finding.corrected));
    }
}
