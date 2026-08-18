use super::helpers::*;
use crate::*;

pub(super) fn before_prism(
    lines: &mut Vec<SourceLine>,
    options: &Options,
    offenses: &mut Vec<Offense>,
) {
    check_frozen_string_literal_comment(lines, options, offenses);
}

pub(super) fn after_prism(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    check_hash_syntax(lines, options, offenses);
    check_keyword_parameters_order(lines, options, offenses);
    check_redundant_begin(lines, options, offenses);
    check_if_unless_modifier(lines, options, offenses);
    check_case_like_if(lines, options, offenses);
    check_conditional_assignment(lines, options, offenses);
    check_empty_case_condition(lines, options, offenses);
    check_empty_else(lines, options, offenses);
    check_guard_clause(lines, options, offenses);
    check_hash_like_case(lines, options, offenses);
    check_class_methods_definitions(lines, options, offenses);
    check_endless_method(lines, options, offenses);
    check_documentation(lines, options, offenses);
    check_trailing_commas(lines, options, offenses);
    check_it_assignment(lines, options, offenses);
    check_numbered_parameters(lines, options, offenses);
    check_string_literals(lines, options, offenses);
}

fn check_frozen_string_literal_comment(
    lines: &mut Vec<SourceLine>,
    options: &Options,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/FrozenStringLiteralComment";
    if !cop_enabled(options, cop) {
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

fn check_hash_syntax(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Style/HashSyntax";
    if !cop_enabled(options, cop) {
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

fn check_keyword_parameters_order(
    lines: &[SourceLine],
    options: &Options,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/KeywordParametersOrder";
    if !cop_enabled(options, cop) {
        return;
    }

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.body.trim();
        if !trimmed.starts_with("def ") || !trimmed.contains('(') {
            continue;
        }

        if optional_keyword_before_required_keyword(trimmed) {
            push_offense(
                offenses,
                cop,
                "Place required keyword parameters before optional keyword parameters.",
                index + 1,
                line.body.find("def").unwrap_or(0) + 1,
                trimmed.len(),
                false,
                false,
            );
        }
    }
}

fn check_redundant_begin(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Style/RedundantBegin";
    if !cop_enabled(options, cop) {
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

fn check_if_unless_modifier(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Style/IfUnlessModifier";
    if !cop_enabled(options, cop) {
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

fn check_case_like_if(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Style/CaseLikeIf";
    if !cop_enabled(options, cop) {
        return;
    }

    let mut chain_start = None;
    let mut comparisons = 0;

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.body.trim();
        if trimmed.starts_with("if ") || trimmed.starts_with("elsif ") {
            if trimmed.contains(" == ") || trimmed.contains(".is_a?") {
                chain_start.get_or_insert(index);
                comparisons += 1;
            }
        } else if trimmed == "end" {
            if comparisons >= 3 {
                let start = chain_start.unwrap_or(index);
                push_offense(
                    offenses,
                    cop,
                    "Convert `if` with multiple branches to `case`.",
                    start + 1,
                    leading_spaces(&lines[start].body) + 1,
                    lines[start].body.trim().len(),
                    false,
                    false,
                );
            }
            chain_start = None;
            comparisons = 0;
        }
    }
}

fn check_conditional_assignment(
    lines: &[SourceLine],
    options: &Options,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/ConditionalAssignment";
    if !cop_enabled(options, cop) {
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

fn check_empty_case_condition(
    lines: &[SourceLine],
    options: &Options,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/EmptyCaseCondition";
    if !cop_enabled(options, cop) {
        return;
    }

    for (index, line) in lines.iter().enumerate() {
        if line.body.trim() == "case" {
            push_offense(
                offenses,
                cop,
                "Do not use empty `case` condition, instead use an `if` expression.",
                index + 1,
                leading_spaces(&line.body) + 1,
                4,
                true,
                false,
            );
        }
    }
}

fn check_empty_else(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Style/EmptyElse";
    if !cop_enabled(options, cop) {
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

fn check_guard_clause(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Style/GuardClause";
    if !cop_enabled(options, cop) {
        return;
    }

    for index in 0..lines.len().saturating_sub(2) {
        let first = lines[index].body.trim();
        let second = lines[index + 1].body.trim();
        let third = lines[index + 2].body.trim();
        if first.starts_with("if ")
            && third == "end"
            && matches!(second, "return" | "break" | "next")
            || second.starts_with("raise ")
        {
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

fn check_hash_like_case(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Style/HashLikeCase";
    if !cop_enabled(options, cop) {
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

fn check_class_methods_definitions(
    lines: &[SourceLine],
    options: &Options,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Style/ClassMethodsDefinitions";
    if !cop_enabled(options, cop) {
        return;
    }

    for (index, line) in lines.iter().enumerate() {
        if line.body.trim() == "class << self" {
            push_offense(
                offenses,
                cop,
                "Do not define public methods within class << self.",
                index + 1,
                leading_spaces(&line.body) + 1,
                line.body.trim().len(),
                true,
                false,
            );
        }
    }
}

fn check_endless_method(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Style/EndlessMethod";
    if !cop_enabled(options, cop) {
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

fn check_documentation(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Style/Documentation";
    if !cop_enabled(options, cop) {
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

fn check_trailing_commas(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    check_collection_trailing_comma(
        lines,
        options,
        offenses,
        "Style/TrailingCommaInArrayLiteral",
        '[',
        ']',
        "Put a comma after the last item of a multiline array.",
    );
    check_collection_trailing_comma(
        lines,
        options,
        offenses,
        "Style/TrailingCommaInHashLiteral",
        '{',
        '}',
        "Put a comma after the last item of a multiline hash.",
    );

    let cop = "Style/TrailingCommaInArguments";
    if !cop_enabled(options, cop) {
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

fn check_collection_trailing_comma(
    lines: &[SourceLine],
    options: &Options,
    offenses: &mut Vec<Offense>,
    cop: &'static str,
    open: char,
    close: char,
    message: &str,
) {
    if !cop_enabled(options, cop) {
        return;
    }

    for index in 1..lines.len() {
        if lines[index].body.trim() != close.to_string() {
            continue;
        }

        let has_open = lines[..index]
            .iter()
            .rev()
            .take(20)
            .any(|line| line.body.contains(open));
        let previous = lines[index - 1].body.trim_end();

        if has_open && !previous.is_empty() && !previous.ends_with(',') {
            push_offense(
                offenses,
                cop,
                message,
                index,
                lines[index - 1].body.chars().count(),
                1,
                true,
                false,
            );
        }
    }
}

fn check_it_assignment(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Style/ItAssignment";
    if !cop_enabled(options, cop) {
        return;
    }

    for (index, line) in lines.iter().enumerate() {
        if line.body.trim_start().starts_with("it =") {
            push_offense(
                offenses,
                cop,
                "Do not use `it` as a local variable.",
                index + 1,
                line.body.find("it").unwrap_or(0) + 1,
                2,
                false,
                false,
            );
        }
    }
}

fn check_numbered_parameters(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Style/NumberedParameters";
    if !cop_enabled(options, cop) {
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

fn check_string_literals(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Style/StringLiterals";
    if !cop_enabled(options, cop) {
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
