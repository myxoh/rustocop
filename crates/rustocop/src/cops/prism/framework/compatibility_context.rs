use std::borrow::Cow;
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

/// Source-oriented RuboCop callbacks still describe ranges as byte offsets.
/// Keeping that representation accepted by the compatibility corrector lets
/// investigation callbacks move onto `ProcessedSource` without first
/// rewriting otherwise source-shaped range logic. Parser-shaped node ranges
/// continue to use character offsets through `CompatibilitySourceRange`.
impl CompatibilityRange for Range<usize> {
    fn byte_range(self, _buffer: &SourceBuffer<'_>) -> Option<Range<usize>> {
        Some(self)
    }
}

impl CompatibilityRange for ruby_prism::Location<'_> {
    fn byte_range(self, _buffer: &SourceBuffer<'_>) -> Option<Range<usize>> {
        Some(self.start_offset()..self.end_offset())
    }
}

impl CompatibilityRange for &ruby_prism::Location<'_> {
    fn byte_range(self, _buffer: &SourceBuffer<'_>) -> Option<Range<usize>> {
        Some(self.start_offset()..self.end_offset())
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

    pub(super) fn swap(&mut self, left: impl CompatibilityRange, right: impl CompatibilityRange) {
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
    prism_result: Option<&'processed ruby_prism::ParseResult<'source>>,
    buffer: &'processed SourceBuffer<'source>,
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
            prism_result: None,
            buffer: processed_source.buffer(),
        }
    }

    pub(super) fn new_with_prism(
        context: &'context mut Context,
        cop_name: &'static str,
        processed_source: &'processed ProcessedSource<'source>,
        prism_result: &'processed ruby_prism::ParseResult<'source>,
    ) -> Self {
        Self {
            reporter: context.reporter(cop_name),
            processed_source,
            prism_result: Some(prism_result),
            buffer: processed_source.buffer(),
        }
    }

    pub(super) fn source(&self) -> &'source str {
        self.processed_source.raw_source()
    }

    pub(super) fn path(&self) -> &str {
        self.reporter.path()
    }

    pub(super) fn source_file(&self) -> super::SourceFile<'source> {
        super::SourceFile::new(self.source())
    }

    /// Literal locations from the Prism result already produced by the engine.
    ///
    /// Keeping these helpers on the compatibility context is important: the
    /// source-only `SourceFile` fallback has to parse independently because it
    /// has no parser-result lifetime. Compatibility cops must never pay that
    /// fallback cost.
    pub(super) fn literal_ranges(&self) -> Vec<Range<usize>> {
        super::SourceFile::literal_ranges_from(self.prism_result())
    }

    pub(super) fn heredoc_ranges(&self) -> Vec<Range<usize>> {
        super::SourceFile::heredoc_ranges_from(self.prism_result())
    }

    pub(super) fn comment_ranges(&self) -> Vec<Range<usize>> {
        super::SourceFile::comment_ranges_from(self.prism_result())
    }

    pub(super) fn line_index(&self, offset: usize) -> usize {
        self.buffer.line_index_for_byte(offset)
    }

    pub(super) fn line_start_at(&self, index: usize) -> usize {
        self.buffer.line_start_byte_at_index(index)
    }

    pub(super) fn line_at(&self, index: usize) -> &'source str {
        self.buffer.source_line(index + 1)
    }

    pub(super) fn processed_source(&self) -> &'processed ProcessedSource<'source> {
        self.processed_source
    }

    pub(super) fn prism_result(&self) -> &'processed ruby_prism::ParseResult<'source> {
        self.prism_result
            .expect("investigation compatibility context has a shared Prism result")
    }

    pub(super) fn source_buffer(&self) -> &'processed SourceBuffer<'source> {
        self.buffer
    }

    pub(super) fn source_range(&self, node: NodeRef<'_>) -> Option<SourceRange<'_, 'source>> {
        let range = node.source_range()?;
        Some(SourceRange::new(self.buffer, range.start, range.end))
    }

    /// Return the runtime value RuboCop's Parser-shaped AST exposes for a
    /// plain string node. Prism retains `__FILE__` as a distinct source node,
    /// while Parser folds it into a `str` containing the current path.
    pub(super) fn string_content<'node>(&self, node: NodeRef<'node>) -> Option<Cow<'node, str>> {
        if let Some(content) = node.str_content() {
            Some(Cow::Borrowed(content))
        } else if node.kind() == "__FILE__" {
            Some(Cow::Owned(self.processed_source.file_path().to_owned()))
        } else {
            None
        }
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

    pub(super) fn range_between(
        &self,
        begin_pos: usize,
        end_pos: usize,
    ) -> CompatibilitySourceRange {
        CompatibilitySourceRange { begin_pos, end_pos }
    }

    pub(super) fn range_source(&self, range: &CompatibilitySourceRange) -> &'source str {
        self.buffer.slice(range.character_range())
    }

    /// RuboCop asks whether the token immediately before a comment ends at
    /// the comment's beginning. Prism gives us authoritative comment ranges,
    /// but our intentionally small compatibility lexer can omit an earlier
    /// token after syntax it does not model. At a comment boundary this is
    /// equivalent to checking the preceding source character for whitespace.
    pub(super) fn comment_immediately_follows_code(
        &self,
        comment: &crate::rubocop::ast::processed_source::SourceComment,
    ) -> bool {
        let begin = comment.range.start;
        begin > 0
            && !self
                .range_source(&self.range_between(begin - 1, begin))
                .chars()
                .all(char::is_whitespace)
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

    pub(super) fn config_contains(&self, key: &str) -> bool {
        self.reporter.config_contains(key)
    }

    pub(super) fn config_explicit(&self, key: &str) -> bool {
        self.reporter.config_explicit(key)
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

    pub(super) fn related_config_value(&self, cop_name: &str, key: &str) -> Option<&str> {
        self.reporter.related_config_value(cop_name, key)
    }

    pub(super) fn related_config_values(&self, cop_name: &str, key: &str) -> &[String] {
        self.reporter.related_config_values(cop_name, key)
    }

    pub(super) fn related_config_map(
        &self,
        cop_name: &str,
        key: &str,
    ) -> Option<&std::collections::HashMap<String, String>> {
        self.reporter.related_config_map(cop_name, key)
    }

    pub(super) fn related_config_explicit(&self, cop_name: &str, key: &str) -> bool {
        self.reporter.related_config_explicit(cop_name, key)
    }

    pub(super) fn related_cop_normally_enabled(&self, cop_name: &str) -> bool {
        self.reporter.related_cop_normally_enabled(cop_name)
    }

    pub(super) fn cop_enabled(&self, cop_name: &str) -> bool {
        self.reporter.cop_enabled(cop_name)
    }

    pub(super) fn autocorrect_enabled(&self) -> bool {
        self.reporter.autocorrect_enabled()
    }

    pub(super) fn source_encoding(&self) -> crate::config::SourceEncoding {
        self.reporter.source_encoding()
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

    pub(super) fn report_bytes(&mut self, message: Vec<u8>, offense: impl CompatibilityRange) {
        if let Some(offense) = offense.byte_range(&self.buffer) {
            self.reporter.report_bytes(message, offense);
        }
    }

    pub(super) fn replace(
        &mut self,
        message: impl Into<String>,
        offense: impl CompatibilityRange,
        edit: impl CompatibilityRange,
        replacement: impl Into<String>,
    ) {
        let (Some(offense), Some(edit)) = (
            offense.byte_range(&self.buffer),
            edit.byte_range(&self.buffer),
        ) else {
            return;
        };
        self.reporter.replace(message, offense, edit, replacement);
    }

    pub(super) fn replace_many(
        &mut self,
        message: impl Into<String>,
        offense: impl CompatibilityRange,
        edits: Vec<(Range<usize>, String)>,
    ) {
        let Some(offense) = offense.byte_range(&self.buffer) else {
            return;
        };
        self.reporter.replace_many(message, offense, edits);
    }

    pub(super) fn remove(
        &mut self,
        message: impl Into<String>,
        offense: impl CompatibilityRange,
        edit: impl CompatibilityRange,
    ) {
        let (Some(offense), Some(edit)) = (
            offense.byte_range(&self.buffer),
            edit.byte_range(&self.buffer),
        ) else {
            return;
        };
        self.reporter.remove(message, offense, edit);
    }

    pub(super) fn insert(
        &mut self,
        message: impl Into<String>,
        offense: impl CompatibilityRange,
        offset: usize,
        text: impl Into<String>,
    ) {
        let Some(offense) = offense.byte_range(&self.buffer) else {
            return;
        };
        self.reporter.insert(message, offense, offset, text);
    }

    pub(super) fn replace_indirectly(
        &mut self,
        message: impl Into<String>,
        offense: impl CompatibilityRange,
        edit: impl CompatibilityRange,
        replacement: impl Into<String>,
    ) {
        let (Some(offense), Some(edit)) = (
            offense.byte_range(&self.buffer),
            edit.byte_range(&self.buffer),
        ) else {
            return;
        };
        self.reporter
            .replace_indirectly(message, offense, edit, replacement);
    }

    pub(super) fn apply_correction_indirectly(
        &mut self,
        message: impl Into<String>,
        offense: impl CompatibilityRange,
        correction: CorrectionPlan,
    ) {
        let Some(offense) = offense.byte_range(&self.buffer) else {
            return;
        };
        self.reporter
            .replace_many_indirectly(message, offense, correction.into_edits());
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
