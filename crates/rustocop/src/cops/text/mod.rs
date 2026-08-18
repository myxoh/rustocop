mod bundler;
mod extensions;
mod helpers;
mod layout;
mod lint;
mod lint_semantic;
mod metrics;
mod style;
mod style_declarations;

use crate::config::InspectionConfig;
pub(crate) use crate::model::{push_offense, Offense, SourceLine};

pub(crate) fn before_prism(
    path: &str,
    lines: &mut Vec<SourceLine>,
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    layout::before_prism(lines, options, offenses);
    lint::before_prism(lines, options, offenses);
    style::before_prism(lines, options, offenses);
    bundler::before_prism(path, lines, options, offenses);
}

pub(crate) fn after_prism(
    path: &str,
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    layout::after_prism(lines, options, offenses);
    style::after_prism(lines, options, offenses);
    lint::after_prism(lines, options, offenses);
    metrics::after_prism(lines, options, offenses);
    extensions::after_prism(path, lines, options, offenses);
}
