mod bundler;
mod extensions;
mod helpers;
mod layout;
mod lint;
mod metrics;
mod style;

pub(crate) use crate::diagnostic::{push_offense, Offense};
pub(crate) use crate::source_lines::SourceLine;
use crate::Options;

pub(crate) fn before_prism(
    path: &str,
    lines: &mut Vec<SourceLine>,
    options: &Options,
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
    options: &Options,
    offenses: &mut Vec<Offense>,
) {
    layout::after_prism(lines, options, offenses);
    style::after_prism(lines, options, offenses);
    lint::after_prism(lines, options, offenses);
    metrics::after_prism(lines, options, offenses);
    extensions::after_prism(path, lines, options, offenses);
}
