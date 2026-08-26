// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/range_help.rb
// Source SHA-256: 5662cc556c28cbf79b0613c9f1bb4764aa1b72b7d41b7bdd816b902021eb5660

use std::ops::Range;

use crate::rubocop::ast::source::{SourceBuffer, SourceRange};

const BYTE_ORDER_MARK: char = '\u{feff}';

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Side {
    Both,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SurroundingSpace {
    pub(crate) side: Side,
    pub(crate) newlines: bool,
    pub(crate) whitespace: bool,
    pub(crate) continuations: bool,
}

impl Default for SurroundingSpace {
    fn default() -> Self {
        Self {
            side: Side::Both,
            newlines: true,
            whitespace: false,
            continuations: false,
        }
    }
}

pub(crate) struct RangeHelp<'buffer, 'source> {
    processed_source: &'buffer SourceBuffer<'source>,
}

impl<'buffer, 'source> RangeHelp<'buffer, 'source> {
    pub(crate) fn new(processed_source: &'buffer SourceBuffer<'source>) -> Self {
        Self { processed_source }
    }

    pub(crate) fn source_range(
        &self,
        source_buffer: &'buffer SourceBuffer<'source>,
        line_number: usize,
        column: usize,
        length: usize,
    ) -> SourceRange<'buffer, 'source> {
        let line_begin_pos = if line_number == 0 {
            0
        } else {
            source_buffer.line_start(line_number)
        };
        let begin_pos = line_begin_pos + column;
        SourceRange::new(source_buffer, begin_pos, begin_pos + length)
    }

    pub(crate) fn source_range_columns(
        &self,
        source_buffer: &'buffer SourceBuffer<'source>,
        line_number: usize,
        columns: Range<usize>,
    ) -> SourceRange<'buffer, 'source> {
        self.source_range(
            source_buffer,
            line_number,
            columns.start,
            columns.end - columns.start,
        )
    }

    pub(crate) fn contents_range(
        &self,
        begin_delimiter: SourceRange<'buffer, 'source>,
        end_delimiter: SourceRange<'buffer, 'source>,
    ) -> SourceRange<'buffer, 'source> {
        SourceRange::new(
            begin_delimiter.buffer(),
            begin_delimiter.end_pos(),
            end_delimiter.begin_pos(),
        )
    }

    pub(crate) fn arguments_range(
        &self,
        first_argument: SourceRange<'buffer, 'source>,
        last_argument: SourceRange<'buffer, 'source>,
    ) -> SourceRange<'buffer, 'source> {
        first_argument.join(last_argument)
    }

    pub(crate) fn range_between(
        &self,
        start_pos: usize,
        end_pos: usize,
    ) -> SourceRange<'buffer, 'source> {
        SourceRange::new(self.processed_source, start_pos, end_pos)
    }

    pub(crate) fn range_with_surrounding_comma(
        &self,
        range: SourceRange<'buffer, 'source>,
        side: Side,
    ) -> SourceRange<'buffer, 'source> {
        let buffer = range.buffer();
        let (go_left, go_right) = directions(side);
        let begin_pos = move_character(buffer, range.begin_pos(), -1, go_left, ',');
        let end_pos = move_character(buffer, range.end_pos(), 1, go_right, ',');
        SourceRange::new(buffer, begin_pos, end_pos)
    }

    pub(crate) fn range_with_surrounding_space(
        &self,
        range: SourceRange<'buffer, 'source>,
        options: SurroundingSpace,
    ) -> SourceRange<'buffer, 'source> {
        let buffer = range.buffer();
        let (go_left, go_right) = directions(options.side);
        let begin_pos = if go_left {
            final_pos(buffer, range.begin_pos(), -1, options)
        } else {
            range.begin_pos()
        };
        let end_pos = if go_right {
            final_pos(buffer, range.end_pos(), 1, options)
        } else {
            range.end_pos()
        };
        SourceRange::new(buffer, begin_pos, end_pos)
    }

    pub(crate) fn range_by_whole_lines(
        &self,
        range: SourceRange<'buffer, 'source>,
        include_final_newline: bool,
    ) -> SourceRange<'buffer, 'source> {
        let buffer = range.buffer();
        let last_line = buffer.source_line(range.last_line());
        let end_offset = last_line
            .chars()
            .count()
            .saturating_sub(range.last_column())
            + usize::from(include_final_newline);
        let begin = range.begin_pos().saturating_sub(range.column());
        let end = (range.end_pos() + end_offset).min(buffer.len());
        SourceRange::new(buffer, begin, end)
    }

    pub(crate) fn range_with_comments(
        &self,
        node: SourceRange<'buffer, 'source>,
        associated_comments: &[SourceRange<'buffer, 'source>],
    ) -> SourceRange<'buffer, 'source> {
        associated_comments
            .iter()
            .copied()
            .fold(node, |result, comment| self.add_range(result, comment))
    }

    pub(crate) fn range_with_comments_and_lines(
        &self,
        node: SourceRange<'buffer, 'source>,
        associated_comments: &[SourceRange<'buffer, 'source>],
    ) -> SourceRange<'buffer, 'source> {
        self.range_by_whole_lines(self.range_with_comments(node, associated_comments), true)
    }

    pub(crate) fn column_offset_between(
        &self,
        base_range: SourceRange<'buffer, 'source>,
        range: SourceRange<'buffer, 'source>,
    ) -> isize {
        self.effective_column(base_range) as isize - self.effective_column(range) as isize
    }

    pub(crate) fn effective_column(&self, range: SourceRange<'buffer, 'source>) -> usize {
        if range.line() == 1 && self.processed_source.source().starts_with(BYTE_ORDER_MARK) {
            range.column().saturating_sub(1)
        } else {
            range.column()
        }
    }

    pub(crate) fn add_range(
        &self,
        range1: SourceRange<'buffer, 'source>,
        range2: SourceRange<'buffer, 'source>,
    ) -> SourceRange<'buffer, 'source> {
        range1.join(range2)
    }
}

fn directions(side: Side) -> (bool, bool) {
    match side {
        Side::Both => (true, true),
        Side::Left => (true, false),
        Side::Right => (false, true),
    }
}

fn final_pos(
    buffer: &SourceBuffer<'_>,
    position: usize,
    increment: isize,
    options: SurroundingSpace,
) -> usize {
    let position = move_while(buffer, position, increment, true, |character| {
        matches!(character, ' ' | '\t')
    });
    let position = move_string(buffer, position, increment, options.continuations, "\\\n");
    let position = move_character(buffer, position, increment, options.newlines, '\n');
    move_while(
        buffer,
        position,
        increment,
        options.whitespace,
        |character| {
            matches!(
                character,
                ' ' | '\t' | '\r' | '\n' | '\u{000b}' | '\u{000c}'
            )
        },
    )
}

fn move_character(
    buffer: &SourceBuffer<'_>,
    position: usize,
    increment: isize,
    condition: bool,
    needle: char,
) -> usize {
    move_while(buffer, position, increment, condition, |character| {
        character == needle
    })
}

fn move_while(
    buffer: &SourceBuffer<'_>,
    mut position: usize,
    increment: isize,
    condition: bool,
    predicate: impl Fn(char) -> bool,
) -> usize {
    if !condition {
        return position;
    }
    loop {
        let candidate = if increment < 0 {
            position.checked_sub(1)
        } else {
            (position < buffer.len()).then_some(position)
        };
        let Some(candidate) = candidate else {
            break;
        };
        if !buffer.character(candidate).is_some_and(&predicate) {
            break;
        }
        position = position.saturating_add_signed(increment);
    }
    position
}

fn move_string(
    buffer: &SourceBuffer<'_>,
    mut position: usize,
    increment: isize,
    condition: bool,
    needle: &str,
) -> usize {
    let needle_length = needle.chars().count();
    if !condition || needle_length == 0 {
        return position;
    }
    loop {
        let start = if increment < 0 {
            position.checked_sub(needle_length)
        } else {
            Some(position)
        };
        let Some(start) = start else {
            break;
        };
        if buffer.slice(start..start + needle_length) != needle {
            break;
        }
        position = position.saturating_add_signed(increment * needle_length as isize);
    }
    position
}
