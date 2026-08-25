// RuboCop 1.87.0
// Source: lib/rubocop/cop/correctors/multiline_literal_brace_corrector.rb
// Source SHA-256: f3de31199f2ff42aed2683b0c1c76e0ec9cafc019125867de2f894afaeff1a5c

use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::processed_source::ProcessedSource;
use crate::rubocop::ast::source::SourceRange;
use crate::rubocop::cop::corrector::Corrector;
use crate::rubocop::cop::mixin::range_help::{RangeHelp, Side, SurroundingSpace};

pub(crate) struct MultilineLiteralBraceCorrector<'corrector, 'buffer, 'source, 'ast, 'processed> {
    corrector: &'corrector mut Corrector<'buffer, 'source>,
    node: NodeRef<'ast>,
    processed_source: &'processed ProcessedSource<'processed>,
}

impl<'corrector, 'buffer, 'source, 'ast, 'processed>
    MultilineLiteralBraceCorrector<'corrector, 'buffer, 'source, 'ast, 'processed>
{
    pub(crate) fn corrector(&mut self) -> &mut Corrector<'buffer, 'source> {
        self.corrector
    }

    pub(crate) fn node(&self) -> NodeRef<'ast> {
        self.node
    }

    pub(crate) fn processed_source(&self) -> &ProcessedSource<'processed> {
        self.processed_source
    }

    pub(crate) fn correct(
        corrector: &'corrector mut Corrector<'buffer, 'source>,
        node: NodeRef<'ast>,
        processed_source: &'processed ProcessedSource<'processed>,
    ) {
        Self::initialize(corrector, node, processed_source).call();
    }

    pub(crate) fn initialize(
        corrector: &'corrector mut Corrector<'buffer, 'source>,
        node: NodeRef<'ast>,
        processed_source: &'processed ProcessedSource<'processed>,
    ) -> Self {
        Self {
            corrector,
            node,
            processed_source,
        }
    }

    pub(crate) fn call(&mut self) {
        if self.closing_brace_on_same_line() {
            self.correct_same_line_brace();
            return;
        }
        if self.new_line_needed_before_closing_brace() {
            return;
        }
        let Some(end_range) = self.last_element_range_with_trailing_comma() else {
            return;
        };
        self.correct_next_line_brace(end_range);
        self.correct_heredoc_argument_method_chain(end_range);
    }

    pub(crate) fn correct_same_line_brace(&mut self) {
        let Some(closing) = self.node.loc("end") else {
            return;
        };
        self.corrector.insert_before(
            SourceRange::new(
                self.corrector.source_buffer(),
                closing.0.start,
                closing.0.end,
            ),
            "\n",
        );
    }

    pub(crate) fn correct_next_line_brace(&mut self, end_range: SourceRange<'buffer, 'source>) {
        let Some(closing) = self.node.loc("end") else {
            return;
        };
        let closing = SourceRange::new(
            self.corrector.source_buffer(),
            closing.0.start,
            closing.0.end,
        );
        let removal = RangeHelp::new(self.corrector.source_buffer()).range_with_surrounding_space(
            closing,
            SurroundingSpace {
                side: Side::Left,
                ..SurroundingSpace::default()
            },
        );
        let commented = self.children().last().is_some_and(|last| {
            self.processed_source
                .comment_at_line(last.last_line())
                .is_some()
        });
        let content = self.content_if_comment_present();
        if commented {
            // RuboCop's TreeRewriter coalesces this with the removal performed
            // by `select_content_to_be_inserted_after_last_element`. Keep the
            // ranges adjacent for Rust's stricter clobber detection.
            self.corrector.remove(SourceRange::new(
                self.corrector.source_buffer(),
                removal.begin_pos(),
                closing.begin_pos(),
            ));
        } else {
            self.corrector.remove(removal);
        }
        self.corrector.insert_after(end_range, content);
    }

    pub(crate) fn correct_heredoc_argument_method_chain(
        &mut self,
        end_range: SourceRange<'buffer, 'source>,
    ) {
        let Some(parent) = self.node.parent() else {
            return;
        };
        if !self.use_heredoc_argument_method_chain(parent) {
            return;
        }
        let (Some(dot), Some(parent_range)) = (parent.loc("dot"), parent.source_range()) else {
            return;
        };
        let chained_method = SourceRange::new(
            self.corrector.source_buffer(),
            dot.0.start,
            parent_range.end,
        );
        let source = chained_method.source().to_owned();
        self.corrector.remove(chained_method);
        self.corrector.insert_after(end_range, source);
    }

    pub(crate) fn content_if_comment_present(&mut self) -> String {
        let Some(last) = self.children().last().copied() else {
            return String::new();
        };
        if self
            .processed_source
            .comment_at_line(last.last_line())
            .is_some()
        {
            self.select_content_to_be_inserted_after_last_element()
        } else {
            self.node
                .loc("end")
                .map_or_else(String::new, |location| location.1.clone())
        }
    }

    pub(crate) fn use_heredoc_argument_method_chain(&self, parent: NodeRef<'_>) -> bool {
        parent.call_type()
            && self.node.first_argument().is_some_and(|first_argument| {
                first_argument.kind() == "str" && first_argument.heredoc()
            })
    }

    pub(crate) fn select_content_to_be_inserted_after_last_element(&mut self) -> String {
        let Some(node_range) = self.node.source_range() else {
            return String::new();
        };
        let Some(closing) = self.node.loc("end") else {
            return String::new();
        };
        let buffer = self.corrector.source_buffer();
        let whole = RangeHelp::new(buffer).range_by_whole_lines(
            SourceRange::new(buffer, node_range.start, node_range.end),
            false,
        );
        let range = SourceRange::new(buffer, closing.0.start, whole.end_pos());
        let source = range.source().to_owned();
        self.remove_trailing_content_of_comment(range);
        source
    }

    pub(crate) fn remove_trailing_content_of_comment(
        &mut self,
        range: SourceRange<'buffer, 'source>,
    ) {
        self.corrector.remove(range);
    }

    pub(crate) fn last_element_range_with_trailing_comma(
        &self,
    ) -> Option<SourceRange<'buffer, 'source>> {
        let last = self.children().last().copied()?;
        let range = last.source_range()?;
        let range = SourceRange::new(self.corrector.source_buffer(), range.start, range.end);
        Some(
            self.last_element_trailing_comma_range()
                .map_or(range, |comma| range.join(comma)),
        )
    }

    pub(crate) fn last_element_trailing_comma_range(
        &self,
    ) -> Option<SourceRange<'buffer, 'source>> {
        let last = self.children().last().copied()?;
        let range = last.source_range()?;
        let buffer = self.corrector.source_buffer();
        let range = RangeHelp::new(buffer).range_with_surrounding_space(
            SourceRange::new(buffer, range.start, range.end),
            SurroundingSpace {
                side: Side::Right,
                ..SurroundingSpace::default()
            },
        );
        let comma = SourceRange::new(
            buffer,
            range.end_pos(),
            (range.end_pos() + 1).min(buffer.len()),
        );
        (comma.source() == ",").then_some(comma)
    }

    pub(crate) fn closing_brace_on_same_line(&self) -> bool {
        let Some(last) = self.children().last().copied() else {
            return false;
        };
        self.node.loc("end").is_some_and(|closing| {
            SourceRange::new(
                self.corrector.source_buffer(),
                closing.0.start,
                closing.0.end,
            )
            .line()
                == last.last_line()
        })
    }

    pub(crate) fn new_line_needed_before_closing_brace(&self) -> bool {
        let Some(last) = self.last_element_range_with_trailing_comma() else {
            return false;
        };
        self.processed_source
            .comment_at_line(last.last_line())
            .is_some()
            && (self.node.chained() || self.node.argument())
    }

    pub(crate) fn children(&self) -> Vec<NodeRef<'ast>> {
        self.node.child_nodes()
    }
}

#[cfg(test)]
mod spec;
