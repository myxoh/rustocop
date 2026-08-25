// RuboCop 1.87.0
// Source: lib/rubocop/cop/corrector.rb
// Source SHA-256: e5be763b4a0bd2ea7aafbf8e30c1cd37cb20a64bf31cc878b1d853d652b35bf8

use std::fmt;
use std::ops::Range;

use crate::rubocop::ast::source::{SourceBuffer, SourceRange};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CorrectionError {
    InvalidRange,
    DifferentReplacements(Range<usize>),
    OverlappingReplacements(Range<usize>, Range<usize>),
}

impl fmt::Display for CorrectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRange => {
                formatter.write_str("correction range is outside the source buffer")
            }
            Self::DifferentReplacements(range) => {
                write!(formatter, "different replacements for {range:?}")
            }
            Self::OverlappingReplacements(left, right) => {
                write!(
                    formatter,
                    "overlapping replacements: {left:?} and {right:?}"
                )
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Edit {
    range: Range<usize>,
    replacement: String,
    sequence: usize,
}

#[derive(Clone)]
pub(crate) struct Corrector<'buffer, 'source> {
    source_buffer: &'buffer SourceBuffer<'source>,
    edits: Vec<Edit>,
}

impl<'buffer, 'source> Corrector<'buffer, 'source> {
    pub(crate) fn new(source_buffer: &'buffer SourceBuffer<'source>) -> Self {
        Self {
            source_buffer,
            edits: Vec::new(),
        }
    }

    pub(crate) fn source_buffer(&self) -> &'buffer SourceBuffer<'source> {
        self.source_buffer
    }

    pub(crate) fn to_range(
        &self,
        range: SourceRange<'buffer, 'source>,
    ) -> SourceRange<'buffer, 'source> {
        self.check_range_validity(range);
        range
    }

    pub(crate) fn check_range_validity(&self, range: SourceRange<'buffer, 'source>) {
        self.validate_buffer(range);
        assert!(
            range.begin_pos() <= range.end_pos() && range.end_pos() <= self.source_buffer.len(),
            "correction range is outside the source buffer"
        );
    }

    pub(crate) fn remove(&mut self, range: SourceRange<'buffer, 'source>) {
        self.replace(range, "");
    }

    pub(crate) fn insert_before(
        &mut self,
        range: SourceRange<'buffer, 'source>,
        content: impl Into<String>,
    ) {
        self.push(range.begin_pos()..range.begin_pos(), content);
    }

    pub(crate) fn insert_after(
        &mut self,
        range: SourceRange<'buffer, 'source>,
        content: impl Into<String>,
    ) {
        self.push(range.end_pos()..range.end_pos(), content);
    }

    pub(crate) fn wrap(
        &mut self,
        range: SourceRange<'buffer, 'source>,
        before: impl Into<String>,
        after: impl Into<String>,
    ) {
        self.insert_before(range, before);
        self.insert_after(range, after);
    }

    pub(crate) fn replace(
        &mut self,
        range: SourceRange<'buffer, 'source>,
        content: impl Into<String>,
    ) {
        self.validate_buffer(range);
        self.push(range.begin_pos()..range.end_pos(), content);
    }

    pub(crate) fn remove_preceding(&mut self, range: SourceRange<'buffer, 'source>, size: usize) {
        self.validate_buffer(range);
        self.push(
            range.begin_pos().saturating_sub(size)..range.begin_pos(),
            "",
        );
    }

    pub(crate) fn remove_leading(&mut self, range: SourceRange<'buffer, 'source>, size: usize) {
        self.validate_buffer(range);
        self.push(range.begin_pos()..range.begin_pos() + size, "");
    }

    pub(crate) fn remove_trailing(&mut self, range: SourceRange<'buffer, 'source>, size: usize) {
        self.validate_buffer(range);
        self.push(range.end_pos().saturating_sub(size)..range.end_pos(), "");
    }

    pub(crate) fn swap(
        &mut self,
        range1: SourceRange<'buffer, 'source>,
        range2: SourceRange<'buffer, 'source>,
    ) {
        self.validate_buffer(range1);
        self.validate_buffer(range2);

        if range1.end_pos() == range2.begin_pos() {
            self.insert_before(range1, range2.source());
            self.remove(range2);
        } else if range2.end_pos() == range1.begin_pos() {
            self.insert_before(range2, range1.source());
            self.remove(range1);
        } else {
            self.replace(range1, range2.source());
            self.replace(range2, range1.source());
        }
    }

    pub(crate) fn rewrite(mut self) -> Result<String, CorrectionError> {
        self.edits
            .sort_by_key(|edit| (edit.range.start, edit.range.end, edit.sequence));
        self.validate_edits()?;

        let mut rewritten = self.source_buffer.source().to_owned();
        for edit in self.edits.into_iter().rev() {
            let start = self
                .source_buffer
                .byte_position(edit.range.start)
                .ok_or(CorrectionError::InvalidRange)?;
            let end = self
                .source_buffer
                .byte_position(edit.range.end)
                .ok_or(CorrectionError::InvalidRange)?;
            rewritten.replace_range(start..end, &edit.replacement);
        }
        Ok(rewritten)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.edits.is_empty()
    }

    pub(crate) fn merge(&mut self, other: &Corrector<'_, '_>) {
        assert!(
            std::ptr::eq(
                self.source_buffer as *const SourceBuffer<'_> as *const (),
                other.source_buffer as *const SourceBuffer<'_> as *const (),
            ),
            "cannot merge correctors for different source buffers"
        );
        for edit in &other.edits {
            self.push(edit.range.clone(), edit.replacement.clone());
        }
    }

    /// Imports edits produced for a fragment into this corrector's source buffer.
    ///
    /// Parser::Source::TreeRewriter expresses offsets in characters, as does
    /// `SourceBuffer`, so this remains correct for UTF-8 input.
    pub(crate) fn import(
        &mut self,
        other: &Corrector<'_, '_>,
        offset: isize,
    ) -> Result<(), CorrectionError> {
        let checkpoint = self.edits.len();
        for edit in &other.edits {
            let start = edit
                .range
                .start
                .checked_add_signed(offset)
                .ok_or(CorrectionError::InvalidRange)?;
            let end = edit
                .range
                .end
                .checked_add_signed(offset)
                .ok_or(CorrectionError::InvalidRange)?;
            self.push(start..end, edit.replacement.clone());
        }
        if let Err(error) = self.validate_edits() {
            self.edits.truncate(checkpoint);
            return Err(error);
        }
        Ok(())
    }

    pub(crate) fn apply_edits(
        &mut self,
        edits: impl IntoIterator<Item = (Range<usize>, String)>,
    ) -> Result<(), CorrectionError> {
        let checkpoint = self.edits.len();
        for (range, replacement) in edits {
            self.push(range, replacement);
        }
        if let Err(error) = self.validate_edits() {
            self.edits.truncate(checkpoint);
            return Err(error);
        }
        Ok(())
    }

    pub(crate) fn transaction<F>(&mut self, correction: F)
    where
        F: FnOnce(&mut Self),
    {
        let checkpoint = self.edits.len();
        correction(self);
        if self.validate_edits().is_err() {
            self.edits.truncate(checkpoint);
        }
    }

    fn push(&mut self, range: Range<usize>, content: impl Into<String>) {
        self.edits.push(Edit {
            range,
            replacement: content.into(),
            sequence: self.edits.len(),
        });
    }

    fn validate_buffer(&self, range: SourceRange<'buffer, 'source>) {
        assert!(
            std::ptr::eq(self.source_buffer, range.buffer()),
            "correction target buffer is not the current source buffer"
        );
    }

    fn validate_edits(&self) -> Result<(), CorrectionError> {
        let mut edits = self.edits.iter().collect::<Vec<_>>();
        edits.sort_by_key(|edit| (edit.range.start, edit.range.end, edit.sequence));
        for edit in &edits {
            if edit.range.start > edit.range.end || edit.range.end > self.source_buffer.len() {
                return Err(CorrectionError::InvalidRange);
            }
        }
        for pair in edits.windows(2) {
            let (left, right) = (&pair[0], &pair[1]);
            if left.range == right.range
                && left.range.start != left.range.end
                && left.replacement != right.replacement
            {
                return Err(CorrectionError::DifferentReplacements(left.range.clone()));
            }
            if left.range.start < right.range.end
                && right.range.start < left.range.end
                && left.range != right.range
            {
                return Err(CorrectionError::OverlappingReplacements(
                    left.range.clone(),
                    right.range.clone(),
                ));
            }
        }
        Ok(())
    }
}
