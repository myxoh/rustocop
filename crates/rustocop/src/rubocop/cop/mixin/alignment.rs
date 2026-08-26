// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/alignment.rb
// Source SHA-256: 4bd1ece511da0159dab1927bdd26d5b9f12e18b25a3627e503f8745bb3213ee6

use std::ops::Range;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AlignmentItem {
    pub(crate) source_range: Range<usize>,
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) begins_its_line: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AlignmentOffense {
    pub(crate) range: Range<usize>,
    pub(crate) correction_item: Option<AlignmentItem>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Alignment {
    lines: Vec<String>,
    indentation_width: usize,
    comment_lines: Vec<usize>,
    current_offenses: Vec<Range<usize>>,
    pub(crate) column_delta: isize,
}

impl Alignment {
    pub(crate) fn column_delta(&self) -> isize {
        self.column_delta
    }

    pub(crate) fn new(
        lines: Vec<String>,
        indentation_width: Option<usize>,
        default_width: Option<usize>,
    ) -> Self {
        Self {
            lines,
            indentation_width: indentation_width.or(default_width).unwrap_or(2),
            comment_lines: Vec::new(),
            current_offenses: Vec::new(),
            column_delta: 0,
        }
    }
    pub(crate) fn configured_indentation_width(&self) -> usize {
        self.indentation_width
    }
    pub(crate) fn indentation(&self, node: &AlignmentItem) -> String {
        self.offset(node) + &" ".repeat(self.configured_indentation_width())
    }
    pub(crate) fn offset(&self, node: &AlignmentItem) -> String {
        " ".repeat(node.column)
    }
    pub(crate) fn check_alignment(
        &mut self,
        items: &[AlignmentItem],
        base_column: Option<usize>,
    ) -> Vec<AlignmentOffense> {
        let base = base_column
            .or_else(|| items.first().map(|item| self.display_column(item)))
            .unwrap_or(0);
        let bad = self.each_bad_alignment(items, base);
        bad.into_iter()
            .map(|item| {
                let nested = self
                    .current_offenses
                    .iter()
                    .any(|offense| self.within(&item.source_range, offense));
                self.register_offense(&item, (!nested).then_some(item.clone()))
            })
            .collect()
    }
    pub(crate) fn each_bad_alignment(
        &mut self,
        items: &[AlignmentItem],
        base_column: usize,
    ) -> Vec<AlignmentItem> {
        let mut previous_line = 0;
        let mut bad = Vec::new();
        for current in items {
            if current.line > previous_line && current.begins_its_line {
                self.column_delta = base_column as isize - self.display_column(current) as isize;
                if self.column_delta != 0 {
                    bad.push(current.clone());
                }
            }
            previous_line = current.line;
        }
        bad
    }
    pub(crate) fn display_column(&self, item: &AlignmentItem) -> usize {
        self.lines
            .get(item.line.saturating_sub(1))
            .map_or(item.column, |line| {
                UnicodeWidthStr::width(
                    &line[..line
                        .char_indices()
                        .nth(item.column)
                        .map_or(line.len(), |(index, _)| index)],
                )
            })
    }
    pub(crate) fn within(&self, inner: &Range<usize>, outer: &Range<usize>) -> bool {
        inner.start >= outer.start && inner.end <= outer.end
    }
    pub(crate) fn end_of_line_comment(&self, line: usize) -> bool {
        self.comment_lines.contains(&line)
    }
    pub(crate) fn register_offense(
        &self,
        offense_node: &AlignmentItem,
        message_node: Option<AlignmentItem>,
    ) -> AlignmentOffense {
        AlignmentOffense {
            range: offense_node.source_range.clone(),
            correction_item: message_node,
        }
    }
}
