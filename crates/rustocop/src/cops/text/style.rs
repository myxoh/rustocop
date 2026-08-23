use super::{Offense, SourceLine};
use crate::config::InspectionConfig;

pub(super) fn before_prism(
    lines: &mut Vec<SourceLine>,
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let _ = (lines, options, offenses);
}

pub(super) fn after_prism(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    super::style_declarations::check(lines, options, offenses);
}
