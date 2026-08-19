use super::catalog_cop::{custom, replace, report};
use super::*;
use std::collections::HashSet;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        custom("Lint/ShadowedException", shadowed_exception),
        custom("Lint/ConstantDefinitionInBlock", constant_in_block),
        custom(
            "Style/ReturnNilInPredicateMethodDefinition",
            return_nil_predicate,
        ),
        custom("Lint/ShadowingOuterLocalVariable", shadowing_outer_local),
        report(
            "Lint/LiteralAssignmentInCondition",
            "if value = 1",
            "Do not use a literal assignment in a condition.",
        ),
        custom("Naming/HeredocDelimiterCase", heredoc_case),
        replace(
            "Style/OrAssignment",
            "value = value || ",
            "value ||= ",
            "Use self-assignment shorthand `||=`.",
        ),
        custom("Naming/BlockForwarding", block_forwarding),
        report(
            "Lint/AmbiguousAssignment",
            "= puts ",
            "Wrap the right hand side in parentheses to avoid ambiguity.",
        ),
        custom(
            "Naming/RescuedExceptionsVariableName",
            rescued_exception_name,
        ),
        custom("Lint/ConstantReassignment", constant_reassignment),
        custom("Lint/ShadowedArgument", shadowed_argument),
        custom("Naming/InclusiveLanguage", inclusive_language),
    ]
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

fn return_nil_predicate(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut predicate = false;
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        if line.trim_start().starts_with("def ") {
            let method = line.trim_start()[4..]
                .split(['(', ' '])
                .next()
                .unwrap_or("")
                .rsplit('.')
                .next()
                .unwrap_or("")
                .to_string();
            predicate = method.ends_with('?')
                && !context
                    .config_values("AllowedMethods")
                    .iter()
                    .any(|name| name == &method)
                && context.config_values("AllowedPatterns").is_empty();
        }
        if predicate
            && line.trim() == "nil"
            && lines
                .get(index + 1)
                .is_some_and(|(_, line)| line.trim() == "end")
        {
            context.replace(
                "Return `false` instead of `nil` in a predicate method.",
                offset..offset + line.len(),
                offset..offset + line.len(),
                format!("{}false", &line[..line.len() - line.trim_start().len()]),
            );
        }
        if line.trim() == "end" {
            predicate = false;
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
            .trim_start_matches(['-', '~', '\'', '"'])
            .trim_end_matches(['\'', '"'])
            .trim();
        if uppercase
            && !delimiter.is_empty()
            && delimiter.bytes().any(|byte| byte.is_ascii_lowercase())
        {
            let start = offset + line.rfind(delimiter).unwrap_or(at + 2);
            context.replace(
                "Use uppercase heredoc delimiters.",
                start..start + delimiter.len(),
                start..start + delimiter.len(),
                delimiter.to_ascii_uppercase(),
            );
        }
    }
}

fn rescued_exception_name(context: &mut CopContext<'_, '_>) {
    let preferred = context
        .config_value("PreferredName")
        .unwrap_or("e")
        .to_string();
    for (offset, line) in context.source_file().lines() {
        let Some((_, actual)) = line.split_once("rescue => ") else {
            continue;
        };
        let actual = actual.trim();
        if actual != preferred {
            let start = offset + line.find(actual).unwrap_or(0);
            context.replace(
                format!("Use `{preferred}` instead of `{actual}` for a rescued exception."),
                start..start + actual.len(),
                start..start + actual.len(),
                &preferred,
            );
        }
    }
}

fn block_forwarding(context: &mut CopContext<'_, '_>) {
    if context.policy().enforced_style("anonymous") != "anonymous" {
        return;
    }
    let source = context.source();
    if source.contains(" if block")
        || source.contains("block.call")
        || source.lines().any(|line| {
            let line = line.trim_start();
            line == "block" || line.starts_with("block =") || line.starts_with("block ||=")
        })
        || source
            .lines()
            .any(|line| line.trim_start().starts_with("def ") && line.contains(":, &block"))
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

fn shadowed_argument(context: &mut CopContext<'_, '_>) {
    if context.source().contains(" if ") || context.source().contains(" unless ") {
        return;
    }
    if context.config_bool("IgnoreImplicitReferences", false)
        && (context.source().contains("super") || context.source().contains("binding"))
    {
        return;
    }
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for (index, (_, line)) in lines.iter().copied().enumerate() {
        if line.trim_start().starts_with("def ") {
            let arguments = line
                .split_once('(')
                .and_then(|(_, rest)| rest.rsplit_once(')'))
                .map_or_else(Vec::new, |(args, _)| {
                    args.split(',').map(|arg| arg.trim().to_string()).collect()
                });
            let Some((offset, assignment)) = lines.get(index + 1).copied() else {
                continue;
            };
            for argument in &arguments {
                if let Some(at) = assignment.find(&format!("{argument} =")) {
                    if assignment[..at].contains('{') {
                        continue;
                    }
                    let used_later = lines[index + 2..]
                        .iter()
                        .take_while(|(_, line)| line.trim() != "end")
                        .any(|(_, line)| {
                            line.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                                .any(|word| word == argument)
                        });
                    if !used_later {
                        continue;
                    }
                    context.report(
                        "Method argument is overwritten before it is used.",
                        offset + at..offset + at + argument.len(),
                    );
                }
            }
        }
    }
}

fn inclusive_language(context: &mut CopContext<'_, '_>) {
    for (old, new) in [
        ("whitelist", "allowlist"),
        ("blacklist", "denylist"),
        ("master", "primary"),
        ("slave", "replica"),
    ] {
        for start in context
            .source()
            .match_indices(old)
            .map(|(start, _)| start)
            .collect::<Vec<_>>()
        {
            let before = &context.source()[..start];
            let line_start = before.rfind('\n').map_or(0, |at| at + 1);
            let line = &context.source()[line_start
                ..context.source()[start..]
                    .find('\n')
                    .map_or(context.source().len(), |len| start + len)];
            let in_string = line[..start - line_start].matches('"').count() % 2 == 1;
            let symbol = start > 0 && context.source().as_bytes()[start - 1] == b':';
            let variable =
                start > 0 && matches!(context.source().as_bytes()[start - 1], b'@' | b'$');
            if (!context.config_bool("CheckIdentifiers", true)
                && !in_string
                && !symbol
                && !variable)
                || (!context.config_bool("CheckVariables", true) && variable)
                || (!context.config_bool("CheckStrings", false) && in_string)
                || (!context.config_bool("CheckSymbols", true) && symbol)
            {
                continue;
            }
            context.replace(
                format!("Use inclusive language: replace `{old}` with `{new}`."),
                start..start + old.len(),
                start..start + old.len(),
                new,
            );
        }
    }
}
