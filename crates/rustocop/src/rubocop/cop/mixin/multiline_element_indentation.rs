// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/multiline_element_indentation.rb
// Source SHA-256: 295abb00d6ca7c0595630ed224e06173b6db95687169156d26d2c4bd20523aa4

use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::source::{SourceBuffer, SourceRange};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IndentBaseType {
    LeftBraceOrBracket,
    ParentHashKey,
    FirstColumnAfterLeftParenthesis,
    StartOfLine,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ElementIndentationResult {
    pub(crate) expected_column: usize,
    pub(crate) column_delta: isize,
    pub(crate) styles: Vec<String>,
    pub(crate) correct: bool,
    pub(crate) ambiguous: bool,
    pub(crate) message: Option<String>,
}

pub(crate) struct MultilineElementIndentation<'buffer, 'source> {
    buffer: &'buffer SourceBuffer<'source>,
    style: String,
    brace_alignment_style: String,
    configured_indentation_width: usize,
}

impl<'buffer, 'source> MultilineElementIndentation<'buffer, 'source> {
    pub(crate) fn new(
        buffer: &'buffer SourceBuffer<'source>,
        style: impl Into<String>,
        brace_alignment_style: impl Into<String>,
        configured_indentation_width: usize,
    ) -> Self {
        Self {
            buffer,
            style: style.into(),
            brace_alignment_style: brace_alignment_style.into(),
            configured_indentation_width,
        }
    }

    pub(crate) fn each_argument_node<'ast>(
        &self,
        node: NodeRef<'ast>,
        node_type: &str,
    ) -> Vec<(NodeRef<'ast>, SourceRange<'buffer, 'source>)> {
        let Some(parenthesis) = node.loc("begin") else {
            return Vec::new();
        };
        let parenthesis = SourceRange::new(self.buffer, parenthesis.0.start, parenthesis.0.end);
        node.arguments()
            .into_iter()
            .flat_map(|argument| argument.each_node(&[node_type]))
            .filter_map(|type_node| {
                let brace = type_node.loc("begin")?;
                let brace = SourceRange::new(self.buffer, brace.0.start, brace.0.end);
                (brace.line() == parenthesis.line()).then_some((type_node, parenthesis))
            })
            .collect()
    }

    pub(crate) fn check_first(
        &self,
        first: NodeRef<'_>,
        left_brace: SourceRange<'buffer, 'source>,
        left_parenthesis: Option<SourceRange<'buffer, 'source>>,
        offset: usize,
    ) -> ElementIndentationResult {
        let actual_column = first.column();
        let (base, base_type) = self.indent_base(left_brace, first, left_parenthesis);
        let expected_column = base + self.configured_indentation_width + offset;
        let column_delta = expected_column as isize - actual_column as isize;
        let styles = self.detected_styles(actual_column, offset, left_parenthesis, left_brace);
        if column_delta == 0 {
            self.check_expected_style(expected_column, styles)
        } else {
            self.incorrect_style_detected(expected_column, column_delta, styles, base_type)
        }
    }

    pub(crate) fn check_expected_style(
        &self,
        expected_column: usize,
        styles: Vec<String>,
    ) -> ElementIndentationResult {
        ElementIndentationResult {
            expected_column,
            column_delta: 0,
            correct: styles.len() <= 1,
            ambiguous: styles.len() > 1,
            styles,
            message: None,
        }
    }

    pub(crate) fn indent_base(
        &self,
        left_brace: SourceRange<'buffer, 'source>,
        first: NodeRef<'_>,
        left_parenthesis: Option<SourceRange<'buffer, 'source>>,
    ) -> (usize, IndentBaseType) {
        if self.style == self.brace_alignment_style {
            return (left_brace.column(), IndentBaseType::LeftBraceOrBracket);
        }
        if let Some(pair) = self.hash_pair_where_value_beginning_with(left_brace, first) {
            if self.key_and_value_begin_on_same_line(pair)
                && self.right_sibling_begins_on_subsequent_line(pair)
            {
                return (pair.column(), IndentBaseType::ParentHashKey);
            }
        }
        if self.style == "special_inside_parentheses" {
            if let Some(parenthesis) = left_parenthesis {
                return (
                    parenthesis.column() + 1,
                    IndentBaseType::FirstColumnAfterLeftParenthesis,
                );
            }
        }
        let first_non_space = self
            .buffer
            .source_line(left_brace.line())
            .chars()
            .position(|character| !character.is_whitespace())
            .unwrap_or(0);
        (first_non_space, IndentBaseType::StartOfLine)
    }

    pub(crate) fn hash_pair_where_value_beginning_with<'ast>(
        &self,
        left_brace: SourceRange<'buffer, 'source>,
        first: NodeRef<'ast>,
    ) -> Option<NodeRef<'ast>> {
        let parent = first.parent()?;
        let begin = parent.loc("begin")?;
        if begin.0.start != left_brace.begin_pos() || begin.0.end != left_brace.end_pos() {
            return None;
        }
        parent.parent().filter(|pair| pair.kind() == "pair")
    }

    pub(crate) fn key_and_value_begin_on_same_line(&self, pair: NodeRef<'_>) -> bool {
        pair.key()
            .zip(pair.value_node())
            .is_some_and(|(key, value)| key.first_line() == value.first_line())
    }

    pub(crate) fn right_sibling_begins_on_subsequent_line(&self, pair: NodeRef<'_>) -> bool {
        pair.right_sibling()
            .is_some_and(|sibling| pair.last_line() < sibling.first_line())
    }

    pub(crate) fn detected_styles(
        &self,
        actual_column: usize,
        offset: usize,
        left_parenthesis: Option<SourceRange<'buffer, 'source>>,
        left_brace: SourceRange<'buffer, 'source>,
    ) -> Vec<String> {
        let base = actual_column.saturating_sub(self.configured_indentation_width + offset);
        self.detected_styles_for_column(base, left_parenthesis, left_brace)
    }

    pub(crate) fn detected_styles_for_column(
        &self,
        column: usize,
        left_parenthesis: Option<SourceRange<'buffer, 'source>>,
        left_brace: SourceRange<'buffer, 'source>,
    ) -> Vec<String> {
        let mut styles = Vec::new();
        let line_start = self
            .buffer
            .source_line(left_brace.line())
            .chars()
            .position(|character| !character.is_whitespace())
            .unwrap_or(0);
        if column == line_start {
            styles.push("consistent".into());
            if left_parenthesis.is_none() {
                styles.push("special_inside_parentheses".into());
            }
        }
        if left_parenthesis.is_some_and(|parenthesis| column == parenthesis.column() + 1) {
            styles.push("special_inside_parentheses".into());
        }
        if column == left_brace.column() {
            styles.push(self.brace_alignment_style.clone());
        }
        styles
    }

    pub(crate) fn incorrect_style_detected(
        &self,
        expected_column: usize,
        column_delta: isize,
        styles: Vec<String>,
        base_column_type: IndentBaseType,
    ) -> ElementIndentationResult {
        ElementIndentationResult {
            expected_column,
            column_delta,
            ambiguous: false,
            correct: false,
            styles,
            message: Some(format!("Use indentation relative to {base_column_type:?}.")),
        }
    }
}

#[cfg(test)]
mod spec;
