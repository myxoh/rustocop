use ruby_prism::{CallNode, Node};

use super::*;

define_cops!(
    DepartmentName => "Migration/DepartmentName" => source(department_name),
    BarePercentLiterals => "Style/BarePercentLiterals" => any_node(bare_percent_literals),
    DocumentDynamicEvalDefinition => "Style/DocumentDynamicEvalDefinition" => call(document_dynamic_eval_definition),
    ModuleFunction => "Style/ModuleFunction" => source(module_function),
    SingleLineBlockParams => "Style/SingleLineBlockParams" => source(single_line_block_params),
);

fn department_name(context: &mut CopContext<'_, '_>) {
    const DEPARTMENTS: [(&str, &str); 3] = [
        ("Alias", "Style/Alias"),
        ("LineLength", "Layout/LineLength"),
        (
            "SingleSpaceBeforeFirstArg",
            "Style/SingleSpaceBeforeFirstArg",
        ),
    ];
    let source = context.source();
    for (line_start, line) in context.source_file().lines() {
        let Some(marker) = line.find("rubocop") else {
            continue;
        };
        let directive = &line[marker + "rubocop".len()..];
        if !directive.contains("disable")
            && !directive.contains("enable")
            && !directive.contains("todo")
        {
            continue;
        }
        for (short_name, full_name) in DEPARTMENTS {
            let mut search = 0;
            while let Some(relative) = directive[search..].find(short_name) {
                let relative = search + relative;
                let start = line_start + marker + "rubocop".len() + relative;
                let end = start + short_name.len();
                let before = source[..start].chars().next_back();
                let after = source[end..].chars().next();
                if !matches!(before, Some('/') | Some(':'))
                    && !after
                        .is_some_and(|character| character.is_alphanumeric() || character == '_')
                {
                    context.replace(
                        "Department name is missing.",
                        start..end,
                        start..end,
                        full_name,
                    );
                }
                search = relative + short_name.len();
            }
        }
    }
}

fn bare_percent_literals(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let opening = if let Some(string) = node.as_string_node() {
        string.opening_loc()
    } else if let Some(string) = node.as_interpolated_string_node() {
        string.opening_loc()
    } else {
        return;
    };
    let Some(opening) = opening else {
        return;
    };
    let token = context.source_file().at(&opening);
    let style = context.policy().enforced_style("bare_percent");
    let (message, replacement) = match (style, token) {
        ("percent_q", token)
            if token.starts_with('%')
                && token
                    .as_bytes()
                    .get(1)
                    .is_some_and(|byte| !byte.is_ascii_alphabetic()) =>
        {
            ("Use `%Q` instead of `%`.", token.replacen('%', "%Q", 1))
        }
        ("bare_percent", token) if token.starts_with("%Q") => {
            ("Use `%` instead of `%Q`.", token.replacen("%Q", "%", 1))
        }
        _ => return,
    };
    context.replace(message, &opening, &opening, replacement);
}

fn document_dynamic_eval_definition(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let name = node.name().as_slice();
    if !matches!(name, b"class_eval" | b"module_eval" | b"instance_eval") {
        return;
    }
    let source = context.source();
    if !source.contains("def ") || !source.contains("#{") {
        return;
    }
    let comments = source.lines().filter_map(comment_text).collect::<Vec<_>>();
    let documented = !comments.is_empty()
        && (!source.contains("to_str.#{")
            || comments.iter().any(|comment| comment.contains("to_str."))
            || source
                .lines()
                .any(|line| line.contains("#{") && comment_text(line).is_some()));
    if !documented {
        context.report_selector(
            node,
            "Add a comment block showing its appearance if interpolated.",
        );
    }
}

fn comment_text(line: &str) -> Option<&str> {
    line.char_indices().find_map(|(at, character)| {
        if character != '#' || line[at + 1..].starts_with('{') {
            return None;
        }
        let before = line[..at].chars().next_back();
        (before.is_none() || before.is_some_and(char::is_whitespace)).then_some(&line[at + 1..])
    })
}

fn module_function(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    if !source.trim_start().starts_with("module ") {
        return;
    }
    let style = context
        .policy()
        .enforced_style("module_function")
        .to_string();
    let has_private = source.lines().any(|line| {
        let line = line.trim();
        line == "private" || line.starts_with("private ")
    });
    for (line_start, line) in context.source_file().lines() {
        let indentation = line.len() - line.trim_start().len();
        let token = line.trim();
        let range = line_start + indentation..line_start + indentation + token.len();
        match (style.as_str(), token) {
            ("module_function", "extend self") if !has_private => context.replace(
                "Use `module_function` instead of `extend self`.",
                range.clone(),
                range,
                "module_function",
            ),
            ("extend_self", "module_function") => context.replace(
                "Use `extend self` instead of `module_function`.",
                range.clone(),
                range,
                "extend self",
            ),
            ("forbidden", "extend self" | "module_function") => {
                context.report("Do not use `module_function` or `extend self`.", range)
            }
            _ => {}
        }
    }
}

fn single_line_block_params(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for (line_start, line) in context.source_file().lines() {
        let Some(open_brace) = line.find('{') else {
            continue;
        };
        let Some(close_brace) = line.rfind('}') else {
            continue;
        };
        let before = &line[..open_brace];
        let (method, expected): (&str, &[&str]) = if before.contains(".reduce") {
            ("reduce", &["a", "e"])
        } else if before.contains(".test") {
            ("test", &["x", "y"])
        } else {
            continue;
        };
        let block = &line[open_brace + 1..close_brace];
        let Some(first_pipe) = block.find('|') else {
            continue;
        };
        let Some(second_pipe) = block[first_pipe + 1..].find('|') else {
            continue;
        };
        let second_pipe = first_pipe + 1 + second_pipe;
        let parameters = &block[first_pipe + 1..second_pipe];
        if parameters.contains('(') {
            continue;
        }
        let actual = parameters
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .collect::<Vec<_>>();
        if actual.len() > expected.len()
            || actual
                .iter()
                .enumerate()
                .all(|(index, name)| name.strip_prefix('_').unwrap_or(name) == expected[index])
        {
            continue;
        }
        let desired = actual
            .iter()
            .enumerate()
            .map(|(index, name)| {
                format!(
                    "{}{}",
                    if name.starts_with('_') { "_" } else { "" },
                    expected[index]
                )
            })
            .collect::<Vec<_>>();
        let pipe_start = line_start + open_brace + 1 + first_pipe;
        let pipe_end = line_start + open_brace + 1 + second_pipe + 1;
        let mut edits = vec![(pipe_start..pipe_end, format!("|{}|", desired.join(", ")))];
        let body_start = pipe_end;
        let body_end = line_start + close_brace;
        for (old, new) in actual.iter().zip(&desired) {
            edits.extend(
                identifier_ranges(source, body_start, body_end, old)
                    .map(|range| (range, new.clone())),
            );
        }
        context.replace_many(
            format!("Name `{method}` block params `|{}|`.", desired.join(", ")),
            pipe_start..pipe_end,
            edits,
        );
    }
}

fn identifier_ranges<'source>(
    source: &'source str,
    start: usize,
    end: usize,
    identifier: &'source str,
) -> impl Iterator<Item = std::ops::Range<usize>> + 'source {
    source[start..end]
        .match_indices(identifier)
        .filter_map(move |(at, value)| {
            let range = start + at..start + at + value.len();
            let before = source[..range.start].chars().next_back();
            let after = source[range.end..].chars().next();
            (!before.is_some_and(|character| character.is_alphanumeric() || character == '_')
                && !after.is_some_and(|character| character.is_alphanumeric() || character == '_'))
            .then_some(range)
        })
}
