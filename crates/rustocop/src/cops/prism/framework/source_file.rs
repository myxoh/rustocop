use std::ops::Range;

use ruby_prism::{Location, Node};

/// A source-relative edit used while rendering a larger correction.
///
/// This is deliberately separate from the correction engine's final edits:
/// cops use it to construct one replacement without hand-rolling reverse
/// sorting, offset rebasing, and overlap checks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SourceEdit {
    pub(super) range: Range<usize>,
    pub(super) replacement: String,
}

impl SourceEdit {
    pub(super) fn replace(range: Range<usize>, replacement: impl Into<String>) -> Self {
        Self {
            range,
            replacement: replacement.into(),
        }
    }

    pub(super) fn remove(range: Range<usize>) -> Self {
        Self::replace(range, "")
    }
}

/// A safe, allocation-free view over the source currently being inspected.
#[derive(Clone, Copy)]
pub(super) struct SourceFile<'source> {
    source: &'source str,
}

// Source geometry is intentionally a ready-to-use authoring API; individual
// helpers become live as layout and literal cop families migrate.
#[allow(dead_code)]
impl<'source> SourceFile<'source> {
    pub(super) fn new(source: &'source str) -> Self {
        Self { source }
    }

    pub(super) fn as_str(self) -> &'source str {
        self.source
    }

    pub(super) fn slice(self, range: Range<usize>) -> Option<&'source str> {
        self.source.get(range)
    }

    pub(super) fn at(self, location: &Location<'_>) -> &'source str {
        self.source
            .get(location.start_offset()..location.end_offset())
            .unwrap_or_default()
    }

    pub(super) fn node(self, node: &Node<'_>) -> &'source str {
        self.at(&node.location())
    }

    pub(super) fn node_range(self, node: &Node<'_>) -> Range<usize> {
        let location = node.location();
        location.start_offset()..location.end_offset()
    }

    pub(super) fn lines(self) -> impl Iterator<Item = (usize, &'source str)> {
        self.source.split_inclusive('\n').scan(0, |offset, line| {
            let start = *offset;
            *offset += line.len();
            Some((start, line.strip_suffix('\n').unwrap_or(line)))
        })
    }

    pub(super) fn line_range(self, offset: usize) -> Range<usize> {
        let offset = self.char_boundary_at_or_before(offset);
        let start = self.line_start(offset);
        let end = self.source[offset..]
            .find('\n')
            .map_or(self.source.len(), |position| offset + position + 1);
        start..end
    }

    /// Full physical lines touched by `range`, including a trailing newline
    /// when one exists. This is useful when removing complete statements.
    pub(super) fn full_line_range(self, range: Range<usize>) -> Range<usize> {
        let start = self.line_start(range.start);
        let end = if range.end >= self.source.len() {
            self.source.len()
        } else {
            let range_end = self.char_boundary_at_or_before(range.end);
            self.source[range_end..]
                .find('\n')
                .map_or(range_end, |position| range_end + position + 1)
        };
        start..end
    }

    /// Returns the byte offset of the physical line containing `offset`.
    pub(super) fn line_start(self, offset: usize) -> usize {
        let offset = self.char_boundary_at_or_before(offset);
        self.source[..offset]
            .rfind('\n')
            .map_or(0, |position| position + 1)
    }

    /// Returns the byte offset immediately before the line ending. For CRLF,
    /// both line-ending bytes are excluded.
    pub(super) fn line_end(self, offset: usize) -> usize {
        let offset = self.char_boundary_at_or_before(offset);
        let end = self.source[offset..]
            .find('\n')
            .map_or(self.source.len(), |position| offset + position);
        end.checked_sub(1)
            .filter(|before| self.source.as_bytes().get(*before) == Some(&b'\r'))
            .unwrap_or(end)
    }

    pub(super) fn line(self, offset: usize) -> &'source str {
        &self.source[self.line_start(offset)..self.line_end(offset)]
    }

    /// Leading horizontal whitespace on the line containing `offset`.
    pub(super) fn indentation(self, offset: usize) -> Range<usize> {
        let start = self.line_start(offset);
        let width = self.source[start..self.line_end(offset)]
            .bytes()
            .take_while(|byte| matches!(byte, b' ' | b'\t'))
            .count();
        start..start + width
    }

    pub(super) fn indentation_text(self, offset: usize) -> &'source str {
        self.slice(self.indentation(offset)).unwrap_or_default()
    }

    /// Applies non-overlapping absolute edits inside `container` and returns
    /// the rendered replacement for that container. Invalid or overlapping
    /// edits fail as a group instead of producing a partial correction.
    pub(super) fn rewrite(
        self,
        container: Range<usize>,
        mut edits: Vec<SourceEdit>,
    ) -> Option<String> {
        let mut rendered = self.slice(container.clone())?.to_string();
        edits.sort_by_key(|edit| (edit.range.start, edit.range.end));
        let valid = edits.iter().all(|edit| {
            container.start <= edit.range.start
                && edit.range.start <= edit.range.end
                && edit.range.end <= container.end
        }) && edits
            .windows(2)
            .all(|pair| pair[0].range.end <= pair[1].range.start);
        if !valid {
            return None;
        }
        for edit in edits.into_iter().rev() {
            rendered.replace_range(
                edit.range.start - container.start..edit.range.end - container.start,
                &edit.replacement,
            );
        }
        Some(rendered)
    }

    pub(super) fn same_line(self, left: usize, right: usize) -> bool {
        self.line_start(left) == self.line_start(right)
    }

    /// Zero-based character column. Prism locations use byte offsets, while
    /// RuboCop diagnostics and layout rules reason in characters.
    pub(super) fn column(self, offset: usize) -> usize {
        let offset = self.char_boundary_at_or_before(offset);
        self.source[self.line_start(offset)..offset].chars().count()
    }

    pub(super) fn whitespace_before(self, offset: usize) -> Range<usize> {
        let end = self.char_boundary_at_or_before(offset);
        let start = self.source[..end]
            .trim_end_matches(char::is_whitespace)
            .len();
        start..end
    }

    pub(super) fn whitespace_after(self, offset: usize) -> Range<usize> {
        let start = self.char_boundary_at_or_before(offset);
        let count = self.source[start..]
            .chars()
            .take_while(|character| character.is_whitespace())
            .map(char::len_utf8)
            .sum::<usize>();
        start..start + count
    }

    fn char_boundary_at_or_before(self, offset: usize) -> usize {
        let mut offset = offset.min(self.source.len());
        while !self.source.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    }

    /// Finds source text outside ordinary quoted strings and line comments.
    /// AST locations remain preferable; this is for genuinely source-wide
    /// conventions such as magic comments and punctuation layout.
    pub(super) fn code_offsets(self, needle: &str) -> Vec<usize> {
        let bytes = self.source.as_bytes();
        let needle = needle.as_bytes();
        if needle.is_empty() {
            return Vec::new();
        }
        let mut offsets = Vec::new();
        let mut quote = None;
        let mut escaped = false;
        let mut comment = false;
        let mut index = 0;
        while index < bytes.len() {
            let byte = bytes[index];
            if comment {
                comment = byte != b'\n';
            } else if let Some(delimiter) = quote {
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == delimiter {
                    quote = None;
                }
            } else if byte == b'#' {
                comment = true;
            } else if matches!(byte, b'\'' | b'"') {
                quote = Some(byte);
            } else if bytes[index..].starts_with(needle) {
                offsets.push(index);
                index += needle.len().saturating_sub(1);
            }
            index += 1;
        }
        offsets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_safe_lines_ranges_and_whitespace() {
        let source = SourceFile::new("one  \n  two\n");
        assert_eq!(
            source.lines().collect::<Vec<_>>(),
            [(0, "one  "), (6, "  two")]
        );
        assert_eq!(source.line_range(8), 6..12);
        assert_eq!(source.line_start(8), 6);
        assert_eq!(source.line_end(8), 11);
        assert_eq!(source.line(8), "  two");
        assert_eq!(source.indentation(8), 6..8);
        assert_eq!(source.indentation_text(8), "  ");
        assert_eq!(source.full_line_range(8..9), 6..12);
        assert!(source.same_line(6, 10));
        assert!(!source.same_line(5, 6));
        assert_eq!(source.column(8), 2);
        assert_eq!(source.whitespace_before(6), 3..6);
        assert_eq!(source.whitespace_after(6), 6..8);
        assert_eq!(source.slice(6..9), Some("  t"));
    }

    #[test]
    fn rewrites_a_container_atomically() {
        let source = SourceFile::new("before [one, two] after");
        assert_eq!(
            source.rewrite(
                7..17,
                vec![SourceEdit::replace(8..11, "1"), SourceEdit::remove(11..16),],
            ),
            Some("[1]".to_string())
        );
        assert_eq!(
            source.rewrite(
                7..17,
                vec![SourceEdit::remove(8..13), SourceEdit::remove(10..16)],
            ),
            None
        );
        assert_eq!(source.rewrite(7..17, vec![SourceEdit::remove(0..2)]), None);
    }

    #[test]
    fn finds_code_without_matching_strings_or_comments() {
        let source = SourceFile::new("call; other\n'not; code'\n# hidden; token\nnext; value\n");
        assert_eq!(source.code_offsets(";"), [4, 44]);
    }

    #[test]
    fn handles_crlf_and_character_columns() {
        let source = SourceFile::new("  café\r\n\tvalue\r\n");
        let cafe_end = "  café".len();
        assert_eq!(source.line_end(2), cafe_end);
        assert_eq!(source.line(2), "  café");
        assert_eq!(source.column(cafe_end), 6);
        assert_eq!(source.indentation(cafe_end + 2), cafe_end + 2..cafe_end + 3);
    }

    #[test]
    fn accepts_offsets_inside_multibyte_characters() {
        let text = "one\n  なまえ\n  [:🇺🇸]\n";
        let source = SourceFile::new(text);
        for offset in 0..=text.len() {
            let _ = source.line_start(offset);
            let _ = source.line_end(offset);
            let _ = source.line(offset);
            let _ = source.column(offset);
        }
        let name = text.find('な').unwrap();
        assert_eq!(source.line(name + 1), "  なまえ");
        assert_eq!(source.column(name + 1), 2);
    }
}
