use std::ops::Range;

use ruby_prism::{Location, Node};

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

    pub(super) fn lines(self) -> impl Iterator<Item = (usize, &'source str)> {
        self.source.split_inclusive('\n').scan(0, |offset, line| {
            let start = *offset;
            *offset += line.len();
            Some((start, line.strip_suffix('\n').unwrap_or(line)))
        })
    }

    pub(super) fn line_range(self, offset: usize) -> Range<usize> {
        let offset = offset.min(self.source.len());
        let start = self.source[..offset]
            .rfind('\n')
            .map_or(0, |position| position + 1);
        let end = self.source[offset..]
            .find('\n')
            .map_or(self.source.len(), |position| offset + position + 1);
        start..end
    }

    pub(super) fn whitespace_before(self, offset: usize) -> Range<usize> {
        let end = offset.min(self.source.len());
        let start = self.source[..end]
            .trim_end_matches(char::is_whitespace)
            .len();
        start..end
    }

    pub(super) fn whitespace_after(self, offset: usize) -> Range<usize> {
        let start = offset.min(self.source.len());
        let count = self.source[start..]
            .chars()
            .take_while(|character| character.is_whitespace())
            .map(char::len_utf8)
            .sum::<usize>();
        start..start + count
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
        assert_eq!(source.whitespace_before(6), 3..6);
        assert_eq!(source.whitespace_after(6), 6..8);
        assert_eq!(source.slice(6..9), Some("  t"));
    }

    #[test]
    fn finds_code_without_matching_strings_or_comments() {
        let source = SourceFile::new("call; other\n'not; code'\n# hidden; token\nnext; value\n");
        assert_eq!(source.code_offsets(";"), [4, 44]);
    }
}
