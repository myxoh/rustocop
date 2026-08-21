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
    for (offset, line) in context.source_file().lines() {
        let Some(each) = line.find(".each") else { continue };
        let safe_navigation = line.as_bytes().get(each.saturating_sub(1)) == Some(&b'&');
        let receiver_end = each - usize::from(safe_navigation);
        let Some(open) = line[..receiver_end].rfind('(') else { continue };
        if line.as_bytes().get(receiver_end.saturating_sub(1)) != Some(&b')') { continue; }
        let range = &line[open + 1..receiver_end - 1];
        let (start, end, inclusive) = if let Some((start, end)) = range.split_once("...") {
            (start, end, false)
        } else if let Some((start, end)) = range.split_once("..") {
            (start, end, true)
        } else {
            continue;
        };
        let (Ok(start_number), Ok(end_number)) = (start.parse::<usize>(), end.parse::<usize>()) else { continue };
        let offense_end = each + ".each".len();
        let block_header = line[offense_end..].trim_start();
        if block_header.starts_with('{') || block_header.starts_with("do") {
            let delimiter = if block_header.starts_with('{') { '}' } else { '\n' };
            let header = block_header.split(delimiter).next().unwrap_or(block_header);
            if header.contains('|') { continue; }
        }
        let iterations = end_number.saturating_sub(start_number) + usize::from(inclusive);
        context.replace(
            "Use `Integer#times` for a simple loop which iterates a fixed number of times.",
            offset + open..offset + offense_end,
            offset + open..offset + offense_end,
            format!("{iterations}.times"),
        );
    }
}
