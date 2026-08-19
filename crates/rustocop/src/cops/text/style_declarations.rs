use super::helpers::*;
use super::{push_offense, Offense, SourceLine};
use crate::config::InspectionConfig;

pub(super) fn check(lines: &[SourceLine], options: &InspectionConfig, offenses: &mut Vec<Offense>) {
    check_endless_method(lines, options, offenses);
    check_documentation(lines, options, offenses);
    check_trailing_commas(lines, options, offenses);
    check_numbered_parameters(lines, options, offenses);
    check_string_literals(lines, options, offenses);
}

fn check_endless_method(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/EndlessMethod";
    if !options.cop_enabled(cop) {
        return;
    }
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.body.trim();
        if trimmed.starts_with("def ") && trimmed.contains(" = ") {
            push_offense(
                offenses,
                cop,
                "Avoid endless method definitions.",
                index + 1,
                leading_spaces(&line.body) + 1,
                trimmed.len(),
                false,
                false,
            );
        }
    }
}

fn check_documentation(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/Documentation";
    if !options.cop_enabled(cop) {
        return;
    }
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.body.trim();
        if !(trimmed.starts_with("class ") || trimmed.starts_with("module ")) {
            continue;
        }
        let documented = lines[..index]
            .iter()
            .rev()
            .find(|previous| !previous.body.trim().is_empty())
            .is_some_and(|previous| previous.body.trim_start().starts_with('#'));
        if !documented {
            push_offense(
                offenses,
                cop,
                "Missing top-level documentation comment.",
                index + 1,
                leading_spaces(&line.body) + 1,
                trimmed.len(),
                false,
                false,
            );
        }
    }
}

fn check_trailing_commas(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/TrailingCommaInArguments";
    if !options.cop_enabled(cop) {
        return;
    }
    for index in 1..lines.len() {
        if lines[index].body.trim() == ")" && lines[index - 1].body.trim_end().ends_with(',') {
            push_offense(
                offenses,
                cop,
                "Avoid comma after the last parameter of a method call.",
                index,
                lines[index - 1].body.chars().count(),
                1,
                true,
                false,
            );
        }
    }
}

fn check_numbered_parameters(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/NumberedParameters";
    if !options.cop_enabled(cop) {
        return;
    }
    for (index, line) in lines.iter().enumerate() {
        if let Some(column) = find_numbered_parameter(&line.body) {
            push_offense(
                offenses,
                cop,
                "Avoid using numbered parameters.",
                index + 1,
                column,
                2,
                false,
                false,
            );
        }
    }
}

fn check_string_literals(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/StringLiterals";
    if !options.cop_enabled(cop) {
        return;
    }
    for (index, line) in lines.iter().enumerate() {
        if let Some(column) = find_single_quoted_literal(&line.body) {
            push_offense(
                offenses,
                cop,
                "Prefer double-quoted strings.",
                index + 1,
                column,
                1,
                true,
                false,
            );
        }
    }
}
