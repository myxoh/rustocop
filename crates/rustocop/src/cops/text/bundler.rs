use super::{Offense, SourceLine};
use crate::config::InspectionConfig;

pub(super) fn before_prism(
    path: &str,
    lines: &mut [SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let _ = (path, lines, options, offenses);
}
