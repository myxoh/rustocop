use super::*;

define_cops! {
    InvertibleUnlessCondition => "Style/InvertibleUnlessCondition" => source(invertible_unless),
    CombinableLoops => "Style/CombinableLoops" => source(combinable_loops),
    EachForSimpleLoop => "Style/EachForSimpleLoop" => source(each_for_simple_loop),
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
