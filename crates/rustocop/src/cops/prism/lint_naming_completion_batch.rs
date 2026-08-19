use super::*;
use std::collections::{HashMap, HashSet};

define_cops! {
    UnderscorePrefixedVariableName => "Lint/UnderscorePrefixedVariableName" => source(underscore_variable),
    HeredocDelimiterNaming => "Naming/HeredocDelimiterNaming" => source(heredoc_naming),
    DeprecatedConstants => "Lint/DeprecatedConstants" => source(deprecated_constants),
    RedundantCopEnableDirective => "Lint/RedundantCopEnableDirective" => source(redundant_enable),
    UnreachablePatternBranch => "Lint/UnreachablePatternBranch" => source(unreachable_pattern),
    MethodParameterName => "Naming/MethodParameterName" => source(method_parameter_name),
}

fn underscore_variable(context: &mut CopContext<'_, '_>) {
    let mut candidates = Vec::new();
    let mut occurrences = HashMap::new();
    for (offset, line) in context.source_file().lines() {
        for (at, _) in line.match_indices('_') {
            let tail = &line[at..];
            let len = tail
                .bytes()
                .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
                .count();
            if len < 2 {
                continue;
            }
            let name = &tail[..len];
            *occurrences.entry(name).or_insert(0) += 1;
            candidates.push((offset + at, name));
        }
    }
    let mut seen = HashSet::new();
    for (start, name) in candidates {
        if occurrences.get(name).copied().unwrap_or_default() > 1 && seen.insert(start) {
            context.report(
                "Do not use prefix `_` for a variable that is used.",
                start..start + name.len(),
            );
        }
    }
}

fn heredoc_naming(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for (line_offset, line) in context.source_file().lines() {
        let Some(at) = line.find("<<") else { continue };
        let delimiter = line[at + 2..]
            .trim_start_matches(['-', '~', '\'', '"', '`'])
            .trim_end_matches(['\'', '"', '`'])
            .trim();
        if !matches!(delimiter, "END" | "EOH" | "EOS" | "EOL") {
            continue;
        }
        if let Some(start) = source[line_offset..].rfind(&format!("\n{delimiter}")) {
            let absolute = line_offset + start + 1;
            context.report(
                "Use meaningful heredoc delimiters.",
                absolute..absolute + delimiter.len(),
            );
        }
    }
}

fn deprecated_constants(context: &mut CopContext<'_, '_>) {
    for (old, new) in [("NIL", "nil"), ("TRUE", "true"), ("FALSE", "false")] {
        for start in context.source_file().code_offsets(old) {
            context.replace(
                format!("Use `{new}` instead of `{old}`, deprecated since Ruby 2.4."),
                start..start + old.len(),
                start..start + old.len(),
                new,
            );
        }
    }
}

fn redundant_enable(context: &mut CopContext<'_, '_>) {
    let mut disabled = HashSet::new();
    for (offset, line) in context.source_file().lines() {
        if let Some(list) = line.split("rubocop:disable ").nth(1) {
            disabled.extend(list.split(',').map(|cop| cop.trim().to_string()));
        }
        let Some(list) = line.split("rubocop:enable ").nth(1) else {
            continue;
        };
        for cop in list.split(',').map(str::trim) {
            if disabled.remove(cop) {
                continue;
            }
            let start = offset + line.find(cop).unwrap_or(0);
            context.remove(
                format!("Unnecessary enabling of {cop}."),
                start..start + cop.len(),
                start..start + cop.len(),
            );
        }
    }
}

fn unreachable_pattern(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut catch_all = false;
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        let Some(pattern) = line.trim_start().strip_prefix("in ") else {
            continue;
        };
        if catch_all {
            let end = lines[index + 1..]
                .iter()
                .find(|(_, next)| next.trim_start().starts_with("in ") || next.trim() == "end")
                .map_or(offset + line.len(), |(at, _)| *at);
            context.report(
                "Unreachable `in` pattern branch detected.",
                offset..end.saturating_sub(1),
            );
        }
        catch_all = pattern.trim() == "_"
            || pattern
                .trim()
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b == b'_');
    }
}

fn method_parameter_name(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("def ") {
            continue;
        }
        let Some(open) = line.find('(') else { continue };
        let Some(close) = line[open..].find(')').map(|at| open + at) else {
            continue;
        };
        for parameter in line[open + 1..close]
            .split(',')
            .map(|p| p.trim().trim_start_matches(['*', '&']))
        {
            if parameter.chars().last().is_some_and(|c| c.is_ascii_digit()) {
                let start = offset + line.find(parameter).unwrap_or(0);
                context.report(
                    "Do not end method parameter with a number.",
                    start..start + parameter.len(),
                );
            }
        }
    }
}
