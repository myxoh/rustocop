use ruby_prism::{CallNode, Node};

use super::*;

define_cops!(
    DepartmentName => "Migration/DepartmentName" => source(department_name),
    BarePercentLiterals => "Style/BarePercentLiterals" => any_node(bare_percent_literals),
    DocumentDynamicEvalDefinition => "Style/DocumentDynamicEvalDefinition" => call(document_dynamic_eval_definition),
    ModuleFunction => "Style/ModuleFunction" => source(module_function),
    SingleLineBlockParams => "Style/SingleLineBlockParams" => node(as_block_node, single_line_block_params),
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

fn single_line_block_params(node: &ruby_prism::BlockNode<'_>, context: &mut CopContext<'_, '_>) {
    let location = node.location();
    if !context.source_file().same_line(
        location.start_offset(),
        location.end_offset().saturating_sub(1),
    ) {
        return;
    }
    let Some(call) = context
        .ancestors()
        .iter()
        .rev()
        .find_map(Node::as_call_node)
    else {
        return;
    };
    if call.receiver().is_none() {
        return;
    }
    let method = String::from_utf8_lossy(call.name().as_slice());
    let Some(expected) = configured_block_params(context, method.as_ref()) else {
        return;
    };
    let Some(block_parameters) = node
        .parameters()
        .and_then(|parameters| parameters.as_block_parameters_node())
    else {
        return;
    };
    let Some(parameters) = block_parameters.parameters() else {
        return;
    };
    if !block_parameters.locals().is_empty()
        || !parameters.optionals().is_empty()
        || parameters.rest().is_some()
        || !parameters.posts().is_empty()
        || !parameters.keywords().is_empty()
        || parameters.keyword_rest().is_some()
        || parameters.block().is_some()
    {
        return;
    }
    let required = parameters.requireds().iter().collect::<Vec<_>>();
    let actual = required
        .iter()
        .map(|parameter| parameter.as_required_parameter_node())
        .collect::<Option<Vec<_>>>();
    let Some(actual) = actual else {
        return;
    };
    if actual.is_empty() || actual.len() > expected.len() {
        return;
    }
    let actual_names = actual
        .iter()
        .map(|parameter| String::from_utf8_lossy(parameter.name().as_slice()).into_owned())
        .collect::<Vec<_>>();
    if actual_names
        .iter()
        .zip(&expected)
        .all(|(actual, expected)| actual.trim_start_matches('_') == expected)
    {
        return;
    }

    let desired = actual_names
        .iter()
        .zip(&expected)
        .map(|(actual, expected)| {
            format!(
                "{}{expected}",
                if actual.starts_with('_') { "_" } else { "" }
            )
        })
        .collect::<Vec<_>>();
    let replacements = actual_names
        .iter()
        .zip(&desired)
        .map(|(actual, desired)| (actual.as_bytes().to_vec(), desired.clone()))
        .collect::<std::collections::HashMap<_, _>>();
    let mut reads = LocalReadEdits {
        replacements: &replacements,
        edits: Vec::new(),
    };
    if let Some(body) = node.body() {
        ruby_prism::Visit::visit(&mut reads, &body);
    }
    let parameter_range =
        block_parameters.location().start_offset()..block_parameters.location().end_offset();
    let mut edits = vec![(parameter_range.clone(), format!("|{}|", desired.join(", ")))];
    edits.extend(reads.edits);
    context.replace_many(
        format!("Name `{method}` block params `|{}|`.", desired.join(", ")),
        parameter_range,
        edits,
    );
}

fn configured_block_params(context: &CopContext<'_, '_>, method: &str) -> Option<Vec<String>> {
    let mut current = None;
    let mut parameters = Vec::new();
    for item in context.config_values("Methods") {
        if let Some(name) = item.strip_suffix(':') {
            if current.as_deref() == Some(method) {
                return Some(parameters);
            }
            current = Some(name.to_string());
            parameters = Vec::new();
        } else if current.is_some() {
            parameters.push(item.clone());
        }
    }
    (current.as_deref() == Some(method)).then_some(parameters)
}

struct LocalReadEdits<'a> {
    replacements: &'a std::collections::HashMap<Vec<u8>, String>,
    edits: Vec<(std::ops::Range<usize>, String)>,
}

impl<'pr> ruby_prism::Visit<'pr> for LocalReadEdits<'_> {
    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode<'pr>) {
        if let Some(replacement) = self.replacements.get(node.name().as_slice()) {
            let location = node.location();
            self.edits.push((
                location.start_offset()..location.end_offset(),
                replacement.clone(),
            ));
        }
    }
}
