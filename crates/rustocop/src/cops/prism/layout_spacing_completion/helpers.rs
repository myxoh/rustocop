use super::*;

pub(super) fn aligned_argument_column(
    source: &str,
    current_offset: usize,
    column: usize,
) -> bool {
    SourceFile::new(source).lines().any(|(offset, line)| {
        offset != current_offset
            && line.len() > column
            && line
                .as_bytes()
                .get(column)
                .is_some_and(|byte| !byte.is_ascii_whitespace())
    })
}
