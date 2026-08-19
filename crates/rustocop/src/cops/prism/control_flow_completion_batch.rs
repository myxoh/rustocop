use super::*;

define_cops! {
    RedundantConditional => "Style/RedundantConditional" => source(redundant_conditional),
    InvertibleUnlessCondition => "Style/InvertibleUnlessCondition" => source(invertible_unless),
    CombinableLoops => "Style/CombinableLoops" => source(combinable_loops),
    EachForSimpleLoop => "Style/EachForSimpleLoop" => source(each_for_simple_loop),
    RescueModifier => "Style/RescueModifier" => source(rescue_modifier),
    RedundantSelfAssignmentBranch => "Style/RedundantSelfAssignmentBranch" => source(redundant_self_branch),
}

fn redundant_conditional(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(5) {
        let (start, header) = window[0];
        let header = header.trim_start();
        let keyword = if header.starts_with("if ") {
            "if"
        } else if header.starts_with("unless ") {
            "unless"
        } else {
            continue;
        };
        if window[2].1.trim() != "else" || window[4].1.trim() != "end" {
            continue;
        }
        let (truthy, falsey) = (window[1].1.trim(), window[3].1.trim());
        if !matches!((truthy, falsey), ("true", "false") | ("false", "true")) {
            continue;
        }
        let condition = header.trim_start_matches(keyword).trim();
        let direct = (keyword == "if") == (truthy == "true");
        let replacement = if direct {
            condition.to_string()
        } else {
            format!("!({condition})")
        };
        let end = window[4].0 + window[4].1.len();
        context.replace(
            format!("This conditional expression can just be replaced by `{replacement}`."),
            start..end,
            start..end,
            replacement,
        );
    }
    for (offset, line) in context.source_file().lines() {
        let code = line.trim();
        let (suffix, negated) = if code.ends_with(" ? true : false") {
            (" ? true : false", false)
        } else if code.ends_with(" ? false : true") {
            (" ? false : true", true)
        } else {
            continue;
        };
        let condition = code.trim_end_matches(suffix);
        let replacement = if negated {
            format!("!({condition})")
        } else {
            condition.to_string()
        };
        let start = offset + line.find(code).unwrap_or(0);
        context.replace(
            format!("This conditional expression can just be replaced by `{replacement}`."),
            start..start + code.len(),
            start..start + code.len(),
            replacement,
        );
    }
}

fn invertible_unless(context: &mut CopContext<'_, '_>) {
    let pairs = [
        ("odd?", "even?"),
        ("include?", "exclude?"),
        ("any?", "none?"),
        ("empty?", "any?"),
    ];
    for (offset, line) in context.source_file().lines() {
        let Some(unless_at) = line.find(" unless ") else {
            continue;
        };
        let condition = line[unless_at + 8..].trim();
        if condition.starts_with("begin ")
            || condition.contains(" && ")
            || condition.contains(" || ")
        {
            continue;
        }
        if let Some(stripped) = condition.strip_prefix('!') {
            let inverted = if let Some(double_stripped) = condition.strip_prefix("!!") {
                format!("!{double_stripped}")
            } else {
                stripped.to_string()
            };
            let replacement = format!("{} if {inverted}", line[..unless_at].trim_end());
            let code = line.trim();
            let start = offset + line.find(code).unwrap_or(0);
            context.replace(
                format!("Prefer `if {inverted}` over `unless {condition}`."),
                start..start + code.len(),
                start..start + code.len(),
                replacement,
            );
            continue;
        }
        if let Some(inverted) = invert_operator(condition) {
            let replacement = format!("{} if {inverted}", line[..unless_at].trim_end());
            let code = line.trim();
            let start = offset + line.find(code).unwrap_or(0);
            context.replace(
                format!("Prefer `if {inverted}` over `unless {condition}`."),
                start..start + code.len(),
                start..start + code.len(),
                replacement,
            );
            continue;
        }
        let Some((from, to)) = pairs
            .into_iter()
            .find(|(from, _)| condition.starts_with(from))
        else {
            continue;
        };
        let inverted = condition.replacen(from, to, 1);
        let replacement = format!("{} if {inverted}", line[..unless_at].trim_end());
        let code = line.trim();
        let start = offset + line.find(code).unwrap_or(0);
        context.replace(
            format!("Prefer `if {inverted}` over `unless {condition}`."),
            start..start + code.len(),
            start..start + code.len(),
            replacement,
        );
    }
}

fn invert_operator(condition: &str) -> Option<String> {
    for (from, to) in [
        (" != ", " == "),
        (" == ", " != "),
        (" >= ", " < "),
        (" <= ", " > "),
        (" > ", " <= "),
        (" < ", " >= "),
    ] {
        if condition.contains(from) {
            return Some(condition.replacen(from, to, 1));
        }
    }
    None
}

fn combinable_loops(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut previous: Option<(String, usize)> = None;
    for (offset, line) in lines {
        let trimmed = line.trim_start();
        let Some(dot) = trimmed.find('.') else {
            previous = None;
            continue;
        };
        let receiver = &trimmed[..dot];
        let method = trimmed[dot + 1..]
            .split([' ', '{'])
            .next()
            .unwrap_or_default();
        if !["each", "each_with_index", "reverse_each"].contains(&method) {
            previous = None;
            continue;
        }
        let identity = format!("{receiver}.{method}");
        if previous.as_ref().is_some_and(|(seen, _)| seen == &identity) {
            let indent = line.len() - trimmed.len();
            context.report(
                "Combine this loop with the previous loop.",
                offset + indent..offset + line.len(),
            );
        }
        previous = Some((identity, offset));
    }
}

fn each_for_simple_loop(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let mut search = 0;
    while let Some(relative) = source[search..].find(").each") {
        let each = search + relative;
        let Some(open) = source[..each].rfind("(0...") else {
            break;
        };
        let number = &source[open + 5..each];
        if !number.bytes().all(|byte| byte.is_ascii_digit()) {
            search = each + 6;
            continue;
        }
        let offense_end = each + ").each".len();
        context.replace(
            "Use `Integer#times` for a simple loop which iterates a fixed number of times.",
            open..offense_end,
            open..offense_end,
            format!("{number}.times"),
        );
        search = offense_end;
    }
}

fn rescue_modifier(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let Some(at) = line.find(" rescue ") else {
            continue;
        };
        let code = line.trim();
        let indent = line.len() - line.trim_start().len();
        let body = line[indent..at].trim();
        let handler = line[at + 8..].trim();
        context.replace(
            "Avoid using `rescue` in its modifier form.",
            offset + indent..offset + line.len(),
            offset + indent..offset + line.len(),
            format!("begin\n  {body}\nrescue\n  {handler}\nend"),
        );
        let _ = code;
    }
}

fn redundant_self_branch(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let Some((left, rest)) = line.trim().split_once(" = ") else {
            continue;
        };
        let Some((condition, branches)) = rest.split_once(" ? ") else {
            continue;
        };
        let Some((truthy, falsey)) = branches.split_once(" : ") else {
            continue;
        };
        let (branch, replacement) = if falsey.trim() == left {
            (falsey.trim(), format!("{left} = {truthy} if {condition}"))
        } else if truthy.trim() == left {
            (
                truthy.trim(),
                format!("{left} = {falsey} unless {condition}"),
            )
        } else {
            continue;
        };
        let start = offset + line.rfind(branch).unwrap_or(0);
        context.replace(
            "Remove the self-assignment branch.",
            start..start + branch.len(),
            offset + line.find(left).unwrap_or(0)..offset + line.len(),
            replacement,
        );
    }
}
