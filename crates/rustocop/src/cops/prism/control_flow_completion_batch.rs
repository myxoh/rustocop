use super::*;

define_cops! {
    CombinableLoops => "Style/CombinableLoops" => source(combinable_loops),
    EachForSimpleLoop => "Style/EachForSimpleLoop" => source(each_for_simple_loop),
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
