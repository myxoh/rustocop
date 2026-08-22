use super::catalog_cop::{custom, report};
use super::*;
use std::collections::HashSet;

mod naming;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    let mut cops = vec![
        custom("Lint/ShadowedException", shadowed_exception),
        custom("Lint/ConstantDefinitionInBlock", constant_in_block),
        custom("Lint/ShadowingOuterLocalVariable", shadowing_outer_local),
        report(
            "Lint/LiteralAssignmentInCondition",
            "if value = 1",
            "Do not use a literal assignment in a condition.",
        ),
        custom("Naming/HeredocDelimiterCase", heredoc_case),
        custom("Naming/BlockForwarding", block_forwarding),
        custom("Lint/AmbiguousAssignment", ambiguous_assignment),
        custom(
            "Naming/RescuedExceptionsVariableName",
            rescued_exception_name,
        ),
        custom("Lint/ConstantReassignment", constant_reassignment),
    ];
    cops.extend(naming::cops());
    cops
}

fn ambiguous_assignment(context: &mut CopContext<'_, '_>) {
    for (needle, operator) in [("=-", "-"), ("=+", "+"), ("=*", "*"), ("=!", "!")] {
        for start in context.source_file().code_offsets(needle) {
            if !context.source()[..start]
                .as_bytes()
                .last()
                .is_some_and(|byte| byte.is_ascii_whitespace())
            {
                continue;
            }
            if needle == "=!" && context.source().as_bytes().get(start + 2) == Some(&b'!') {
                continue;
            }
            context.report(
                format!("Suspicious assignment detected. Did you mean `{operator}=`?"),
                start..start + needle.len(),
            );
        }
    }
}

fn shadowed_exception(context: &mut CopContext<'_, '_>) {
    let mut rescued = None::<String>;
    for (offset, line) in context.source_file().lines() {
        if let Some((_, name)) = line.split_once("rescue => ") {
            rescued = Some(name.trim().to_string());
        } else if let Some(name) = &rescued {
            if let Some(at) = line.find(&format!("{name} =")) {
                context.report(
                    "Rescued exception variable is overwritten.",
                    offset + at..offset + at + name.len(),
                );
            }
        }
        if line.trim() == "end" {
            rescued = None;
        }
    }
}

fn constant_in_block(context: &mut CopContext<'_, '_>) {
    let mut block_depth = 0;
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        if trimmed.contains(" do") || trimmed.ends_with('{') {
            block_depth += 1;
        }
        if block_depth > 0 {
            let name = trimmed.split('=').next().unwrap_or("").trim();
            if !name.is_empty()
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte == b'_')
            {
                let start = offset + line.find(name).unwrap_or(0);
                context.report(
                    "Do not define constants inside a block.",
                    start..start + name.len(),
                );
            }
        }
        if trimmed == "end" && block_depth > 0 {
            block_depth -= 1;
        }
    }
}

fn shadowing_outer_local(context: &mut CopContext<'_, '_>) {
    let mut locals = HashSet::new();
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        if ["def ", "class ", "module "]
            .iter()
            .any(|prefix| trimmed.starts_with(prefix))
            || trimmed == "end"
        {
            locals.clear();
        }
        if let Some((name, _)) = line.split_once(" = ") {
            let name = name.trim();
            if !name.is_empty()
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
                && !line.contains(&format!("|{name}|"))
                && !line.contains(&format!("|{name},"))
                && !line.contains(&format!(", {name}|"))
            {
                locals.clear();
                locals.insert(name.to_string());
            }
        }
        if let Some(first) = line.find('|') {
            if let Some(close) = line[first + 1..].find('|').map(|at| first + 1 + at) {
                for argument in line[first + 1..close].split(',').map(str::trim) {
                    if locals.contains(argument) {
                        let start =
                            offset + first + 1 + line[first + 1..close].find(argument).unwrap_or(0);
                        context.report(
                            "Shadowing outer local variable.",
                            start..start + argument.len(),
                        );
                    }
                }
            }
        }
        if !trimmed.is_empty()
            && !line.contains(" = ")
            && !line.contains('|')
            && !trimmed.starts_with('#')
        {
            locals.clear();
        }
    }
}

fn heredoc_case(context: &mut CopContext<'_, '_>) {
    let uppercase = context.policy().enforced_style("uppercase") == "uppercase";
    for (offset, line) in context.source_file().lines() {
        let Some(at) = line.find("<<") else { continue };
        let delimiter = line[at + 2..]
            .trim_start_matches(['-', '~', '\'', '"', '`'])
            .trim_end_matches(['\'', '"', '`'])
            .trim();
        let wrong_case = if uppercase {
            delimiter.bytes().any(|byte| byte.is_ascii_lowercase())
        } else {
            delimiter.bytes().any(|byte| byte.is_ascii_uppercase())
        };
        if !delimiter.is_empty() && wrong_case {
            let start = context.source()[offset..]
                .find(&format!("\n{delimiter}"))
                .map_or(offset + line.rfind(delimiter).unwrap_or(at + 2), |relative| {
                    offset + relative + 1
                });
            context.replace(
                if uppercase {
                    "Use uppercase heredoc delimiters."
                } else {
                    "Use lowercase heredoc delimiters."
                },
                start..start + delimiter.len(),
                start..start + delimiter.len(),
                if uppercase {
                    delimiter.to_ascii_uppercase()
                } else {
                    delimiter.to_ascii_lowercase()
                },
            );
        }
    }
}

fn rescued_exception_name(context: &mut CopContext<'_, '_>) {
    let preferred = context
        .config_value("PreferredName")
        .unwrap_or("e")
        .to_string();
    let mut rescue_indent = None;
    for (offset, line) in context.source_file().lines() {
        if !line.trim_start().starts_with("rescue") {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        if rescue_indent.is_some_and(|outer| indent > outer) {
            continue;
        }
        rescue_indent.get_or_insert(indent);
        let Some((_, actual)) = line.split_once("=>") else {
            continue;
        };
        let actual = actual
            .trim_start()
            .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
            .next()
            .unwrap_or("");
        if line
            .find(actual)
            .and_then(|start| line.as_bytes().get(start + actual.len()))
            == Some(&b'.')
        {
            continue;
        }
        let expected = if actual.starts_with('_') && !preferred.starts_with('_') {
            format!("_{preferred}")
        } else {
            preferred.clone()
        };
        if context.source()[..offset].contains(&format!("{preferred} =")) {
            continue;
        }
        if !actual.is_empty() && actual != expected {
            let start = offset + line.rfind(actual).unwrap_or(0);
            context.replace(
                format!("Use `{expected}` instead of `{actual}`."),
                start..start + actual.len(),
                start..start + actual.len(),
                &expected,
            );
        }
    }
}

fn block_forwarding(context: &mut CopContext<'_, '_>) {
    let style = context.policy().enforced_style("anonymous");
    if style == "explicit" {
        let block_name_in_use = context.source().lines().any(|line| {
            line.trim_start().starts_with("def ") && line.contains("block, &")
        });
        for start in context.source_file().code_offsets("&") {
            let next = context.source().as_bytes().get(start + 1).copied();
            if next.is_some_and(|byte| byte == b':' || byte == b'_' || byte.is_ascii_alphanumeric()) {
                continue;
            }
            if block_name_in_use {
                context.report("Use explicit block forwarding.", start..start + 1);
            } else {
                context.replace(
                    "Use explicit block forwarding.",
                    start..start + 1,
                    start..start + 1,
                    "&block",
                );
            }
        }
        return;
    }
    if style != "anonymous" {
        return;
    }
    if !context.target_ruby_version().at_least(3, 1) {
        return;
    }
    let source = context.source();
    if source.contains(" if block")
        || source.contains("block.call")
        || source.lines().any(|line| {
            let line = line.trim_start();
            line == "block" || line.starts_with("block =") || line.starts_with("block ||=")
        })
        || source.lines().any(|line| {
            line.trim_start().starts_with("def ")
                && line
                    .split_once("&block")
                    .is_some_and(|(before, _)| before.contains(':'))
        })
        || (!context.target_ruby_version().at_least(3, 4)
            && source.contains("block_method do"))
    {
        return;
    }
    for start in context.source_file().code_offsets("&block") {
        context.replace(
            "Use anonymous block forwarding.",
            start..start + 6,
            start..start + 6,
            "&",
        );
    }
}

fn constant_reassignment(context: &mut CopContext<'_, '_>) {
    if context.source().contains("remove_const")
        || context.source().contains(" do\n")
        || context.source().contains(" unless ")
        || context
            .source()
            .lines()
            .any(|line| line.trim_start().starts_with("if "))
    {
        return;
    }
    if context
        .source()
        .lines()
        .filter(|line| {
            ["class ", "module "]
                .iter()
                .any(|keyword| line.trim_start().starts_with(keyword))
        })
        .count()
        > 1
    {
        return;
    }
    let mut constants = HashSet::new();
    for (offset, line) in context.source_file().lines() {
        if [
            "class ", "module ", "def ", "if ", "unless ", "case ", "begin",
        ]
        .iter()
        .any(|keyword| line.trim_start().starts_with(keyword))
            || matches!(line.trim(), "end" | "else" | "elsif" | "rescue" | "ensure")
        {
            constants.clear();
        }
        if line.contains("||=") {
            continue;
        }
        let Some((name, _)) = line.split_once('=') else {
            continue;
        };
        let name = name.trim();
        if !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte == b'_')
            && !constants.insert(name.to_string())
        {
            let start = offset + line.find(name).unwrap_or(0);
            context.report("Constant is already assigned.", start..start + name.len());
        }
    }
}
