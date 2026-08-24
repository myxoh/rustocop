use super::*;

define_cops! {
    CombinableLoops => "Style/CombinableLoops" => source(combinable_loops),
    EachForSimpleLoop => "Style/EachForSimpleLoop" => source(each_for_simple_loop),
}

fn combinable_loops(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let parsed = lines
        .iter()
        .map(|(offset, line)| parse_combinable_loop(*offset, line))
        .collect::<Vec<_>>();
    let mut index = 0;
    while index < parsed.len() {
        let Some(first) = parsed[index].as_ref() else {
            index += 1;
            continue;
        };
        if first.body.trim().is_empty() {
            index += 1;
            continue;
        }

        let mut group_end = index + 1;
        while group_end < parsed.len() {
            let Some(candidate) = parsed[group_end].as_ref() else {
                break;
            };
            if candidate.identity != first.identity || candidate.body.trim().is_empty() {
                break;
            }
            group_end += 1;
        }
        if group_end == index + 1 {
            index += 1;
            continue;
        }

        for candidate_index in index + 1..group_end {
            let candidate = parsed[candidate_index].as_ref().expect("parsed loop");
            let mut parameter_run_start = candidate_index;
            while parameter_run_start > index
                && parsed[parameter_run_start - 1]
                    .as_ref()
                    .is_some_and(|previous| previous.parameters == candidate.parameters)
            {
                parameter_run_start -= 1;
            }
            if parameter_run_start < candidate_index {
                let mut parameter_run_end = candidate_index + 1;
                while parameter_run_end < group_end
                    && parsed[parameter_run_end]
                        .as_ref()
                        .is_some_and(|next| next.parameters == candidate.parameters)
                {
                    parameter_run_end += 1;
                }
                let run_first = parsed[parameter_run_start].as_ref().expect("parsed loop");
                let run_last = parsed[parameter_run_end - 1].as_ref().expect("parsed loop");
                let mut replacement = run_first.before_closing.trim_end().to_string();
                for (merged_index, merged) in parsed[parameter_run_start + 1..parameter_run_end]
                    .iter()
                    .filter_map(Option::as_ref)
                    .enumerate()
                {
                    replacement.push('\n');
                    if merged_index + parameter_run_start + 2 == parameter_run_end {
                        replacement.push_str(merged.body.trim_start());
                    } else {
                        replacement.push_str(merged.body.trim());
                    }
                }
                if run_first.closing_suffix == "}" && !replacement.ends_with(char::is_whitespace) {
                    replacement.push(' ');
                }
                replacement.push_str(run_first.closing_suffix);
                context.replace(
                    "Combine this loop with the previous loop.",
                    candidate.range.clone(),
                    run_first.range.start..run_last.range.end,
                    replacement,
                );
            } else {
                context.report(
                    "Combine this loop with the previous loop.",
                    candidate.range.clone(),
                );
            }
        }
        index = group_end;
    }

    report_multiline_combinable_loops(&lines, context);
}

struct MultilineCombinableLoop {
    identity: String,
    parameters: String,
    start: usize,
    body_start: usize,
    closing_line_start: usize,
    end: usize,
    indent: usize,
    multiline: bool,
}

fn report_multiline_combinable_loops(
    lines: &[(usize, &str)],
    context: &mut CopContext<'_, '_>,
) {
    let mut loops = lines
        .iter()
        .enumerate()
        .filter_map(|(index, _)| parse_multiline_combinable_loop(lines, index))
        .collect::<Vec<_>>();
    loops.extend(lines.iter().filter_map(|(offset, line)| {
        let parsed = parse_combinable_loop(*offset, line)?;
        Some(MultilineCombinableLoop {
            identity: parsed.identity.to_string(),
            parameters: parsed.parameters.to_string(),
            start: parsed.range.start,
            body_start: parsed.range.start,
            closing_line_start: parsed.range.end,
            end: parsed.range.end,
            indent: line.len() - line.trim_start().len(),
            multiline: false,
        })
    }));
    loops.sort_by_key(|candidate| (candidate.start, candidate.end));
    loops.dedup_by_key(|candidate| (candidate.start, candidate.end));
    let source = context.source();
    let mut consumed = vec![false; loops.len()];
    for index in 0..loops.len() {
        if consumed[index] {
            continue;
        }
        let first = &loops[index];
        let mut group = vec![index];
        let mut current = index;
        while let Some(next) = (current + 1..loops.len()).find(|candidate| {
                loops[*candidate].indent == first.indent
                    && loops[*candidate].start >= loops[current].end
            }) {
            let previous = &loops[current];
            let candidate = &loops[next];
            if candidate.identity != first.identity
                || !only_trivia(&source[previous.end..candidate.start])
            {
                break;
            }
            group.push(next);
            current = next;
        }
        if group.len() == 1 {
            continue;
        }
        if !group.iter().any(|candidate| loops[*candidate].multiline) {
            continue;
        }
        for member in &group {
            consumed[*member] = true;
        }

        let same_parameters = group
            .iter()
            .all(|candidate| loops[*candidate].parameters == first.parameters);
        let replacement = same_parameters
            .then(|| {
                group
                    .iter()
                    .all(|candidate| loops[*candidate].multiline)
                    .then(|| {
            let last = &loops[*group.last().expect("loop group")];
            let mut combined = source[first.start..first.closing_line_start].to_string();
            for candidate in group.iter().skip(1).map(|candidate| &loops[*candidate]) {
                combined.push_str(&source[candidate.body_start..candidate.closing_line_start]);
            }
            combined.push_str(&source[first.closing_line_start..first.end]);
            (first.start..last.end, combined)
                    })
            })
            .flatten();

        for candidate in group.iter().skip(1).map(|candidate| &loops[*candidate]) {
            let offense = candidate.start..candidate.end;
            if let Some((edit, combined)) = replacement.as_ref() {
                context.replace(
                    "Combine this loop with the previous loop.",
                    offense,
                    edit.clone(),
                    combined.clone(),
                );
            } else if same_parameters {
                context.replace(
                    "Combine this loop with the previous loop.",
                    offense.clone(),
                    offense.clone(),
                    source[offense].to_string(),
                );
            } else {
                context.report("Combine this loop with the previous loop.", offense);
            }
        }
    }
}

fn parse_multiline_combinable_loop(
    lines: &[(usize, &str)],
    index: usize,
) -> Option<MultilineCombinableLoop> {
    let (offset, line) = lines[index];
    let source = line.trim_start();
    if source.starts_with('#') {
        return None;
    }
    let indent = line.len() - source.len();
    let opening = source.rfind(" do")?;
    if source[opening + 3..].contains(" end") {
        return None;
    }
    let call = source[..opening].trim_end();
    let method = call
        .rsplit_once('.')
        .map_or(call, |(_, method)| method)
        .split(['(', ' '])
        .next()?;
    if !method.starts_with("each") && !method.ends_with("_each") {
        return None;
    }
    let parameters = source[opening + 3..].trim().to_string();
    let closing_index = (index + 1..lines.len()).find(|candidate| {
        let candidate_line = lines[*candidate].1;
        let trimmed = candidate_line.trim_start();
        candidate_line.len() - trimmed.len() == indent
            && (trimmed == "end" || trimmed.starts_with("end "))
    })?;
    if closing_index == index + 1 {
        return None;
    }
    let closing_line = lines[closing_index].1;
    let closing_line_start = lines[closing_index].0;
    Some(MultilineCombinableLoop {
        identity: call.to_string(),
        parameters,
        start: offset + indent,
        body_start: lines[index + 1].0,
        closing_line_start,
        end: closing_line_start + closing_line.len() - closing_line.trim_start().len() + 3,
        indent,
        multiline: true,
    })
}

fn only_trivia(source: &str) -> bool {
    source.lines().all(|line| {
        let line = line.trim();
        line.is_empty() || line.starts_with('#') || line == "end"
    })
}

struct CombinableLoop<'source> {
    identity: &'source str,
    parameters: &'source str,
    body: &'source str,
    before_closing: &'source str,
    closing_suffix: &'source str,
    range: std::ops::Range<usize>,
}

fn parse_combinable_loop(offset: usize, line: &str) -> Option<CombinableLoop<'_>> {
    let indent = line.len() - line.trim_start().len();
    let source = line.trim_start();
    if source.starts_with('#') {
        return None;
    }
    let (identity_end, body_start, closing_start, closing_suffix) = if source.starts_with("for ") {
        let opening = source.find(" do ")?;
        let closing = source.rfind(" end")?;
        (opening, opening + " do".len(), closing, &source[closing..])
    } else {
        let method_end = [".each_with_index", ".reverse_each", ".each"]
            .into_iter()
            .find_map(|method| source.find(method).map(|start| start + method.len()))?;
        let remainder = &source[method_end..];
        if let Some(opening) = remainder.find('{') {
            let opening = method_end + opening;
            let closing = source.rfind('}')?;
            (opening, opening + 1, closing, &source[closing..])
        } else {
            let opening = remainder.find(" do")? + method_end;
            let closing = source.rfind(" end")?;
            (opening, opening + " do".len(), closing, &source[closing..])
        }
    };
    if body_start > closing_start {
        return None;
    }
    let mut content_start = body_start;
    while content_start < closing_start && source.as_bytes().get(content_start) == Some(&b' ') {
        content_start += 1;
    }
    let (parameters, body_start) = if source.as_bytes().get(content_start) == Some(&b'|') {
        let parameter_end = source[content_start + 1..].find('|')? + content_start + 2;
        if parameter_end > closing_start {
            return None;
        }
        (&source[content_start..parameter_end], parameter_end)
    } else {
        ("", body_start)
    };
    if body_start > closing_start {
        return None;
    }
    let body = &source[body_start..closing_start];
    Some(CombinableLoop {
        identity: source[..identity_end].trim_end(),
        parameters,
        body,
        before_closing: &source[..closing_start],
        closing_suffix,
        range: offset + indent..offset + line.len(),
    })
}

fn each_for_simple_loop(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let Some(each) = line.find(".each") else { continue };
        if line
            .as_bytes()
            .get(each + ".each".len())
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            continue;
        }
        let safe_navigation = line.as_bytes().get(each.saturating_sub(1)) == Some(&b'&');
        let receiver_end = each - usize::from(safe_navigation);
        let Some(open) = line[..receiver_end].rfind('(') else { continue };
        if line[..open]
            .trim_end()
            .as_bytes()
            .last()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b')' | b']'))
        {
            continue;
        }
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
