use super::catalog_cop::{custom, replace, report};
use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        custom("Style/ParenthesesAroundCondition", parentheses_condition),
        replace(
            "Style/RescueStandardError",
            "rescue Exception",
            "rescue StandardError",
            "Avoid rescuing the `Exception` class.",
        ),
        custom("Lint/UnreachableLoop", unreachable_loop),
        custom("Style/IdenticalConditionalBranches", identical_branches),
        custom("Style/NegatedIfElseCondition", negated_if_else),
        custom("Style/InfiniteLoop", infinite_loop),
        report(
            "Style/MapCompactWithConditionalBlock",
            ".map { |x| if ",
            "Use `filter_map` instead of `map` followed by `compact`.",
        ),
        custom("Style/YodaCondition", yoda_condition),
        custom("Lint/EmptyConditionalBody", empty_conditional),
        custom("Style/RedundantReturn", redundant_return),
        report(
            "Lint/NoReturnInBeginEndBlocks",
            "begin\n  return",
            "Do not return from an explicit `begin` block.",
        ),
        report(
            "Lint/RescueType",
            "rescue '",
            "Rescue an exception class rather than a string literal.",
        ),
        custom("Lint/DuplicateBranch", identical_branches),
        replace(
            "Style/RedundantCondition",
            "condition ? true : false",
            "condition",
            "Use the condition directly.",
        ),
        report(
            "Style/SoleNestedConditional",
            "else\n  if ",
            "Consider merging nested conditions.",
        ),
        report(
            "Style/IfWithBooleanLiteralBranches",
            "if predicate?\n  true",
            "Use a boolean expression instead of an if with boolean branches.",
        ),
        custom("Style/OneLineConditional", one_line_conditional),
        custom("Lint/UnreachableCode", unreachable_code),
        custom("Lint/LiteralAsCondition", literal_condition),
    ]
}

fn negated_if_else(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    if !lines.iter().any(|(_, line)| line.trim() == "else")
        || lines
            .iter()
            .any(|(_, line)| line.trim_start().starts_with("elsif "))
    {
        return;
    }
    for (offset, line) in lines {
        let condition = line.trim_start();
        if condition.starts_with("if !")
            && !condition.starts_with("if !!")
            && !condition.contains(" && ")
            && !condition.contains(" || ")
        {
            context.report(
                "Invert the negated condition and swap the branches.",
                offset..offset + line.len(),
            );
        }
    }
}

fn unreachable_loop(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if ["while ", "until ", "for "]
            .iter()
            .any(|keyword| window[0].1.trim_start().starts_with(keyword))
            && matches!(window[1].1.trim(), "break" | "return" | "raise")
        {
            context.report(
                "This loop will have at most one iteration.",
                window[0].0..window[1].0 + window[1].1.len(),
            );
        }
    }
}

fn parentheses_condition(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        if ["if (", "unless (", "while (", "until ("]
            .iter()
            .any(|prefix| trimmed.starts_with(prefix))
            && trimmed.find(')').is_some_and(|close| {
                trimmed[close + 1..].trim().is_empty()
                    || trimmed[close + 1..].trim_start().starts_with("then")
            })
        {
            let open = trimmed.find('(').unwrap_or(0);
            let close = trimmed.find(')').unwrap_or(trimmed.len());
            let body = &trimmed[open + 1..close];
            if body.is_empty()
                || body.contains(';')
                || body.contains(" = ")
                || [" rescue ", " if ", " unless ", " while ", " until "]
                    .iter()
                    .any(|keyword| body.contains(keyword))
            {
                continue;
            }
            let at = offset + line.find('(').unwrap_or(0);
            context.report(
                "Don't use parentheses around the condition of a conditional.",
                at..offset + line.find(')').unwrap_or(line.len()) + 1,
            );
        }
    }
}

fn one_line_conditional(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        if (trimmed.starts_with("if ") || trimmed.starts_with("unless "))
            && trimmed.contains(" then ")
            && trimmed.ends_with(" end")
            && trimmed
                .split_once(" else ")
                .is_some_and(|(_, branch)| branch != "end")
            && !trimmed.split_once(" then ").is_some_and(|(_, body)| {
                body.split_once(" else ")
                    .unwrap_or((body, ""))
                    .0
                    .contains(';')
            })
        {
            context.report(
                "Favor a normal conditional over a one-line conditional.",
                offset..offset + line.len(),
            );
        }
    }
}

fn identical_branches(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(5) {
        if window[0].1.trim_start().starts_with("if ")
            && window[2].1.trim() == "else"
            && window[4].1.trim() == "end"
            && window[1].1.trim() == window[3].1.trim()
            && window[1].1.trim() != "()"
        {
            context.report(
                "Move identical branch contents out of the conditional.",
                window[0].0..window[4].0 + window[4].1.len(),
            );
        }
    }
}

fn yoda_condition(context: &mut CopContext<'_, '_>) {
    let style = context
        .policy()
        .enforced_style("forbid_for_all_comparison_operators")
        .to_string();
    if style.starts_with("require_") {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        for operator in [" == ", " != ", " < ", " > "] {
            let Some(at) = line.find(operator) else {
                continue;
            };
            let left = line[..at].split_whitespace().last().unwrap_or("");
            let right = line[at + operator.len()..]
                .split_whitespace()
                .next()
                .unwrap_or("");
            if style == "forbid_for_equality_operators_only" && !matches!(operator, " == " | " != ")
            {
                continue;
            }
            if (left.starts_with(['\'', '"']) || left.bytes().all(|byte| byte.is_ascii_digit()))
                && !left.contains("#{")
                && right
                    .bytes()
                    .next()
                    .is_some_and(|byte| byte.is_ascii_lowercase())
            {
                context.report(
                    "Reverse the order of the operands in this comparison.",
                    offset + line.find(left).unwrap_or(0)
                        ..offset + at + operator.len() + right.len(),
                );
            }
        }
    }
}

fn empty_conditional(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if ["if ", "unless ", "while ", "until "]
            .iter()
            .any(|keyword| window[0].1.trim_start().starts_with(keyword))
            && window[1].1.trim() == "end"
        {
            context.report(
                "Avoid empty conditional bodies.",
                window[0].0..window[1].0 + window[1].1.len(),
            );
        }
    }
}

fn redundant_return(context: &mut CopContext<'_, '_>) {
    if context.source().contains("proc do") || context.source().contains("lambda do") {
        return;
    }
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if let Some(value) = window[0].1.trim_start().strip_prefix("return ") {
            if context.config_bool("AllowMultipleReturnValues", false) && value.contains(',') {
                continue;
            }
            if window[1].1.trim() == "end" {
                let start = window[0].0 + window[0].1.find("return ").unwrap_or(0);
                context.replace(
                    "Redundant `return` detected.",
                    start..start + 7,
                    start..start + 7,
                    "",
                );
                let _ = value;
            }
        }
    }
}

fn infinite_loop(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    if source
        .lines()
        .any(|line| line.contains("while true") && line.trim_start() != "while true")
        || source
            .lines()
            .any(|line| line.contains(",") && line.contains(" = "))
    {
        return;
    }
    for start in context.source_file().code_offsets("while true") {
        context.replace(
            "Use `Kernel#loop` for infinite loops.",
            start..start + 10,
            start..start + 10,
            "loop do",
        );
    }
}

fn unreachable_code(context: &mut CopContext<'_, '_>) {
    let redefines_raise = context.source().lines().any(|line| {
        let line = line.trim_start();
        line.starts_with("def raise")
            || line.starts_with("def self.raise")
            || line.starts_with("def fail")
            || line.starts_with("def self.fail")
    });
    let dynamic_receiver = context.source().contains("instance_eval");
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        let terminator = window[0].1.trim();
        if matches!(terminator, "raise" | "fail") && (redefines_raise || dynamic_receiver) {
            continue;
        }
        if matches!(terminator, "return" | "break" | "next" | "raise" | "fail")
            && !window[1].1.trim().is_empty()
            && !matches!(window[1].1.trim(), "end" | "else" | "ensure" | "rescue")
            && window[0].1.len() - window[0].1.trim_start().len()
                == window[1].1.len() - window[1].1.trim_start().len()
        {
            context.report(
                "Unreachable code detected.",
                window[1].0..window[1].0 + window[1].1.len(),
            );
        }
    }
}

fn literal_condition(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        for condition in [
            "if true",
            "if false",
            "if nil",
            "unless true",
            "unless false",
        ] {
            if let Some(at) = line.find(condition) {
                context.report(
                    "Literal used as a condition.",
                    offset + at..offset + at + condition.len(),
                );
            }
        }
    }
}
