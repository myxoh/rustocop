use super::helpers::*;
use super::{push_offense, Offense, SourceLine};
use crate::config::InspectionConfig;

pub(super) fn before_prism(
    lines: &mut Vec<SourceLine>,
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    check_frozen_string_literal_comment(lines, options, offenses);
}

pub(super) fn after_prism(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    check_hash_syntax(lines, options, offenses);
    check_redundant_begin(lines, options, offenses);
    check_if_unless_modifier(lines, options, offenses);
    check_conditional_assignment(lines, options, offenses);
    check_empty_else(lines, options, offenses);
    check_guard_clause(lines, options, offenses);
    check_hash_like_case(lines, options, offenses);
    super::style_declarations::check(lines, options, offenses);
}

fn check_frozen_string_literal_comment(
    lines: &mut Vec<SourceLine>,
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/FrozenStringLiteralComment";
    if !options.cop_enabled(cop) {
        return;
    }

    let mut index = 0;
    while index < lines.len().min(3) {
        if lines[index].body.contains("frozen_string_literal:") {
            let corrected = options.autocorrect;
            push_offense(
                offenses,
                cop,
                "Missing frozen string literal comment.",
                index + 1,
                1,
                lines[index].body.chars().count().max(1),
                true,
                corrected,
            );

            if corrected {
                lines.remove(index);
                continue;
            }
        }

        index += 1;
    }
}

fn check_hash_syntax(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/HashSyntax";
    if !options.cop_enabled(cop) {
        return;
    }

    for (index, line) in lines.iter().enumerate() {
        if let Some(column) = line.body.find("=>").map(|position| position + 1) {
            push_offense(
                offenses,
                cop,
                "Use the new Ruby 1.9 hash syntax.",
                index + 1,
                column,
                2,
                true,
                false,
            );
        }
    }
}

fn check_redundant_begin(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/RedundantBegin";
    if !options.cop_enabled(cop) {
        return;
    }

    for (index, line) in lines.iter().enumerate() {
        if line.body.trim() == "begin" {
            push_offense(
                offenses,
                cop,
                "Redundant `begin` block detected.",
                index + 1,
                leading_spaces(&line.body) + 1,
                5,
                false,
                false,
            );
        }
    }
}

fn check_if_unless_modifier(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/IfUnlessModifier";
    if !options.cop_enabled(cop) {
        return;
    }

    for index in 0..lines.len().saturating_sub(2) {
        let first = lines[index].body.trim();
        let second = lines[index + 1].body.trim();
        let third = lines[index + 2].body.trim();

        if (first.starts_with("if ") || first.starts_with("unless "))
            && !second.is_empty()
            && third == "end"
        {
            push_offense(
                offenses,
                cop,
                "Favor modifier if/unless usage when you have a single-line body.",
                index + 1,
                leading_spaces(&lines[index].body) + 1,
                first.len(),
                false,
                false,
            );
        }
    }
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
                false,
                false,
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
                true,
                false,
            );
        }
    }
}

fn check_guard_clause(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/GuardClause";
    if !options.cop_enabled(cop) {
        return;
    }

    for index in 0..lines.len().saturating_sub(2) {
        let first = lines[index].body.trim();
        let second = lines[index + 1].body.trim();
        let third = lines[index + 2].body.trim();
        let exits_scope =
            matches!(second, "return" | "break" | "next") || second.starts_with("raise ");
        if first.starts_with("if ") && third == "end" && exits_scope {
            push_offense(
                offenses,
                cop,
                "Use a guard clause instead of wrapping the code inside a conditional expression.",
                index + 1,
                leading_spaces(&lines[index].body) + 1,
                first.len(),
                false,
                false,
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
                false,
                false,
            );
        }
    }
}
