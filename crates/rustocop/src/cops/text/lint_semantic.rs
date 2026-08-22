use super::helpers::*;
use super::{push_offense, CorrectionStatus, Offense, SourceLine};
use crate::config::InspectionConfig;

pub(super) fn check(lines: &[SourceLine], options: &InspectionConfig, offenses: &mut Vec<Offense>) {
    check_unused_method_argument(lines, options, offenses);
    check_debugger(lines, options, offenses);
}

fn check_unused_method_argument(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Lint/UnusedMethodArgument";
    if !options.cop_enabled(cop) {
        return;
    }
    let mut index = 0;
    while index < lines.len() {
        let line = &lines[index].body;
        let trimmed = line.trim();
        if !trimmed.starts_with("def ") || !trimmed.contains('(') {
            index += 1;
            continue;
        }
        let endless = endless_method_parts(trimmed);
        let args = method_arguments(endless.map_or(trimmed, |(signature, _)| signature));
        if args.is_empty() {
            index += 1;
            continue;
        }
        if let Some((_, body)) = endless {
            report_unused_arguments(&args, body, line, index, offenses, cop);
            index += 1;
            continue;
        }
        let Some(end) = find_matching_end(lines, index) else {
            index += 1;
            continue;
        };
        let body = lines[index + 1..end]
            .iter()
            .map(|line| line.body.as_str())
            .collect::<Vec<&str>>()
            .join("\n");
        if body.lines().any(|line| line.trim() == "super")
            || (body.contains("binding") && !body.contains("binding(") && !body.contains("def "))
        {
            index = end + 1;
            continue;
        }
        report_unused_arguments(&args, &body, line, index, offenses, cop);
        index = end + 1;
    }
}

fn endless_method_parts(source: &str) -> Option<(&str, &str)> {
    let open = source.find('(')?;
    let mut depth = 0usize;
    let mut close = None;
    for (offset, character) in source[open..].char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    close = Some(open + offset);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close?;
    let remainder = source[close + 1..].trim_start();
    let body = remainder.strip_prefix('=')?.trim_start();
    Some((&source[..=close], body))
}

fn report_unused_arguments(
    arguments: &[String],
    body: &str,
    signature_line: &str,
    index: usize,
    offenses: &mut Vec<Offense>,
    cop: &'static str,
) {
    for argument in arguments {
        if argument.starts_with('_')
            || body
                .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
                .any(|word| word == argument)
        {
            continue;
        }
        let keyword = signature_line
            .find(argument)
            .and_then(|start| signature_line.as_bytes().get(start + argument.len()))
            == Some(&b':');
        push_offense(
            offenses,
            cop,
            &if keyword {
                format!("Unused method argument - `{argument}`.")
            } else {
                format!(
                    "Unused method argument - `{argument}`. If it's necessary, use `_` or `_{argument}` as an argument name to indicate that it won't be used. If it's unnecessary, remove it."
                )
            },
            index + 1,
            signature_line.find(argument).unwrap_or(0) + 1,
            argument.len(),
            if keyword {
                CorrectionStatus::Unavailable
            } else {
                CorrectionStatus::Pending
            },
        );
    }
}

fn check_debugger(lines: &[SourceLine], options: &InspectionConfig, offenses: &mut Vec<Offense>) {
    let cop = "Lint/Debugger";
    if !options.cop_enabled(cop) {
        return;
    }
    let debuggers = [
        "binding.pry",
        "binding.irb",
        "debugger",
        "byebug",
        "save_and_open_page",
        "save_and_open_screenshot",
    ];
    for (index, line) in lines.iter().enumerate() {
        for debugger in debuggers {
            if let Some(position) = strip_comment(&line.body).find(debugger) {
                push_offense(
                    offenses,
                    cop,
                    "Remove debugger entry point.",
                    index + 1,
                    position + 1,
                    debugger.len(),
                    CorrectionStatus::Unavailable,
                );
            }
        }
    }
}
