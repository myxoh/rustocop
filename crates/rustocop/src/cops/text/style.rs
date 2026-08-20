use super::helpers::*;
use super::{push_offense, CorrectionStatus, Offense, SourceLine};
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
    check_conditional_assignment(lines, options, offenses);
    check_empty_else(lines, options, offenses);
    check_hash_like_case(lines, options, offenses);
    super::style_declarations::check(lines, options, offenses);
}

fn check_conditional_assignment(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/ConditionalAssignment";
    if !options.cop_enabled(cop) {
        return;
    }

    for index in 0..lines.len().saturating_sub(3) {
        let first = lines[index].body.trim();
        if !(first.starts_with("if ") || first.starts_with("case ")) {
            continue;
        }

        let mut assigned = Vec::new();
        for line in lines.iter().skip(index + 1).take(8) {
            let trimmed = line.body.trim();
            if trimmed == "end" {
                break;
            }
            if let Some(name) = assignment_name(trimmed) {
                assigned.push(name);
            }
        }

        if assigned.len() >= 2 && assigned.iter().all(|name| name == &assigned[0]) {
            push_offense(
                offenses,
                cop,
                "Use the return of the conditional for variable assignment and comparison.",
                index + 1,
                leading_spaces(&lines[index].body) + 1,
                first.len(),
                CorrectionStatus::Unavailable,
            );
        }
    }
}

fn check_empty_else(lines: &[SourceLine], options: &InspectionConfig, offenses: &mut Vec<Offense>) {
    let cop = "Style/EmptyElse";
    if !options.cop_enabled(cop) {
        return;
    }

    for index in 0..lines.len().saturating_sub(1) {
        if lines[index].body.trim() == "else" && lines[index + 1].body.trim() == "end" {
            push_offense(
                offenses,
                cop,
                "Redundant `else`-clause.",
                index + 1,
                leading_spaces(&lines[index].body) + 1,
                4,
                CorrectionStatus::Pending,
            );
        }
    }
}

fn check_hash_like_case(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/HashLikeCase";
    if !options.cop_enabled(cop) {
        return;
    }

    for index in 0..lines.len().saturating_sub(4) {
        if !lines[index].body.trim().starts_with("case ") {
            continue;
        }

        let mut simple_when_count = 0;
        for line in lines.iter().skip(index + 1) {
            let trimmed = line.body.trim();
            if trimmed == "end" {
                break;
            }
            if trimmed.starts_with("when ") && !trimmed.contains(',') {
                simple_when_count += 1;
            }
        }

        if simple_when_count >= 3 {
            push_offense(
                offenses,
                cop,
                "Consider replacing `case` with a hash lookup.",
                index + 1,
                leading_spaces(&lines[index].body) + 1,
                lines[index].body.trim().len(),
                CorrectionStatus::Unavailable,
            );
        }
    }
}
