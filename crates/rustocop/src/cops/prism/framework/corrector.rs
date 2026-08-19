use std::ops::Range;

/// A set of source edits that must be accepted or rejected together.
///
/// This is the Rustocop counterpart to the corrector yielded by RuboCop's
/// `add_offense` DSL. Cops describe edits here; the reporter validates and
/// applies the complete transaction atomically.
#[derive(Default)]
pub(super) struct CorrectionPlan {
    edits: Vec<(Range<usize>, String)>,
}

impl CorrectionPlan {
    pub(super) fn replace(&mut self, range: Range<usize>, replacement: impl Into<String>) {
        self.edits.push((range, replacement.into()));
    }

    pub(super) fn swap(&mut self, source: &str, left: Range<usize>, right: Range<usize>) -> bool {
        if left.end > right.start {
            return false;
        }
        let (Some(left_source), Some(right_source)) =
            (source.get(left.clone()), source.get(right.clone()))
        else {
            return false;
        };
        self.replace(left, right_source);
        self.replace(right, left_source);
        true
    }

    pub(super) fn into_edits(self) -> Vec<(Range<usize>, String)> {
        self.edits
    }
}
