use super::catalog_cop::custom;
use super::*;

mod registry;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    let mut cops = vec![
        custom("Lint/UnreachableLoop", unreachable_loop),
        custom("Lint/EmptyConditionalBody", empty_conditional),
    ];
    cops.extend(registry::cops());
    cops
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

fn identical_branches(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(5) {
        if window[0].1.trim_start().starts_with("if ")
            && window[2].1.trim() == "else"
            && window[4].1.trim() == "end"
            && window[1].1.trim() == window[3].1.trim()
        {
            context.report("Duplicate branch body detected.", window[1].0..window[3].0 + window[3].1.len());
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
