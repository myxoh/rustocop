// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/comments_help.rb
// Source SHA-256: 65172714b7ffcf136480b48d5ec620cefe869c3e39498cd04100fb0e0cb6e2f6

use std::collections::BTreeMap;
use std::ops::Range;

use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::processed_source::{ProcessedSource, SourceComment};
use crate::rubocop::ast::source::{SourceBuffer, SourceRange};

pub(crate) struct CommentsHelp<'processed, 'source> {
    processed_source: &'processed ProcessedSource<'source>,
    buffer: SourceBuffer<'source>,
    disabled_line_ranges: BTreeMap<String, Vec<Range<usize>>>,
}

impl<'processed, 'source> CommentsHelp<'processed, 'source> {
    pub(crate) fn new(processed_source: &'processed ProcessedSource<'source>) -> Self {
        Self {
            processed_source,
            buffer: processed_source.buffer(),
            disabled_line_ranges: BTreeMap::new(),
        }
    }

    pub(crate) fn with_disabled_line_ranges(
        processed_source: &'processed ProcessedSource<'source>,
        disabled_line_ranges: BTreeMap<String, Vec<Range<usize>>>,
    ) -> Self {
        Self {
            processed_source,
            buffer: processed_source.buffer(),
            disabled_line_ranges,
        }
    }

    pub(crate) fn source_range_with_comment(
        &self,
        node: NodeRef<'_>,
    ) -> Option<SourceRange<'_, 'source>> {
        Some(SourceRange::new(
            self.buffer(),
            self.begin_pos_with_comment(node)?,
            self.end_position_for(node)?,
        ))
    }

    pub(crate) fn contains_comments(&self, node: NodeRef<'_>) -> bool {
        !self.comments_in_range(node).is_empty()
    }

    pub(crate) fn comments_in_range(&self, node: NodeRef<'_>) -> Vec<&SourceComment> {
        if node.source_range().is_none() {
            return Vec::new();
        }
        self.processed_source
            .each_comment_in_lines(node.first_line()..self.find_end_line(node))
    }

    pub(crate) fn comments_contain_disables(&self, node: NodeRef<'_>, cop_name: &str) -> bool {
        let Some(disabled_ranges) = self.disabled_line_ranges.get(cop_name) else {
            return false;
        };
        let node_range = node.first_line()..self.find_end_line(node);
        disabled_ranges
            .iter()
            .any(|disabled| covers(disabled, &node_range) || covers(&node_range, disabled))
    }

    pub(crate) fn end_position_for(&self, node: NodeRef<'_>) -> Option<usize> {
        let end = node.source_range()?.end;
        let line = SourceRange::new(self.buffer(), end, end).line();
        Some(self.buffer().line_range(line).end)
    }

    pub(crate) fn begin_pos_with_comment(&self, node: NodeRef<'_>) -> Option<usize> {
        let first_comment = self
            .processed_source
            .ast_with_comments()
            .into_iter()
            .find(|(candidate, _)| *candidate == node)
            .and_then(|(_, comments)| comments.first().copied());
        if first_comment.is_some_and(|comment| comment.line < node.first_line()) {
            self.start_line_position(first_comment?.line)
        } else {
            self.start_line_position(node.first_line())
        }
    }

    pub(crate) fn start_line_position(&self, line: usize) -> Option<usize> {
        Some(
            self.buffer()
                .line_start(line)
                .saturating_sub(usize::from(line > 1)),
        )
    }

    pub(crate) fn buffer(&self) -> &SourceBuffer<'source> {
        &self.buffer
    }

    pub(crate) fn find_end_line(&self, node: NodeRef<'_>) -> usize {
        let special = if node.kind() == "if" {
            if node.has_else() {
                node.loc("else")
                    .map(|location| self.line_for(location.0.start))
            } else if node.ternary() {
                node.else_branch().map(NodeRef::first_line)
            } else if node.elsif() {
                node.ancestors()
                    .into_iter()
                    .find(|ancestor| ancestor.kind() == "if" && !ancestor.elsif())
                    .and_then(|ancestor| ancestor.loc("end"))
                    .map(|location| self.line_for(location.0.start))
            } else if node.keyword_name() == Some("if")
                && node
                    .parent()
                    .is_some_and(|parent| parent.loc("begin").is_some())
            {
                node.parent()
                    .and_then(|parent| parent.loc("end"))
                    .map(|location| self.line_for(location.0.start))
            } else {
                None
            }
        } else if matches!(node.kind(), "block" | "numblock" | "itblock") {
            node.loc("end")
                .map(|location| self.line_for(location.0.start))
        } else if let Some(sibling) = node
            .right_sibling()
            .filter(|sibling| sibling.source_range().is_some())
        {
            Some(sibling.first_line())
        } else {
            node.parent().map(|parent| {
                parent.loc("end").map_or_else(
                    || parent.first_line(),
                    |location| self.line_for(location.0.start),
                )
            })
        };
        special.unwrap_or(node.last_line())
    }

    fn line_for(&self, position: usize) -> usize {
        SourceRange::new(self.buffer(), position, position).line()
    }
}

fn covers(outer: &Range<usize>, inner: &Range<usize>) -> bool {
    outer.start <= inner.start && outer.end >= inner.end
}

#[cfg(test)]
mod spec;
