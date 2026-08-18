pub(super) fn source_lines(source: &str) -> impl Iterator<Item = (usize, &str)> {
    super::SourceFile::new(source).lines()
}

pub(super) fn line_end(source: &str, start: usize) -> usize {
    source[start..]
        .find('\n')
        .map_or(source.len(), |offset| start + offset + 1)
}

pub(super) fn all_offsets<'source>(
    source: &'source str,
    needle: &'source str,
) -> impl Iterator<Item = usize> + 'source {
    source.match_indices(needle).map(|(offset, _)| offset)
}

pub(super) fn identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}
