use super::helpers::*;
use super::{push_offense, CorrectionStatus, Offense, SourceLine};
use crate::config::InspectionConfig;

pub(super) fn before_prism(
    lines: &mut [SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    check_small_line_cops(lines, options, offenses);
}

pub(super) fn after_prism(
    _lines: &[SourceLine],
    _options: &InspectionConfig,
    _offenses: &mut Vec<Offense>,
) {
}

#[allow(clippy::too_many_lines)]
fn check_small_line_cops(
    lines: &mut [SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let mut begin_without_rescue = Vec::new();
    for (index, line) in lines.iter_mut().enumerate() {
        let original = line.body.clone();
        let trimmed = original.trim_start();
        let indentation = leading_spaces(&original);

        check_useless_else(
            trimmed,
            index,
            indentation,
            &mut begin_without_rescue,
            options,
            offenses,
        );
    }
}

fn check_useless_else(
    trimmed: &str,
    index: usize,
    indentation: usize,
    begin_without_rescue: &mut Vec<bool>,
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    if trimmed == "begin" {
        begin_without_rescue.push(true);
    } else if trimmed.starts_with("rescue") {
        if let Some(begin) = begin_without_rescue.last_mut() {
            *begin = false;
        }
    } else if trimmed == "end" {
        begin_without_rescue.pop();
    } else if trimmed == "else"
        && begin_without_rescue.last().copied() == Some(true)
        && options.cop_enabled("Lint/UselessElseWithoutRescue")
        && !options.target_ruby_version.at_least(2, 6)
    {
        push_offense(
            offenses,
            "Lint/UselessElseWithoutRescue",
            "`else` without `rescue` is useless.",
            index + 1,
            indentation + 1,
            4,
            CorrectionStatus::Unavailable,
        );
    }
}
