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
            .then(left.cop_name.cmp(right.cop_name))
    });
}

fn prism_offense(source: &str, index: &SourceIndex, finding: prism::Finding) -> Offense {
    let (line, column) = index.position(source, finding.start_offset);
    let empty_location = finding.start_offset == 0 && finding.end_offset == 0;
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
    let (last_line, last_column) = if empty_location {
        (1, 0)
    } else if ends_at_newline {
        let (line, _) = index.position(source, finding.end_offset);
        (line, 0)
    } else {
        index.position(source, last_offset)
    };
    Offense {
        cop_name: finding.cop_name,
        message: finding.message,
        corrected: finding.corrected,
        correctable: finding.correctable,
        line,
        column,
        last_line,
        last_column,
        length: finding
            .end_offset
            .saturating_sub(finding.start_offset)
            .max(1),
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
        assert_eq!((newline.last_line, newline.last_column), (2, 0));
    }
}
