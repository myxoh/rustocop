use std::cmp::Reverse;
use std::ops::Range;

pub(super) struct Edit {
    pub(super) range: Range<usize>,
    pub(super) replacement: String,
}

pub(super) struct Correction {
    pub(super) finding_index: usize,
    pub(super) edits: Vec<Edit>,
}

pub(super) fn accepted_corrections(
    source: &str,
    mut corrections: Vec<Correction>,
) -> (Vec<Correction>, Vec<usize>) {
    for correction in &mut corrections {
        correction
            .edits
            .sort_by_key(|edit| (edit.range.start, edit.range.end));
    }
    corrections.sort_by_key(|correction| {
        correction
            .edits
            .first()
            .map_or((usize::MAX, Reverse(0)), |first| {
                (
                    first.range.start,
                    Reverse(
                        correction
                            .edits
                            .last()
                            .map_or(first.range.end, |last| last.range.end),
                    ),
                )
            })
    });
    let mut accepted = Vec::new();
    let mut accepted_ranges = Vec::<Range<usize>>::new();
    let mut accepted_containers = Vec::<Range<usize>>::new();
    let mut subsumed = Vec::new();
    for correction in corrections {
        let valid = !correction.edits.is_empty()
            && correction
                .edits
                .iter()
                .all(|edit| edit.range.start <= edit.range.end && edit.range.end <= source.len())
            && correction
                .edits
                .windows(2)
                .all(|pair| pair[0].range.end <= pair[1].range.start);
        if !valid {
            continue;
        }
        let conflicts = correction.edits.iter().any(|edit| {
            accepted_ranges
                .iter()
                .any(|range| ranges_overlap(&edit.range, range))
        });
        if conflicts {
            if correction.edits.iter().all(|edit| {
                accepted_containers
                    .iter()
                    .any(|range| range.start <= edit.range.start && edit.range.end <= range.end)
            }) {
                subsumed.push(correction.finding_index);
            }
            continue;
        }
        let contiguous = correction
            .edits
            .windows(2)
            .all(|pair| pair[0].range.end == pair[1].range.start);
        if contiguous {
            let first = &correction.edits[0].range;
            let last = &correction.edits[correction.edits.len() - 1].range;
            accepted_containers.push(first.start..last.end);
        }
        accepted_ranges.extend(correction.edits.iter().map(|edit| edit.range.clone()));
        accepted.push(correction);
    }
    (accepted, subsumed)
}

fn ranges_overlap(left: &Range<usize>, right: &Range<usize>) -> bool {
    left.start < right.end && right.start < left.end
}

pub(super) fn apply_edits(source: &str, mut edits: Vec<Edit>) -> String {
    edits.sort_by_key(|edit| (edit.range.start, edit.range.end));
    let mut corrected = source.to_string();
    for edit in edits.into_iter().rev() {
        corrected.replace_range(edit.range, &edit.replacement);
    }
    corrected
}
