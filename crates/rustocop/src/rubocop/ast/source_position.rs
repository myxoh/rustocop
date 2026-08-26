use std::ops::Range;

/// Maps Prism byte offsets to RuboCop's character-based source positions.
///
/// RuboCop compatibility nodes ask for these conversions frequently. Scanning
/// the source prefix for every node is quadratic on generated Ruby files, so
/// keep one compact index for the duration of translation or lexing.
pub(crate) struct SourcePositionIndex {
    ascii: bool,
    char_byte_offsets: Vec<usize>,
    line_start_bytes: Vec<usize>,
    line_start_characters: Vec<usize>,
}

impl SourcePositionIndex {
    pub(crate) fn new(source: &str) -> Self {
        let ascii = source.is_ascii();
        let mut char_byte_offsets = Vec::new();
        let mut line_start_bytes = vec![0];
        let mut line_start_characters = vec![0];
        for (character_offset, (byte_offset, character)) in source.char_indices().enumerate() {
            if !ascii {
                char_byte_offsets.push(byte_offset);
            }
            if character == '\n' {
                line_start_bytes.push(byte_offset + 1);
                line_start_characters.push(character_offset + 1);
            }
        }
        if !ascii {
            char_byte_offsets.push(source.len());
        }
        Self {
            ascii,
            char_byte_offsets,
            line_start_bytes,
            line_start_characters,
        }
    }

    pub(crate) fn character_offset(&self, byte_offset: usize) -> usize {
        if self.ascii {
            byte_offset
        } else {
            self.char_byte_offsets
                .partition_point(|candidate| *candidate < byte_offset)
        }
    }

    pub(crate) fn character_range(&self, range: Range<usize>) -> Range<usize> {
        self.character_offset(range.start)..self.character_offset(range.end)
    }

    pub(crate) fn line_for_byte(&self, byte_offset: usize) -> usize {
        self.line_start_bytes
            .partition_point(|start| *start <= byte_offset)
    }

    pub(crate) fn column_for_byte(&self, byte_offset: usize) -> usize {
        let line = self
            .line_start_bytes
            .partition_point(|start| *start <= byte_offset)
            .saturating_sub(1);
        self.character_offset(byte_offset) - self.line_start_characters[line]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexes_ascii_and_unicode_positions() {
        let ascii = SourcePositionIndex::new("one\ntwo");
        assert_eq!(ascii.character_range(4..7), 4..7);
        assert_eq!(ascii.line_for_byte(4), 2);
        assert_eq!(ascii.column_for_byte(6), 2);

        let unicode = SourcePositionIndex::new("café\nなまえ");
        let name = "café\n".len();
        assert_eq!(unicode.character_offset(name), 5);
        assert_eq!(unicode.line_for_byte(name), 2);
        assert_eq!(unicode.column_for_byte(name + "な".len()), 1);
    }
}
