use super::catalog_cop::{custom, report};
use super::*;

mod registry;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    let mut cops = vec![
        custom("Style/ParenthesesAroundCondition", parentheses_condition),
        custom("Lint/UnreachableLoop", unreachable_loop),
        custom("Style/IdenticalConditionalBranches", identical_branches),
        custom("Style/NegatedIfElseCondition", negated_if_else),
        custom("Style/InfiniteLoop", infinite_loop),
        report(
            "Style/MapCompactWithConditionalBlock",
            ".map { |x| if ",
            "Use `filter_map` instead of `map` followed by `compact`.",
        ),
        custom("Lint/EmptyConditionalBody", empty_conditional),
    ];
    cops.extend(registry::cops());
    cops
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
