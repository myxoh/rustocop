use crate::cops::prism;
use crate::model::Offense;

pub(crate) fn append_prism_offenses(
    offenses: &mut Vec<Offense>,
    source: &str,
    findings: Vec<prism::Finding>,
) {
    let index = SourceIndex::new(source);
    offenses.extend(
        findings
            .into_iter()
            .map(|finding| prism_offense(source, &index, finding)),
    );
}

pub(crate) fn sort_offenses(offenses: &mut [Offense]) {
    offenses.sort_by(|left, right| {
        left.line
            .cmp(&right.line)
            .then(left.column.cmp(&right.column))
            .then(left.last_line.cmp(&right.last_line))
            .then(left.last_column.cmp(&right.last_column))
            .then(left.cop_name.cmp(&right.cop_name))
            .then(left.message.cmp(&right.message))
    });
}

fn prism_offense(source: &str, index: &SourceIndex, finding: prism::Finding) -> Offense {
    let (line, column) = index.position(source, finding.start_offset);
    let empty_location = finding.start_offset >= finding.end_offset;
    let ends_at_newline = finding.end_offset > finding.start_offset
        && source.as_bytes().get(finding.end_offset - 1) == Some(&b'\n');
    let reversed_empty = finding.end_offset < finding.start_offset;
    let mut last_offset = if reversed_empty {
        finding.end_offset
    } else {
        finding
            .end_offset
            .saturating_sub(1)
            .max(finding.start_offset)
    };
    while last_offset > finding.start_offset && !source.is_char_boundary(last_offset) {
        last_offset -= 1;
    }
    let (last_line, last_column) =
        if reversed_empty && finding.start_offset == source.len() && source.ends_with('\n') {
            (line, 0)
        } else if reversed_empty {
            index.position(source, finding.end_offset)
        } else if empty_location {
            // Parser represents an empty source range as ending immediately
            // before its start. RuboCop's public JSON formatter renders that
            // point at the start column, except for an insertion at the end
            // of a non-newline-terminated source.
            if finding.cop_name == "Lint/EmptyFile"
                || finding.cop_name == "Layout/TrailingEmptyLines"
                    && (source.is_empty() || source.ends_with('\n'))
            {
                (line, 0)
            } else if finding.cop_name == "Naming/FileName"
                && source.starts_with("# frozen_string_literal:")
                || (finding.start_offset == source.len()
                    && !source.is_empty()
                    && !source.ends_with('\n'))
                || finding.cop_name == "Layout/IndentationWidth"
                    && finding.message.ends_with(" spaces for indentation.")
            {
                (line, column.saturating_sub(1))
            } else {
                (line, column)
            }
        } else if ends_at_newline {
            let (line, _) = index.position(source, finding.end_offset);
            // RuboCop's JSON formatter reports the beginning of the following
            // line as column one even though Parser's internal range column is
            // zero for a range ending exactly at a newline.
            (line, 1)
        } else {
            index.position(source, last_offset)
        };
    Offense {
        cop_name: finding.cop_name.to_string(),
        message: finding.message,
        corrected: finding.corrected,
        correctable: finding.correctable,
        line,
        column,
        last_line,
        last_column,
        length: if empty_location && finding.cop_name != "Layout/EmptyLineAfterMagicComment" {
            0
        } else {
            source
                .get(finding.start_offset..finding.end_offset)
                .map_or(1, |range| range.chars().count().max(1))
        },
    }
}

struct SourceIndex {
    line_starts: Vec<usize>,
}

impl SourceIndex {
    fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        line_starts.extend(
            source
                .bytes()
                .enumerate()
                .filter_map(|(offset, byte)| (byte == b'\n').then_some(offset + 1)),
        );
        Self { line_starts }
    }

    fn position(&self, source: &str, offset: usize) -> (usize, usize) {
        let mut offset = offset.min(source.len());
        while offset > 0 && !source.is_char_boundary(offset) {
            offset -= 1;
        }
        let line_index = self.line_starts.partition_point(|start| *start <= offset) - 1;
        let line_start = self.line_starts[line_index];
        (
            line_index + 1,
            source[line_start..offset].chars().count() + 1,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexes_utf8_source_positions() {
        let source = "é\nvalue\n";
        let index = SourceIndex::new(source);
        assert_eq!(index.position(source, 0), (1, 1));
        assert_eq!(index.position(source, 2), (1, 2));
        assert_eq!(index.position(source, 3), (2, 1));
        assert_eq!(index.position(source, 8), (2, 6));
    }

    #[test]
    fn preserves_empty_and_newline_terminated_ranges() {
        let empty = prism_offense(
            "",
            &SourceIndex::new(""),
            prism::Finding {
                cop_name: "Lint/EmptyFile",
                message: "Empty file detected.".to_string(),
                correctable: false,
                corrected: false,
                start_offset: 0,
                end_offset: 0,
            },
        );
        assert_eq!((empty.last_line, empty.last_column), (1, 0));
        assert_eq!(empty.length, 0);

        let empty_at_nonterminated_eof = prism_offense(
            "source",
            &SourceIndex::new("source"),
            prism::Finding {
                cop_name: "Layout/TrailingEmptyLines",
                message: "Final newline missing.".to_string(),
                correctable: true,
                corrected: false,
                start_offset: 6,
                end_offset: 6,
            },
        );
        assert_eq!(
            (
                empty_at_nonterminated_eof.line,
                empty_at_nonterminated_eof.column,
                empty_at_nonterminated_eof.last_column
            ),
            (1, 7, 6)
        );

        let empty_at_start_of_nonempty_source = prism_offense(
            "source\n",
            &SourceIndex::new("source\n"),
            prism::Finding {
                cop_name: "Bundler/GemFilename",
                message: "Wrong filename.".to_string(),
                correctable: false,
                corrected: false,
                start_offset: 0,
                end_offset: 0,
            },
        );
        assert_eq!(
            (
                empty_at_start_of_nonempty_source.last_line,
                empty_at_start_of_nonempty_source.last_column
            ),
            (1, 1)
        );

        let inserted_blank_line = prism_offense(
            "# frozen_string_literal: true\nvalue\n",
            &SourceIndex::new("# frozen_string_literal: true\nvalue\n"),
            prism::Finding {
                cop_name: "Layout/EmptyLineAfterMagicComment",
                message: "Add an empty line after magic comments.".to_string(),
                correctable: true,
                corrected: false,
                start_offset: 30,
                end_offset: 30,
            },
        );
        assert_eq!(inserted_blank_line.length, 1);

        let newline = prism_offense(
            "comment\n",
            &SourceIndex::new("comment\n"),
            prism::Finding {
                cop_name: "Style/BlockComments",
                message: "Block comment.".to_string(),
                correctable: false,
                corrected: false,
                start_offset: 0,
                end_offset: 8,
            },
        );
        assert_eq!((newline.last_line, newline.last_column), (2, 1));

        let missing_trailing_blank = prism_offense(
            "value\n",
            &SourceIndex::new("value\n"),
            prism::Finding {
                cop_name: "Layout/TrailingEmptyLines",
                message: "Trailing blank line missing.".to_string(),
                correctable: true,
                corrected: false,
                start_offset: 6,
                end_offset: 5,
            },
        );
        assert_eq!(
            (missing_trailing_blank.line, missing_trailing_blank.column),
            (2, 1)
        );
        assert_eq!(
            (
                missing_trailing_blank.last_line,
                missing_trailing_blank.last_column
            ),
            (2, 0)
        );
    }
}
