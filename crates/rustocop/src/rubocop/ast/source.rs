use std::ops::Range;

/// Rust translation of the source geometry exposed by
/// `Parser::Source::Buffer` and `Parser::Source::Range` to RuboCop.
#[derive(Clone, Debug)]
pub(crate) struct SourceBuffer<'source> {
    source: &'source str,
    char_to_byte: Vec<usize>,
    line_starts: Vec<usize>,
}

impl<'source> SourceBuffer<'source> {
    pub(crate) fn new(source: &'source str) -> Self {
        let mut line_starts = vec![0];
        let mut char_to_byte = Vec::with_capacity(source.chars().count() + 1);
        for (character_offset, (byte_offset, character)) in source.char_indices().enumerate() {
            char_to_byte.push(byte_offset);
            if character == '\n' {
                line_starts.push(character_offset + 1);
            }
        }
        char_to_byte.push(source.len());
        Self {
            source,
            char_to_byte,
            line_starts,
        }
    }

    pub(crate) fn source(&self) -> &'source str {
        self.source
    }

    pub(crate) fn source_range(&self) -> SourceRange<'_, 'source> {
        SourceRange::new(self, 0, self.len())
    }

    pub(crate) fn len(&self) -> usize {
        self.char_to_byte.len() - 1
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn line_start(&self, line_number: usize) -> usize {
        if line_number == 0 {
            0
        } else {
            self.line_starts
                .get(line_number - 1)
                .copied()
                .unwrap_or(self.len())
        }
    }

    pub(crate) fn line_range(&self, line_number: usize) -> Range<usize> {
        let start = self.line_start(line_number);
        let next = self
            .line_starts
            .get(line_number)
            .copied()
            .unwrap_or(self.len());
        let mut end = next;
        if self.character(end.saturating_sub(1)) == Some('\n') {
            end -= 1;
        }
        if self.character(end.saturating_sub(1)) == Some('\r') {
            end -= 1;
        }
        start..end
    }

    pub(crate) fn source_line(&self, line_number: usize) -> &'source str {
        self.slice(self.line_range(line_number))
    }

    pub(crate) fn character(&self, offset: usize) -> Option<char> {
        let start = *self.char_to_byte.get(offset)?;
        let end = *self.char_to_byte.get(offset + 1)?;
        self.source.get(start..end)?.chars().next()
    }

    pub(crate) fn slice(&self, range: Range<usize>) -> &'source str {
        let Some(&start) = self.char_to_byte.get(range.start) else {
            return "";
        };
        let Some(&end) = self.char_to_byte.get(range.end) else {
            return "";
        };
        self.source.get(start..end).unwrap_or_default()
    }

    pub(crate) fn byte_position(&self, character_offset: usize) -> Option<usize> {
        self.char_to_byte.get(character_offset).copied()
    }

    pub(crate) fn character_position(&self, byte_offset: usize) -> Option<usize> {
        self.char_to_byte.binary_search(&byte_offset).ok()
    }

    pub(crate) fn line_index_for_byte(&self, byte_offset: usize) -> usize {
        let byte_offset = byte_offset.min(self.source.len());
        let character_offset = self.character_position(byte_offset).unwrap_or_else(|| {
            self.char_to_byte
                .partition_point(|position| *position < byte_offset)
        });
        self.line_index(character_offset)
    }

    pub(crate) fn line_start_byte_at_index(&self, index: usize) -> usize {
        self.byte_position(self.line_start(index + 1))
            .unwrap_or(self.source.len())
    }

    fn line_index(&self, offset: usize) -> usize {
        let offset = offset.min(self.len());
        self.line_starts.partition_point(|start| *start <= offset) - 1
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SourceRange<'buffer, 'source> {
    buffer: &'buffer SourceBuffer<'source>,
    begin_pos: usize,
    end_pos: usize,
}

impl<'buffer, 'source> SourceRange<'buffer, 'source> {
    pub(crate) fn new(
        buffer: &'buffer SourceBuffer<'source>,
        begin_pos: usize,
        end_pos: usize,
    ) -> Self {
        assert!(begin_pos <= end_pos, "source range must be ordered");
        assert!(end_pos <= buffer.len(), "source range exceeds buffer");
        Self {
            buffer,
            begin_pos,
            end_pos,
        }
    }

    pub(crate) fn from_byte_range(
        buffer: &'buffer SourceBuffer<'source>,
        range: Range<usize>,
    ) -> Option<Self> {
        Some(Self::new(
            buffer,
            buffer.character_position(range.start)?,
            buffer.character_position(range.end)?,
        ))
    }

    pub(crate) fn byte_range(self) -> Range<usize> {
        self.buffer.byte_position(self.begin_pos).unwrap_or(0)
            ..self
                .buffer
                .byte_position(self.end_pos)
                .unwrap_or(self.buffer.source().len())
    }

    pub(crate) fn buffer(self) -> &'buffer SourceBuffer<'source> {
        self.buffer
    }

    pub(crate) fn begin_pos(self) -> usize {
        self.begin_pos
    }

    pub(crate) fn end_pos(self) -> usize {
        self.end_pos
    }

    pub(crate) fn source(self) -> &'source str {
        self.buffer.slice(self.begin_pos..self.end_pos)
    }

    pub(crate) fn is_empty(self) -> bool {
        self.begin_pos == self.end_pos
    }

    pub(crate) fn len(self) -> usize {
        self.end_pos - self.begin_pos
    }

    pub(crate) fn overlaps(self, other: Self) -> bool {
        assert!(std::ptr::eq(self.buffer, other.buffer));
        self.begin_pos < other.end_pos && other.begin_pos < self.end_pos
    }

    pub(crate) fn contains(self, other: Self) -> bool {
        assert!(std::ptr::eq(self.buffer, other.buffer));
        self.begin_pos <= other.begin_pos && self.end_pos >= other.end_pos
    }

    pub(crate) fn resize(self, size: usize) -> Self {
        Self::new(
            self.buffer,
            self.begin_pos,
            self.begin_pos.saturating_add(size).min(self.buffer.len()),
        )
    }

    pub(crate) fn end(self) -> Self {
        Self::new(self.buffer, self.end_pos, self.end_pos)
    }

    pub(crate) fn line(self) -> usize {
        self.buffer.line_index(self.begin_pos) + 1
    }

    pub(crate) fn last_line(self) -> usize {
        self.buffer.line_index(self.end_pos) + 1
    }

    pub(crate) fn column(self) -> usize {
        self.begin_pos - self.buffer.line_starts[self.buffer.line_index(self.begin_pos)]
    }

    pub(crate) fn last_column(self) -> usize {
        self.end_pos - self.buffer.line_starts[self.buffer.line_index(self.end_pos)]
    }

    pub(crate) fn join(self, other: Self) -> Self {
        assert!(std::ptr::eq(self.buffer, other.buffer));
        Self::new(
            self.buffer,
            self.begin_pos.min(other.begin_pos),
            self.end_pos.max(other.end_pos),
        )
    }

    pub(crate) fn adjust(self, begin_delta: isize, end_delta: isize) -> Self {
        let begin = self.begin_pos.saturating_add_signed(begin_delta);
        let end = self.end_pos.saturating_add_signed(end_delta);
        Self::new(self.buffer, begin, end)
    }

    pub(crate) fn intersect(self, other: Self) -> Self {
        assert!(std::ptr::eq(self.buffer, other.buffer));
        let begin = self.begin_pos.max(other.begin_pos);
        let end = self.end_pos.min(other.end_pos).max(begin);
        Self::new(self.buffer, begin, end)
    }
}

impl PartialEq for SourceRange<'_, '_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.buffer, other.buffer)
            && self.begin_pos == other.begin_pos
            && self.end_pos == other.end_pos
    }
}

impl Eq for SourceRange<'_, '_> {}
