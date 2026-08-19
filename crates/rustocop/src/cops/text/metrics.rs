use super::{Offense, SourceLine};
use crate::config::InspectionConfig;

pub(super) fn after_prism(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let _ = (lines, options, offenses);
}
