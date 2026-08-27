use super::*;

define_cops! {
    CombinableLoops => "Style/CombinableLoops" => compatibility_source(combinable_loops),
}

fn combinable_loops(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut non_code_ranges = context.heredoc_ranges();
    non_code_ranges.extend(context.comment_ranges());
    let parsed = lines
        .iter()
        .map(|(offset, line)| {
            let code_start = *offset + line.len() - line.trim_start().len();
            (!non_code_ranges
                .iter()
                .any(|range| range.start <= code_start && code_start < range.end))
            .then(|| parse_combinable_loop(*offset, line))
            .flatten()
        })
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

        let mut group = vec![index];
        let mut cursor = index + 1;
        while cursor < parsed.len() {
            let Some(candidate_index) = (cursor..parsed.len()).find(|candidate| {
                parsed[*candidate].is_some()
                    || !only_trivia(lines[*candidate].1)
            }) else {
                break;
            };
            let Some(candidate) = parsed[candidate_index].as_ref() else { break };
            if candidate.identity != first.identity || candidate.body.trim().is_empty() {
                break;
            }
            group.push(candidate_index);
            cursor = candidate_index + 1;
        }
        if group.len() == 1 {
            index += 1;
            continue;
        }

        for (group_position, candidate_index) in group.iter().copied().enumerate().skip(1) {
            let candidate = parsed[candidate_index].as_ref().expect("parsed loop");
            let mut parameter_run_start = group_position;
            while parameter_run_start > 0
                && parsed[group[parameter_run_start - 1]]
                    .as_ref()
                    .is_some_and(|previous| previous.parameters == candidate.parameters)
            {
                parameter_run_start -= 1;
            }
            if parameter_run_start < group_position {
                let mut parameter_run_end = group_position + 1;
                while parameter_run_end < group.len()
                    && parsed[group[parameter_run_end]]
                        .as_ref()
                        .is_some_and(|next| next.parameters == candidate.parameters)
                {
                    parameter_run_end += 1;
                }
                let run_first = parsed[group[parameter_run_start]].as_ref().expect("parsed loop");
                let run_last = parsed[group[parameter_run_end - 1]].as_ref().expect("parsed loop");
                let mut replacement = run_first.before_closing.trim_end().to_string();
                for (merged_index, merged) in group[parameter_run_start + 1..parameter_run_end]
                    .iter()
                    .filter_map(|member| parsed[*member].as_ref())
                    .enumerate()
                {
                    let previous = parsed[group[parameter_run_start + merged_index]]
                        .as_ref()
                        .expect("previous loop");
                    let trivia = &context.source()[previous.range.end..merged.range.start];
                    if trivia.is_empty() {
                        replacement.push('\n');
                    } else {
                        replacement.push_str(trivia);
                    }
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
        index = group.last().copied().expect("loop group") + 1;
    }

    report_multiline_combinable_loops(&lines, &non_code_ranges, context);
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
    non_code_ranges: &[std::ops::Range<usize>],
    context: &mut CompatibilityCopContext<'_, '_, '_>,
) {
    let mut loops = lines
        .iter()
        .enumerate()
        .filter_map(|(index, (offset, line))| {
            let code_start = *offset + line.len() - line.trim_start().len();
            (!non_code_ranges
                .iter()
                .any(|range| range.start <= code_start && code_start < range.end))
            .then(|| parse_multiline_combinable_loop(lines, index))
            .flatten()
        })
        .collect::<Vec<_>>();
    loops.extend(lines.iter().filter_map(|(offset, line)| {
        let code_start = *offset + line.len() - line.trim_start().len();
        if non_code_ranges
            .iter()
            .any(|range| range.start <= code_start && code_start < range.end)
        {
            return None;
        }
        let parsed = parse_combinable_loop(*offset, line)?;
        Some(MultilineCombinableLoop {
            identity: parsed.identity.to_string(),
            parameters: parsed.parameters.to_string(),
            start: parsed.range.start,
            body_start: parsed.body_start,
            closing_line_start: parsed.closing_start,
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
        let replacement = same_parameters.then(|| {
            let last = &loops[*group.last().expect("loop group")];
            let (mut combined, closing) = if first.multiline {
                (
                    source[first.start..first.closing_line_start].to_string(),
                    format!(" {}", &source[first.closing_line_start..first.end]),
                )
            } else {
                let whole = &source[first.start..first.end];
                let closing_start = whole
                    .rfind('}')
                    .or_else(|| whole.rfind(" end"))
                    .unwrap_or(whole.len());
                (
                    whole[..closing_start].trim_end().to_string(),
                    whole[closing_start..].to_string(),
                )
            };
            if !combined.ends_with('\n') {
                combined.push('\n');
            }
            let mut previous = first;
            for candidate in group.iter().skip(1).map(|candidate| &loops[*candidate]) {
                let trivia = &source[previous.end..candidate.start];
                combined.push_str(
                    trivia
                        .strip_prefix("\r\n")
                        .or_else(|| trivia.strip_prefix('\n'))
                        .unwrap_or(trivia),
                );
                let body = &source[candidate.body_start..candidate.closing_line_start];
                combined.push_str(if candidate.multiline {
                    body.trim_start()
                } else {
                    body.trim()
                });
                previous = candidate;
            }
            combined.push_str(&closing);
            (first.start..last.end, combined)
        });

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
    let (opening, delimiter_len, closing_delimiter) = if let Some(opening) = source.rfind(" do") {
        if source[opening + 3..].contains(" end") {
            return None;
        }
        (opening, 3, "end")
    } else {
        let opening = source.rfind('{')?;
        if source[opening + 1..].contains('}') {
            return None;
        }
        (opening, 1, "}")
    };
    let call = source[..opening].trim_end();
    let method = call
        .rsplit_once('.')
        .map_or(call, |(_, method)| method)
        .split(['(', ' '])
        .next()?;
    if !method.starts_with("each") && !method.ends_with("_each") {
        return None;
    }
    let parameters = source[opening + delimiter_len..].trim().to_string();
    let closing_index = (index + 1..lines.len()).find(|candidate| {
        let candidate_line = lines[*candidate].1;
        let trimmed = candidate_line.trim_start();
        candidate_line.len() - trimmed.len() == indent
            && (trimmed == closing_delimiter
                || trimmed.starts_with(&format!("{closing_delimiter} ")))
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
        end: closing_line_start
            + closing_line.len()
            - closing_line.trim_start().len()
            + closing_delimiter.len(),
        indent,
        multiline: true,
    })
}

fn only_trivia(source: &str) -> bool {
    source.lines().all(|line| {
        let line = line.trim();
        line.is_empty() || line.starts_with('#')
    })
}

struct CombinableLoop<'source> {
    identity: &'source str,
    parameters: &'source str,
    body: &'source str,
    before_closing: &'source str,
    closing_suffix: &'source str,
    body_start: usize,
    closing_start: usize,
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
        (opening, opening + " do".len(), closing, " end")
    } else {
        let method_end = [".each_with_index", ".reverse_each", ".each"]
            .into_iter()
            .find_map(|method| source.find(method).map(|start| start + method.len()))?;
        let receiver_start = method_end
            - [".each_with_index", ".reverse_each", ".each"]
                .into_iter()
                .find(|method| source[..method_end].ends_with(method))?
                .len();
        if source[..receiver_start].contains(['{', '}']) {
            return None;
        }
        let remainder = &source[method_end..];
        if let Some(opening) = remainder.find('{') {
            let opening = method_end + opening;
            let closing = source.rfind('}')?;
            (opening, opening + 1, closing, "}")
        } else {
            let opening = remainder.find(" do")? + method_end;
            let closing = source.rfind(" end")?;
            (opening, opening + " do".len(), closing, " end")
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
        body_start: offset + indent + body_start,
        closing_start: offset + indent + closing_start,
        range: offset + indent..offset + indent + closing_start + closing_suffix.len(),
    })
}

fn each_for_simple_loop(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    let mut non_code_ranges = context.literal_ranges();
    non_code_ranges.extend(context.heredoc_ranges());
    non_code_ranges.extend(context.comment_ranges());
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
        let absolute_open = offset + open;
        if non_code_ranges
            .iter()
            .any(|range| range.start <= absolute_open && absolute_open < range.end)
        {
            continue;
        }
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
            absolute_open..offset + offense_end,
            absolute_open..offset + offense_end,
            format!("{iterations}.times"),
        );
    }
}
