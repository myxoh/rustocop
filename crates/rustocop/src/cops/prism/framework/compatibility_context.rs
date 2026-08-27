use std::ops::Range;

use super::{Context, CopPolicy, CorrectionPlan, Reporter};
use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::processed_source::ProcessedSource;
use crate::rubocop::ast::processed_source::{SourceComment, SourceToken};
use crate::rubocop::ast::source::{SourceBuffer, SourceRange};
use crate::rubocop::cop::mixin::allowed_methods::{AllowedMethods, ConfiguredMethod};
use crate::rubocop::cop::mixin::comments_help::CommentsHelp;
use crate::rubocop::cop::mixin::range_help::RangeHelp;

pub(super) trait CompatibilityRange {
    fn byte_range(self, buffer: &SourceBuffer<'_>) -> Option<Range<usize>>;
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CompatibilitySourceRange {
    begin_pos: usize,
    end_pos: usize,
}

impl CompatibilitySourceRange {
    pub(super) fn begin_pos(&self) -> usize {
        self.begin_pos
    }

    pub(super) fn end_pos(&self) -> usize {
        self.end_pos
    }

    pub(super) fn character_range(&self) -> Range<usize> {
        self.begin_pos..self.end_pos
    }
}

impl CompatibilityRange for CompatibilitySourceRange {
    fn byte_range(self, buffer: &SourceBuffer<'_>) -> Option<Range<usize>> {
        Some(buffer.byte_position(self.begin_pos)?..buffer.byte_position(self.end_pos)?)
    }
}

impl CompatibilityRange for NodeRef<'_> {
    fn byte_range(self, buffer: &SourceBuffer<'_>) -> Option<Range<usize>> {
        let range = self.source_range()?;
        Some(buffer.byte_position(range.start)?..buffer.byte_position(range.end)?)
    }
}

impl CompatibilityRange for SourceRange<'_, '_> {
    fn byte_range(self, _buffer: &SourceBuffer<'_>) -> Option<Range<usize>> {
        Some(self.byte_range())
    }
}

impl CompatibilityRange for &SourceComment {
    fn byte_range(self, buffer: &SourceBuffer<'_>) -> Option<Range<usize>> {
        Some(buffer.byte_position(self.range.start)?..buffer.byte_position(self.range.end)?)
    }
}

impl CompatibilityRange for &SourceToken {
    fn byte_range(self, buffer: &SourceBuffer<'_>) -> Option<Range<usize>> {
        Some(buffer.byte_position(self.range.start)?..buffer.byte_position(self.range.end)?)
    }
}

pub(super) struct CompatibilityCorrector<'plan, 'source> {
    plan: &'plan mut CorrectionPlan,
    buffer: &'plan SourceBuffer<'source>,
}

impl CompatibilityCorrector<'_, '_> {
    pub(super) fn remove(&mut self, range: impl CompatibilityRange) {
        if let Some(range) = range.byte_range(self.buffer) {
            self.plan.remove(range);
        }
    }

    pub(super) fn replace(
        &mut self,
        range: impl CompatibilityRange,
        replacement: impl Into<String>,
    ) {
        if let Some(range) = range.byte_range(self.buffer) {
            self.plan.replace(range, replacement);
        }
    }

    pub(super) fn insert_before(
        &mut self,
        range: impl CompatibilityRange,
        text: impl Into<String>,
    ) {
        if let Some(range) = range.byte_range(self.buffer) {
            self.plan.replace(range.start..range.start, text);
        }
    }

    pub(super) fn insert_after(&mut self, range: impl CompatibilityRange, text: impl Into<String>) {
        if let Some(range) = range.byte_range(self.buffer) {
            self.plan.replace(range.end..range.end, text);
        }
    }

    pub(super) fn wrap(
        &mut self,
        range: impl CompatibilityRange,
        prefix: impl Into<String>,
        suffix: impl Into<String>,
    ) {
        if let Some(range) = range.byte_range(self.buffer) {
            self.plan.replace(range.start..range.start, prefix);
            self.plan.replace(range.end..range.end, suffix);
        }
    }

    pub(super) fn swap(
        &mut self,
        left: impl CompatibilityRange,
        right: impl CompatibilityRange,
    ) {
        let Some(left) = left.byte_range(self.buffer) else {
            return;
        };
        let Some(right) = right.byte_range(self.buffer) else {
            return;
        };
        self.plan.swap(self.buffer.source(), left, right);
    }
}

pub(super) struct CompatibilityCopContext<'context, 'processed, 'source> {
    reporter: Reporter<'context>,
    processed_source: &'processed ProcessedSource<'source>,
    buffer: SourceBuffer<'source>,
}

impl<'context, 'processed, 'source> CompatibilityCopContext<'context, 'processed, 'source> {
    pub(super) fn new(
        context: &'context mut Context,
        cop_name: &'static str,
        processed_source: &'processed ProcessedSource<'source>,
    ) -> Self {
        Self {
            reporter: context.reporter(cop_name),
            processed_source,
            buffer: processed_source.buffer(),
        }
    }

    pub(super) fn source(&self) -> &'source str {
        self.processed_source.raw_source()
    }

    pub(super) fn processed_source(&self) -> &'processed ProcessedSource<'source> {
        self.processed_source
    }

    pub(super) fn source_buffer(&self) -> &SourceBuffer<'source> {
        &self.buffer
    }

    pub(super) fn source_range(&self, node: NodeRef<'_>) -> Option<SourceRange<'_, 'source>> {
        let range = node.source_range()?;
        Some(SourceRange::new(&self.buffer, range.start, range.end))
    }

    pub(super) fn location_range(
        &self,
        node: NodeRef<'_>,
        name: &str,
    ) -> Option<CompatibilitySourceRange> {
        let (range, _) = node.loc(name)?;
        Some(CompatibilitySourceRange {
            begin_pos: range.start,
            end_pos: range.end,
        })
    }

    pub(super) fn location_source<'node>(
        &self,
        node: NodeRef<'node>,
        name: &str,
    ) -> Option<&'node str> {
        node.loc(name).map(|(_, source)| source.as_str())
    }

    pub(super) fn range_between(&self, begin_pos: usize, end_pos: usize) -> CompatibilitySourceRange {
        CompatibilitySourceRange { begin_pos, end_pos }
    }

    pub(super) fn range_source(&self, range: &CompatibilitySourceRange) -> &'source str {
        self.buffer.slice(range.character_range())
    }

    pub(super) fn owned_range(&self, range: SourceRange<'_, '_>) -> CompatibilitySourceRange {
        CompatibilitySourceRange {
            begin_pos: range.begin_pos(),
            end_pos: range.end_pos(),
        }
    }

    pub(super) fn owned_character_range(&self, range: Range<usize>) -> CompatibilitySourceRange {
        CompatibilitySourceRange {
            begin_pos: range.start,
            end_pos: range.end,
        }
    }

    pub(super) fn range_help(&self) -> RangeHelp<'_, 'source> {
        RangeHelp::new(&self.buffer)
    }

    pub(super) fn comments_help(&self) -> CommentsHelp<'_, 'source> {
        CommentsHelp::new(self.processed_source)
    }

    pub(super) fn directive_comment_enabled(
        &self,
        comment: &crate::rubocop::ast::processed_source::SourceComment,
    ) -> bool {
        let Some(marker) = comment.text.find("rubocop") else {
            return false;
        };
        let rest = comment.text[marker + "rubocop".len()..].trim_start();
        let Some(rest) = rest.strip_prefix(':') else {
            return false;
        };
        rest.trim_start().split_whitespace().next() == Some("enable")
    }

    pub(super) fn allowed_methods(&self) -> AllowedMethods {
        AllowedMethods::new(
            self.config_values("AllowedMethods").to_vec(),
            self.config_values("IgnoredMethods")
                .iter()
                .cloned()
                .map(ConfiguredMethod::Name)
                .collect(),
            self.config_values("ExcludedMethods")
                .iter()
                .cloned()
                .map(ConfiguredMethod::Name)
                .collect(),
        )
    }

    pub(super) fn config_value(&self, key: &str) -> Option<&str> {
        self.reporter.config_value(key)
    }

    pub(super) fn config_bool(&self, key: &str, default: bool) -> bool {
        self.reporter.config_bool(key, default)
    }

    pub(super) fn config_usize(&self, key: &str, default: usize) -> usize {
        self.config_value(key)
            .and_then(|value| value.parse().ok())
            .unwrap_or(default)
    }

    pub(super) fn config_values(&self, key: &str) -> &[String] {
        self.reporter.config_values(key)
    }

    pub(super) fn config_map(
        &self,
        key: &str,
    ) -> Option<&std::collections::HashMap<String, String>> {
        self.reporter.config_map(key)
    }

    pub(super) fn target_ruby_version(&self) -> crate::config::RubyVersion {
        self.reporter.target_ruby_version()
    }

    pub(super) fn policy(&self) -> CopPolicy<'_> {
        self.reporter.policy()
    }

    pub(super) fn report(&mut self, message: impl Into<String>, offense: impl CompatibilityRange) {
        if let Some(offense) = offense.byte_range(&self.buffer) {
            self.reporter.report(message, offense);
        }
    }

    pub(super) fn add_offense(
        &mut self,
        offense: impl CompatibilityRange,
        message: impl Into<String>,
        correction: impl FnOnce(&mut CompatibilityCorrector<'_, 'source>),
    ) {
        let Some(offense) = offense.byte_range(&self.buffer) else {
            return;
        };
        let mut plan = CorrectionPlan::default();
        correction(&mut CompatibilityCorrector {
            plan: &mut plan,
            buffer: &self.buffer,
        });
        self.reporter
            .replace_many(message, offense, plan.into_edits());
    }
}
