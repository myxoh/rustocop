// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/end_keyword_alignment.rb
// Source SHA-256: 6efdbf9c40254c69d2b1324a87e85024651f4d0564ee03b6287a1c3a1b825491

use std::collections::BTreeMap;

use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::source::{SourceBuffer, SourceRange};
use crate::rubocop::cop::mixin::range_help::RangeHelp;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EndKeywordOffense {
    pub(crate) message: String,
    pub(crate) detected_styles: Vec<String>,
}

pub(crate) struct EndKeywordAlignment<'buffer, 'source> {
    style: String,
    buffer: &'buffer SourceBuffer<'source>,
}

impl<'buffer, 'source> EndKeywordAlignment<'buffer, 'source> {
    pub(crate) fn new(style: impl Into<String>, buffer: &'buffer SourceBuffer<'source>) -> Self {
        Self {
            style: style.into(),
            buffer,
        }
    }

    pub(crate) fn check_end_kw_in_node(&self, node: NodeRef<'_>) -> Option<EndKeywordOffense> {
        let keyword = node.loc("keyword")?;
        self.check_end_kw_alignment(
            node,
            &BTreeMap::from([(
                self.style.clone(),
                SourceRange::new(self.buffer, keyword.0.start, keyword.0.end),
            )]),
        )
    }

    pub(crate) fn check_end_kw_alignment(
        &self,
        node: NodeRef<'_>,
        align_ranges: &BTreeMap<String, SourceRange<'buffer, 'source>>,
    ) -> Option<EndKeywordOffense> {
        let end = node.loc("end")?;
        let end = SourceRange::new(self.buffer, end.0.start, end.0.end);
        let matching = self.matching_ranges(end, align_ranges);
        if matching.contains_key(&self.style) {
            None
        } else {
            let align_with = align_ranges.get(&self.style)?;
            Some(self.add_offense_for_misalignment(
                end,
                *align_with,
                matching.keys().cloned().collect(),
            ))
        }
    }

    pub(crate) fn matching_ranges(
        &self,
        end_location: SourceRange<'buffer, 'source>,
        align_ranges: &BTreeMap<String, SourceRange<'buffer, 'source>>,
    ) -> BTreeMap<String, SourceRange<'buffer, 'source>> {
        align_ranges
            .iter()
            .filter(|(_, range)| {
                range.line() == end_location.line()
                    || RangeHelp::new(self.buffer).column_offset_between(**range, end_location) == 0
            })
            .map(|(style, range)| (style.clone(), *range))
            .collect()
    }

    pub(crate) fn start_line_range(
        &self,
        node: NodeRef<'_>,
    ) -> Option<SourceRange<'buffer, 'source>> {
        let expression = node.source_range()?;
        let line = SourceRange::new(self.buffer, expression.start, expression.end).line();
        let source = self.buffer.source_line(line);
        let line_range = self.buffer.line_range(line);
        let first_non_space = source
            .chars()
            .position(|character| !character.is_whitespace())?;
        let trailing_start = source.trim_end_matches(char::is_whitespace).chars().count();
        Some(SourceRange::new(
            self.buffer,
            line_range.start + first_non_space,
            line_range.start + trailing_start,
        ))
    }

    pub(crate) fn add_offense_for_misalignment(
        &self,
        end_location: SourceRange<'buffer, 'source>,
        align_with: SourceRange<'buffer, 'source>,
        detected_styles: Vec<String>,
    ) -> EndKeywordOffense {
        EndKeywordOffense {
            message: format!(
                "`end` at {}, {} is not aligned with `{}` at {}, {}.",
                end_location.line(),
                end_location.column(),
                align_with.source(),
                align_with.line(),
                align_with.column()
            ),
            detected_styles,
        }
    }

    pub(crate) const fn style_parameter_name(&self) -> &'static str {
        "EnforcedStyleAlignWith"
    }

    pub(crate) fn variable_alignment(
        &self,
        whole_expression: SourceRange<'buffer, 'source>,
        rhs: NodeRef<'_>,
        end_alignment_style: &str,
    ) -> bool {
        end_alignment_style != "keyword" && !self.line_break_before_keyword(whole_expression, rhs)
    }

    pub(crate) fn line_break_before_keyword(
        &self,
        whole_expression: SourceRange<'buffer, 'source>,
        rhs: NodeRef<'_>,
    ) -> bool {
        rhs.first_line() > whole_expression.line()
    }
}

#[cfg(test)]
mod spec;
