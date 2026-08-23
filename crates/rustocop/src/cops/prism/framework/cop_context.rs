use ruby_prism::{CallNode, Node};

use super::diagnostic::{ByteRange, Reporter};
use super::{CopPolicy, CorrectionPlan, SourceFile};
use crate::config::RubyVersion;

/// The complete, read-only inspection view available to a cop callback, plus
/// its cop-scoped diagnostic reporter.
pub(super) struct CopContext<'context, 'pr> {
    reporter: Reporter<'context>,
    source: &'pr str,
    ancestors: &'pr [Node<'pr>],
}

// This is the supported cop-authoring surface. Not every primitive needs an
// in-tree consumer before the next cop family uses it.
#[allow(dead_code)]
impl<'context, 'pr> CopContext<'context, 'pr> {
    pub(super) fn new(
        reporter: Reporter<'context>,
        source: &'pr str,
        ancestors: &'pr [Node<'pr>],
    ) -> Self {
        Self {
            reporter,
            source,
            ancestors,
        }
    }

    pub(super) fn source(&self) -> &'pr str {
        self.source
    }

    pub(super) fn source_file(&self) -> SourceFile<'pr> {
        SourceFile::new(self.source)
    }

    pub(super) fn path(&self) -> &str {
        self.reporter.path()
    }

    pub(super) fn ancestors(&self) -> &'pr [Node<'pr>] {
        self.ancestors
    }

    pub(super) fn parent(&self) -> Option<&Node<'pr>> {
        self.ancestors.last()
    }

    pub(super) fn nearest_call(&self) -> Option<CallNode<'pr>> {
        self.ancestors.iter().rev().find_map(Node::as_call_node)
    }

    pub(super) fn inside_method(&self) -> bool {
        self.ancestors
            .iter()
            .rev()
            .any(|node| node.as_def_node().is_some())
    }

    pub(super) fn target_ruby_version(&self) -> RubyVersion {
        self.reporter.target_ruby_version()
    }

    pub(super) fn autocorrect_enabled(&self) -> bool {
        self.reporter.autocorrect_enabled()
    }

    pub(super) fn config_value(&self, key: &str) -> Option<&str> {
        self.reporter.config_value(key)
    }

    pub(super) fn related_config_value(&self, cop_name: &str, key: &str) -> Option<&str> {
        self.reporter.related_config_value(cop_name, key)
    }

    pub(super) fn related_config_values(&self, cop_name: &str, key: &str) -> &[String] {
        self.reporter.related_config_values(cop_name, key)
    }

    pub(super) fn related_config_explicit(&self, cop_name: &str, key: &str) -> bool {
        self.reporter.related_config_explicit(cop_name, key)
    }

    pub(super) fn related_config_map(
        &self,
        cop_name: &str,
        key: &str,
    ) -> Option<&std::collections::HashMap<String, String>> {
        self.reporter.related_config_map(cop_name, key)
    }

    pub(super) fn config_bool(&self, key: &str, default: bool) -> bool {
        self.reporter.config_bool(key, default)
    }

    pub(super) fn config_usize(&self, key: &str, default: usize) -> usize {
        self.reporter.config_usize(key, default)
    }

    pub(super) fn config_values(&self, key: &str) -> &[String] {
        self.reporter.config_values(key)
    }

    pub(super) fn config_contains(&self, key: &str) -> bool {
        self.reporter.config_contains(key)
    }

    pub(super) fn config_map(
        &self,
        key: &str,
    ) -> Option<&std::collections::HashMap<String, String>> {
        self.reporter.config_map(key)
    }

    pub(super) fn config_symbol_map(
        &self,
        key: &str,
    ) -> Option<&std::collections::HashMap<String, String>> {
        self.reporter.config_symbol_map(key)
    }

    pub(super) fn policy(&self) -> CopPolicy<'_> {
        self.reporter.policy()
    }

    pub(super) fn report(&mut self, message: impl Into<String>, offense: impl ByteRange) {
        self.reporter.report(message, offense);
    }

    /// Registers an offense and lets the cop describe its autocorrection with
    /// the same transaction-oriented shape as RuboCop's `add_offense` block.
    pub(super) fn add_offense(
        &mut self,
        offense: impl ByteRange,
        message: impl Into<String>,
        correction: impl FnOnce(&mut CorrectionPlan),
    ) {
        let mut plan = CorrectionPlan::default();
        correction(&mut plan);
        self.apply_correction(message, offense, plan);
    }

    pub(super) fn replace(
        &mut self,
        message: impl Into<String>,
        offense: impl ByteRange,
        edit: impl ByteRange,
        replacement: impl Into<String>,
    ) {
        self.reporter.replace(message, offense, edit, replacement);
    }

    /// Replaces every code-only occurrence, excluding quoted strings and line
    /// comments. Prefer AST locations when a rule depends on Ruby structure.
    pub(super) fn replace_code(&mut self, old: &str, new: &str, message: &str) {
        for start in self.source_file().code_offsets(old) {
            self.replace(
                message,
                start..start + old.len(),
                start..start + old.len(),
                new,
            );
        }
    }

    /// Reports every code-only occurrence, excluding quoted strings and line
    /// comments. This is the diagnostic-only counterpart to `replace_code`.
    pub(super) fn report_code(&mut self, needle: &str, message: &str) {
        for start in self.source_file().code_offsets(needle) {
            self.report(message, start..start + needle.len());
        }
    }

    pub(super) fn replace_indirectly(
        &mut self,
        message: impl Into<String>,
        offense: impl ByteRange,
        edit: impl ByteRange,
        replacement: impl Into<String>,
    ) {
        self.reporter
            .replace_indirectly(message, offense, edit, replacement);
    }

    pub(super) fn apply_correction_indirectly(
        &mut self,
        message: impl Into<String>,
        offense: impl ByteRange,
        correction: CorrectionPlan,
    ) {
        self.reporter
            .replace_many_indirectly(message, offense, correction.into_edits());
    }

    /// Applies several coordinated edits as one correction transaction.
    /// Conflicts or invalid ranges reject the complete transaction.
    pub(super) fn replace_many(
        &mut self,
        message: impl Into<String>,
        offense: impl ByteRange,
        edits: Vec<(std::ops::Range<usize>, String)>,
    ) {
        self.reporter.replace_many(message, offense, edits);
    }

    pub(super) fn apply_correction(
        &mut self,
        message: impl Into<String>,
        offense: impl ByteRange,
        correction: CorrectionPlan,
    ) {
        self.replace_many(message, offense, correction.into_edits());
    }

    pub(super) fn remove(
        &mut self,
        message: impl Into<String>,
        offense: impl ByteRange,
        edit: impl ByteRange,
    ) {
        self.reporter.remove(message, offense, edit);
    }

    pub(super) fn insert(
        &mut self,
        message: impl Into<String>,
        offense: impl ByteRange,
        offset: usize,
        text: impl Into<String>,
    ) {
        self.reporter.insert(message, offense, offset, text);
    }

    pub(super) fn replace_selector(
        &mut self,
        call: &CallNode<'_>,
        message: impl Into<String>,
        replacement: impl Into<String>,
    ) {
        if let Some(selector) = call.message_loc() {
            self.replace(message, &selector, &selector, replacement);
        }
    }

    pub(super) fn report_selector(&mut self, call: &CallNode<'_>, message: impl Into<String>) {
        if let Some(selector) = call.message_loc() {
            self.report(message, &selector);
        }
    }

    pub(super) fn report_call(&mut self, call: &CallNode<'_>, message: impl Into<String>) {
        self.report(message, call.location());
    }

    pub(super) fn replace_call(
        &mut self,
        call: &CallNode<'_>,
        message: impl Into<String>,
        replacement: impl Into<String>,
    ) {
        let location = call.location();
        self.replace(message, &location, &location, replacement);
    }

    pub(super) fn remove_call(&mut self, call: &CallNode<'_>, message: impl Into<String>) {
        let location = call.location();
        self.remove(message, &location, &location);
    }

    pub(super) fn report_node(&mut self, node: &Node<'_>, message: impl Into<String>) {
        self.report(message, node.location());
    }

    pub(super) fn replace_node(
        &mut self,
        node: &Node<'_>,
        message: impl Into<String>,
        replacement: impl Into<String>,
    ) {
        let location = node.location();
        self.replace(message, &location, &location, replacement);
    }

    pub(super) fn remove_node(&mut self, node: &Node<'_>, message: impl Into<String>) {
        let location = node.location();
        self.remove(message, &location, &location);
    }

    /// Removes an AST element and the adjacent separator/whitespace supplied by
    /// its sibling locations. This covers arguments, arrays, hashes, parameters,
    /// and rescue lists without making each cop rediscover comma ownership.
    pub(super) fn remove_list_element(
        &mut self,
        node: &Node<'_>,
        previous: Option<&Node<'_>>,
        next: Option<&Node<'_>>,
        message: impl Into<String>,
    ) {
        let location = node.location();
        let edit = if let Some(next) = next {
            location.start_offset()..next.location().start_offset()
        } else if let Some(previous) = previous {
            previous.location().end_offset()..location.end_offset()
        } else {
            location.start_offset()..location.end_offset()
        };
        self.remove(message, &location, edit);
    }

    pub(super) fn wrap_node(
        &mut self,
        node: &Node<'_>,
        message: impl Into<String>,
        prefix: &str,
        suffix: &str,
    ) {
        let replacement = format!("{prefix}{}{suffix}", self.source_file().node(node));
        self.replace_node(node, message, replacement);
    }

    pub(super) fn remove_statement(&mut self, node: &Node<'_>, message: impl Into<String>) {
        let location = node.location();
        let line = self.source_file().line_range(location.start_offset());
        self.remove(message, &location, line);
    }

    pub(super) fn insert_before(
        &mut self,
        node: &Node<'_>,
        message: impl Into<String>,
        text: impl Into<String>,
    ) {
        self.insert(
            message,
            node.location(),
            node.location().start_offset(),
            text,
        );
    }

    pub(super) fn insert_after(
        &mut self,
        node: &Node<'_>,
        message: impl Into<String>,
        text: impl Into<String>,
    ) {
        self.insert(message, node.location(), node.location().end_offset(), text);
    }
}

#[cfg(test)]
#[path = "cop_context_tests.rs"]
mod tests;
