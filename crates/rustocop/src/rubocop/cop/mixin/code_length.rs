// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/code_length.rb
// Source SHA-256: 2d44959d429ffbf9ddc5a6f991a5fcea0c727759838388e7975d9755401a3625

use std::ops::Range;

use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::processed_source::ProcessedSource;

use super::advanced::{code_length_for_node, CodeLengthOptions};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CodeLengthOffense {
    pub(crate) location: Range<usize>,
    pub(crate) message: String,
    pub(crate) length: usize,
}

pub(crate) struct CodeLength {
    cop_label: String,
    max: usize,
    count_comments: bool,
    count_as_one: Vec<String>,
    lsp_enabled: bool,
}

impl CodeLength {
    pub(crate) fn new(
        cop_label: impl Into<String>,
        max: usize,
        count_comments: bool,
        count_as_one: Vec<String>,
        lsp_enabled: bool,
    ) -> Self {
        Self {
            cop_label: cop_label.into(),
            max,
            count_comments,
            count_as_one,
            lsp_enabled,
        }
    }

    pub(crate) fn message(&self, length: usize, max_length: usize) -> String {
        format!(
            "{} has too many lines. [{length}/{max_length}]",
            self.cop_label
        )
    }

    pub(crate) const fn max_length(&self) -> usize {
        self.max
    }

    pub(crate) const fn count_comments(&self) -> bool {
        self.count_comments
    }

    pub(crate) fn count_as_one(&self) -> Vec<String> {
        self.count_as_one.clone()
    }

    pub(crate) fn check_code_length(
        &self,
        node: NodeRef<'_>,
        processed_source: &ProcessedSource<'_>,
    ) -> Result<Option<CodeLengthOffense>, String> {
        if node.line_count() <= self.max_length() {
            return Ok(None);
        }
        let length = self.build_code_length_calculator(node, processed_source)?;
        if length <= self.max_length() {
            return Ok(None);
        }
        Ok(Some(CodeLengthOffense {
            location: self.location(node).unwrap_or_default(),
            message: self.message(length, self.max_length()),
            length,
        }))
    }

    pub(crate) fn irrelevant_line(&self, source_line: &str) -> bool {
        source_line.trim().is_empty()
            || (!self.count_comments() && source_line.trim_start().starts_with('#'))
    }

    pub(crate) fn build_code_length_calculator(
        &self,
        node: NodeRef<'_>,
        processed_source: &ProcessedSource<'_>,
    ) -> Result<usize, String> {
        code_length_for_node(
            node,
            processed_source,
            &CodeLengthOptions {
                count_comments: self.count_comments(),
                count_as_one: self.count_as_one(),
            },
        )
    }

    pub(crate) fn location(&self, node: NodeRef<'_>) -> Option<Range<usize>> {
        if node.kind() == "casgn" {
            return node.loc("name").map(|location| location.0.clone());
        }
        let source = node.source_range()?;
        if self.lsp_enabled {
            let end = node
                .loc("name")
                .or_else(|| node.loc("begin"))
                .map_or(source.end, |location| location.0.end);
            Some(source.start..end)
        } else {
            Some(source)
        }
    }
}

#[cfg(test)]
mod spec;
