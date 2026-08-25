// RuboCop 1.87.0
// Source: lib/rubocop/cop/correctors/parentheses_corrector.rb
// Source SHA-256: 513e01b2b527690d925e0ec44fd513f14bf847ef1622051ca15872a3ec324b27

use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::source::{SourceBuffer, SourceRange};
use crate::rubocop::cop::corrector::Corrector;
use crate::rubocop::cop::mixin::range_help::{RangeHelp, Side, SurroundingSpace};

pub(crate) struct ParenthesesCorrector;

impl ParenthesesCorrector {
    pub(crate) fn correct<'buffer, 'source>(
        corrector: &mut Corrector<'buffer, 'source>,
        node: NodeRef<'_>,
    ) {
        let buffer = corrector.source_buffer();
        let Some(opening) = node
            .loc("begin")
            .map(|location| SourceRange::new(buffer, location.0.start, location.0.end))
        else {
            return;
        };
        let helper = RangeHelp::new(buffer);
        corrector.remove(helper.range_with_surrounding_space(
            opening,
            SurroundingSpace {
                side: Side::Right,
                whitespace: true,
                newlines: false,
                continuations: false,
            },
        ));
        Self::remove_close_paren(corrector, node, buffer);
        Self::handle_orphaned_comma(corrector, node);
        if Self::ternary_condition(node) && Self::next_char_is_question_mark(node) {
            if let Some(closing) = node.loc("end") {
                corrector.insert_after(
                    SourceRange::new(buffer, closing.0.start, closing.0.end),
                    " ",
                );
            }
        }
    }

    pub(crate) fn remove_close_paren<'buffer, 'source>(
        corrector: &mut Corrector<'buffer, 'source>,
        node: NodeRef<'_>,
        buffer: &'buffer SourceBuffer<'source>,
    ) {
        let Some(closing) = node
            .loc("end")
            .map(|location| SourceRange::new(buffer, location.0.start, location.0.end))
        else {
            return;
        };
        let newlines = !Self::comment_above_close_paren_swallows_chain(node, buffer);
        let range = RangeHelp::new(buffer).range_with_surrounding_space(
            closing,
            SurroundingSpace {
                side: Side::Left,
                newlines,
                whitespace: false,
                continuations: false,
            },
        );
        corrector.remove(range);
    }

    pub(crate) fn comment_above_close_paren_swallows_chain(
        node: NodeRef<'_>,
        buffer: &SourceBuffer<'_>,
    ) -> bool {
        let Some(last) = node.child_nodes().last().copied() else {
            return false;
        };
        let Some(body_end) = last.source_range().map(|range| range.end) else {
            return false;
        };
        let Some(close) = node.loc("end").map(|location| location.0.start) else {
            return false;
        };
        body_end < close
            && buffer
                .slice(body_end..close)
                .lines()
                .any(|line| line.contains('#'))
            && Self::chained_after_close_paren(node, buffer)
    }

    pub(crate) fn chained_after_close_paren(node: NodeRef<'_>, buffer: &SourceBuffer<'_>) -> bool {
        let Some(closing) = node.loc("end") else {
            return false;
        };
        let range = SourceRange::new(buffer, closing.0.start, closing.0.end);
        let after = buffer
            .slice(closing.0.end..buffer.line_range(range.line()).end)
            .trim_start();
        !after.is_empty() && !after.starts_with('#')
    }

    pub(crate) fn ternary_condition(node: NodeRef<'_>) -> bool {
        node.parent()
            .is_some_and(|parent| parent.kind() == "if" && parent.ternary())
    }

    pub(crate) fn next_char_is_question_mark(node: NodeRef<'_>) -> bool {
        node.parent()
            .and_then(|parent| parent.loc("question").map(|location| (parent, location)))
            .is_some_and(|(_, question)| {
                node.loc_last_column("end") == Some(question.0.start)
                    || node
                        .loc("end")
                        .is_some_and(|end| end.0.end == question.0.start)
            })
    }

    pub(crate) fn only_closing_paren_before_comma(
        node: NodeRef<'_>,
        buffer: &SourceBuffer<'_>,
    ) -> bool {
        let Some(closing) = node.loc("end") else {
            return false;
        };
        let range = SourceRange::new(buffer, closing.0.start, closing.0.end);
        buffer
            .source_line(range.line())
            .trim_start()
            .strip_prefix(')')
            .is_some_and(|after| after.trim_start().starts_with(','))
    }

    pub(crate) fn handle_orphaned_comma<'buffer, 'source>(
        corrector: &mut Corrector<'buffer, 'source>,
        node: NodeRef<'_>,
    ) {
        if !Self::only_closing_paren_before_comma(node, corrector.source_buffer()) {
            return;
        }
        let range = Self::parens_range(node, corrector.source_buffer());
        let range = Self::extend_range_for_heredoc(node, range, corrector.source_buffer());
        corrector.remove(range);
        Self::add_heredoc_comma(corrector, node);
    }

    pub(crate) fn parens_range<'buffer, 'source>(
        node: NodeRef<'_>,
        buffer: &'buffer SourceBuffer<'source>,
    ) -> SourceRange<'buffer, 'source> {
        let closing = node
            .loc("end")
            .expect("parenthesized node has closing range");
        RangeHelp::new(buffer).range_with_surrounding_space(
            SourceRange::new(buffer, closing.0.start, closing.0.end),
            SurroundingSpace {
                side: Side::Left,
                newlines: true,
                whitespace: true,
                continuations: true,
            },
        )
    }

    pub(crate) fn extend_range_for_heredoc<'buffer, 'source>(
        node: NodeRef<'_>,
        range: SourceRange<'buffer, 'source>,
        buffer: &'buffer SourceBuffer<'source>,
    ) -> SourceRange<'buffer, 'source> {
        if !Self::heredoc(node) {
            return range;
        }
        let line = RangeHelp::new(buffer)
            .range_by_whole_lines(range, false)
            .source();
        let offset = line
            .find(')')
            .and_then(|closing| line[closing + 1..].find(',').map(|comma| comma + 1))
            .unwrap_or(0);
        range.adjust(0, offset as isize)
    }

    pub(crate) fn add_heredoc_comma<'buffer, 'source>(
        corrector: &mut Corrector<'buffer, 'source>,
        node: NodeRef<'_>,
    ) {
        let Some(last) = node
            .child_nodes()
            .last()
            .copied()
            .filter(|child| child.heredoc())
        else {
            return;
        };
        let Some(range) = last.source_range() else {
            return;
        };
        corrector.insert_after(
            SourceRange::new(corrector.source_buffer(), range.start, range.end),
            ",",
        );
    }

    pub(crate) fn heredoc(node: NodeRef<'_>) -> bool {
        node.child_nodes()
            .last()
            .is_some_and(|child| child.heredoc())
    }
}

#[cfg(test)]
mod spec;
