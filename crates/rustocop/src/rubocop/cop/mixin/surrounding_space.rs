// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/surrounding_space.rb
// Source SHA-256: 54db61d6bedac50f539d13e8580ef8861079e3de52a122d4d08976e956385610

use crate::rubocop::ast::source::{SourceBuffer, SourceRange};
use crate::rubocop::ast::token::Token;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SpaceSide {
    Left,
    Right,
    None,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SpaceOffense<'buffer, 'source> {
    pub(crate) range: SourceRange<'buffer, 'source>,
    pub(crate) message: String,
}

pub(crate) struct SurroundingSpace<'buffer, 'source> {
    buffer: &'buffer SourceBuffer<'source>,
    token_table_initialized: bool,
    autocorrect_with_disable_uncorrectable: bool,
}

impl<'buffer, 'source> SurroundingSpace<'buffer, 'source> {
    pub(crate) fn new(
        buffer: &'buffer SourceBuffer<'source>,
        autocorrect_with_disable_uncorrectable: bool,
    ) -> Self {
        Self {
            buffer,
            token_table_initialized: false,
            autocorrect_with_disable_uncorrectable,
        }
    }

    pub(crate) fn side_space_range(
        &self,
        range: SourceRange<'buffer, 'source>,
        side: SpaceSide,
        include_newlines: bool,
    ) -> SourceRange<'buffer, 'source> {
        let mut begin = range.begin_pos();
        let mut end = range.end_pos();
        if side == SpaceSide::Left {
            end = begin;
            begin = self.reposition(begin, -1, include_newlines);
        }
        if side == SpaceSide::Right {
            begin = end;
            end = self.reposition(end, 1, include_newlines);
        }
        SourceRange::new(self.buffer, begin, end)
    }

    pub(crate) fn on_new_investigation(&mut self) {
        self.token_table_initialized = false;
    }

    pub(crate) fn no_space_offenses(
        &self,
        left_token: Option<&Token<'buffer, 'source>>,
        right_token: Option<&Token<'buffer, 'source>>,
        message: &str,
        start_ok: bool,
        end_ok: bool,
    ) -> Vec<SpaceOffense<'buffer, 'source>> {
        let mut offenses = Vec::new();
        if self.extra_space(left_token, SpaceSide::Left) && !start_ok {
            if let Some(token) = left_token {
                offenses.push(self.space_offense(token, SpaceSide::Right, message, "Do not use"));
            }
        }
        if self.extra_space(right_token, SpaceSide::Right)
            && !end_ok
            && (start_ok || !self.autocorrect_with_disable_uncorrectable)
        {
            if let Some(token) = right_token {
                offenses.push(self.space_offense(token, SpaceSide::Left, message, "Do not use"));
            }
        }
        offenses
    }

    pub(crate) fn space_offenses(
        &self,
        left_token: Option<&Token<'buffer, 'source>>,
        right_token: Option<&Token<'buffer, 'source>>,
        message: &str,
        start_ok: bool,
        end_ok: bool,
    ) -> Vec<SpaceOffense<'buffer, 'source>> {
        let mut offenses = Vec::new();
        if !self.extra_space(left_token, SpaceSide::Left) && !start_ok {
            if let Some(token) = left_token {
                offenses.push(self.space_offense(token, SpaceSide::None, message, "Use"));
            }
        }
        if !self.extra_space(right_token, SpaceSide::Right)
            && !end_ok
            && (start_ok || !self.autocorrect_with_disable_uncorrectable)
        {
            if let Some(token) = right_token {
                offenses.push(self.space_offense(token, SpaceSide::None, message, "Use"));
            }
        }
        offenses
    }

    pub(crate) fn extra_space(
        &self,
        token: Option<&Token<'buffer, 'source>>,
        side: SpaceSide,
    ) -> bool {
        token.is_some_and(|token| match side {
            SpaceSide::Left => token.space_after(),
            SpaceSide::Right => token.space_before(),
            SpaceSide::None => false,
        })
    }

    pub(crate) fn reposition(
        &self,
        mut position: usize,
        step: isize,
        include_newlines: bool,
    ) -> usize {
        loop {
            let candidate = if step < 0 {
                position.checked_sub(1)
            } else {
                (position < self.buffer.len()).then_some(position)
            };
            let Some(candidate) = candidate else { break };
            let Some(character) = self.buffer.character(candidate) else {
                break;
            };
            if !(matches!(character, ' ' | '\t') || include_newlines && character == '\n') {
                break;
            }
            position = position.saturating_add_signed(step);
        }
        position
    }

    pub(crate) fn space_offense(
        &self,
        token: &Token<'buffer, 'source>,
        side: SpaceSide,
        message: &str,
        command: &str,
    ) -> SpaceOffense<'buffer, 'source> {
        SpaceOffense {
            range: self.side_space_range(token.pos(), side, false),
            message: message
                .replace("%<command>s", command)
                .replace("%{command}", command),
        }
    }

    pub(crate) fn empty_offenses(
        &self,
        config: &str,
        left: &Token<'buffer, 'source>,
        right: &Token<'buffer, 'source>,
        message: &str,
    ) -> Vec<SpaceOffense<'buffer, 'source>> {
        let range = SourceRange::new(self.buffer, left.begin_pos(), right.end_pos());
        if self.offending_empty_space(config, left, right) {
            vec![self.empty_offense(range, message, "Use one")]
        } else if self.offending_empty_no_space(config, left, right) {
            vec![self.empty_offense(range, message, "Do not use")]
        } else {
            Vec::new()
        }
    }

    pub(crate) fn empty_offense(
        &self,
        range: SourceRange<'buffer, 'source>,
        message: &str,
        command: &str,
    ) -> SpaceOffense<'buffer, 'source> {
        SpaceOffense {
            range,
            message: message
                .replace("%<command>s", command)
                .replace("%{command}", command),
        }
    }

    pub(crate) fn empty_brackets(
        &self,
        left: &Token<'buffer, 'source>,
        right: &Token<'buffer, 'source>,
        tokens: &[Token<'buffer, 'source>],
    ) -> bool {
        let left_index = tokens.iter().position(|token| token == left);
        let right_index = tokens.iter().position(|token| token == right);
        matches!((left_index, right_index), (Some(left), Some(right)) if left + 1 == right)
    }

    pub(crate) fn offending_empty_space(
        &self,
        config: &str,
        left: &Token<'buffer, 'source>,
        right: &Token<'buffer, 'source>,
    ) -> bool {
        config == "space" && !self.space_between(left, right)
    }

    pub(crate) fn offending_empty_no_space(
        &self,
        config: &str,
        left: &Token<'buffer, 'source>,
        right: &Token<'buffer, 'source>,
    ) -> bool {
        config == "no_space" && !self.no_character_between(left, right)
    }

    pub(crate) fn space_between(
        &self,
        left: &Token<'buffer, 'source>,
        right: &Token<'buffer, 'source>,
    ) -> bool {
        left.end_pos() + 1 == right.begin_pos()
            && self.buffer.character(left.end_pos()) == Some(' ')
    }

    pub(crate) fn no_character_between(
        &self,
        left: &Token<'buffer, 'source>,
        right: &Token<'buffer, 'source>,
    ) -> bool {
        left.end_pos() == right.begin_pos()
    }
}

#[cfg(test)]
mod spec;
