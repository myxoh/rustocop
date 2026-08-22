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

    let style = options
        .cop_config
        .value(cop, "EnforcedStyle")
        .unwrap_or("empty");
    let allow_comments = options
        .cop_config
        .bool(cop, "AllowComments")
        .unwrap_or(false);
    let missing_else = options
        .cop_config
        .value("Style/MissingElse", "EnforcedStyle")
        .unwrap_or_default();
    let missing_else_enabled = options
        .cop_config
        .bool("Style/MissingElse", "Enabled")
        .unwrap_or(true);
    for (index, line) in lines.iter().enumerate() {
        for (column, _) in line.body.match_indices("else") {
            let before = line.body.as_bytes().get(column.wrapping_sub(1));
            let after_keyword = line.body.as_bytes().get(column + 4);
            if before.is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                || after_keyword.is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            {
                continue;
            }
            let tail = line.body[column + 4..]
                .trim()
                .trim_start_matches(';')
                .trim();
            let (empty, nil_only, has_comment) = if !tail.is_empty() {
                (
                    tail == "end",
                    tail == "nil" || tail.starts_with("nil;") || tail.starts_with("nil "),
                    tail.starts_with('#'),
                )
            } else {
                let mut cursor = index + 1;
                let mut comment = false;
                while cursor < lines.len() && lines[cursor].body.trim_start().starts_with('#') {
                    comment = true;
                    cursor += 1;
                }
                let next = lines
                    .get(cursor)
                    .map(|next| next.body.trim())
                    .unwrap_or_default();
                let following = lines
                    .get(cursor + 1)
                    .map(|next| next.body.trim())
                    .unwrap_or_default();
                (next == "end", next == "nil" && following == "end", comment)
            };
            let redundant = match style {
                "nil" => nil_only,
                "both" => empty || nil_only,
                _ => empty,
            };
            if !redundant || allow_comments && has_comment {
                continue;
            }
            let kind = enclosing_conditional_kind(lines, index, column);
            let conflicts =
                missing_else_enabled && (missing_else == "both" || missing_else == kind);
            push_offense(
                offenses,
                cop,
                "Redundant `else`-clause.",
                index + 1,
                column + 1,
                4,
                if conflicts || has_comment {
                    CorrectionStatus::Unavailable
                } else {
                    CorrectionStatus::Pending
                },
            );
        }
    }
}

fn enclosing_conditional_kind(lines: &[SourceLine], index: usize, column: usize) -> &'static str {
    let current = &lines[index].body[..column];
    if current
        .rfind("case ")
        .is_some_and(|case_at| current.rfind("if ").is_none_or(|if_at| case_at > if_at))
    {
        return "case";
    }
    for line in lines[..=index].iter().rev() {
        let trimmed = line.body.trim_start();
        if trimmed.starts_with("case ") || trimmed.contains("= case ") {
            return "case";
        }
        if trimmed.starts_with("if ")
            || trimmed.starts_with("unless ")
            || trimmed.contains("= if ")
            || trimmed.contains("= unless ")
        {
            return "if";
        }
    }
    "if"
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
