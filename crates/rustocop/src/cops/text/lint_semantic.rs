use super::helpers::*;
use super::{push_offense, CorrectionStatus, Offense, SourceLine};
use crate::config::InspectionConfig;

pub(super) fn check(lines: &[SourceLine], options: &InspectionConfig, offenses: &mut Vec<Offense>) {
    check_accessor_method_name(lines, options, offenses);
    check_unused_method_argument(lines, options, offenses);
    check_debugger(lines, options, offenses);
}

fn check_accessor_method_name(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Naming/AccessorMethodName";
    if !options.cop_enabled(cop) {
        return;
    }
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.body.trim_start();
        let Some(name) = trimmed.strip_prefix("def ").and_then(first_identifier) else {
            continue;
        };
        if name.starts_with("get_") || name.starts_with("set_") {
            push_offense(
                offenses,
                cop,
                "Do not prefix reader method names with `get_` or writer method names with `set_`.",
                index + 1,
                line.body.find("def").unwrap_or(0) + 5,
                name.len(),
                CorrectionStatus::Unavailable,
            );
        }
    }
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
        let args = method_arguments(trimmed);
        if args.is_empty() {
            index += 1;
            continue;
        }
        let end = find_matching_end(lines, index).unwrap_or(index);
        let body = lines[index + 1..end]
            .iter()
            .map(|line| line.body.as_str())
            .collect::<Vec<&str>>()
            .join("\n");
        for arg in args {
            if arg.starts_with('_') || body.contains(&arg) {
                continue;
            }
            push_offense(
                offenses,
                cop,
                &format!("Unused method argument - `{}`.", arg),
                index + 1,
                line.find(&arg).unwrap_or(0) + 1,
                arg.len(),
                CorrectionStatus::Unavailable,
            );
        }
        index = end + 1;
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
