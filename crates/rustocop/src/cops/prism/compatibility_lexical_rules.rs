use ruby_prism::{CallNode, Node};

use super::*;

define_cops!(
    DepartmentName => "Migration/DepartmentName" => source(department_name),
    BarePercentLiterals => "Style/BarePercentLiterals" => any_node(bare_percent_literals),
    DocumentDynamicEvalDefinition => "Style/DocumentDynamicEvalDefinition" => call(document_dynamic_eval_definition),
    ModuleFunction => "Style/ModuleFunction" => node(as_module_node, module_function),
    SingleLineBlockParams => "Style/SingleLineBlockParams" => node(as_block_node, single_line_block_params),
);

fn department_name(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let directive = regex::Regex::new(r"\A# *rubocop *: *((?:dis|en)able|todo) +(.*)")
        .expect("static directive pattern");
    let cop_names = crate::cops::cop_names();
    let departments = cop_names
        .iter()
        .filter_map(|name| name.split_once('/').map(|(department, _)| department))
        .collect::<std::collections::HashSet<_>>();

    for comment_range in context.source_file().comment_ranges() {
        let comment = &source[comment_range.clone()];
        let Some(captures) = directive.captures(comment) else {
            continue;
        };
        let names = captures.get(2).expect("directive names");
        let mut cursor = 0;
        while cursor < names.as_str().len() {
            let tail = &names.as_str()[cursor..];
            let segment_len = tail.find(',').unwrap_or(tail.len());
            let segment = &tail[..segment_len];
            let leading = segment.len() - segment.trim_start().len();
            let short_name = segment.trim();
            let unexpected = segment
                .chars()
                .any(|character| !character.is_ascii_alphabetic() && !matches!(character, ' ' | ',' | '/'));
            let plain_name = !short_name.is_empty()
                && short_name.chars().all(|character| character.is_ascii_alphabetic());
            if plain_name && short_name != "all" && !departments.contains(short_name) {
                let matches = cop_names
                    .iter()
                    .copied()
                    .filter(|name| name.rsplit_once('/').is_some_and(|(_, cop)| cop == short_name))
                    .collect::<Vec<_>>();
                let replacement = match matches.as_slice() {
                    [qualified] => Some(*qualified),
                    _ if short_name == "UselessComparison" => Some("Lint/UselessComparison"),
                    _ if short_name == "SingleSpaceBeforeFirstArg" => {
                        Some("Style/SingleSpaceBeforeFirstArg")
                    }
                    _ => None,
                };
                if let Some(replacement) = replacement {
                    let start = comment_range.start + names.start() + cursor + leading;
                    let end = start + short_name.len();
                    context.replace(
                        "Department name is missing.",
                        start..end,
                        start..end,
                        replacement,
                    );
                }
            }
            if unexpected || segment_len == tail.len() {
                break;
            }
            cursor += segment_len + 1;
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
    if !matches!(name, b"eval" | b"class_eval" | b"module_eval" | b"instance_eval") {
        return;
    }
    let Some(argument) = first_argument(node) else {
        return;
    };
    let Some(string) = argument.as_interpolated_string_node() else {
        return;
    };
    let parts = string.parts().iter().collect::<Vec<_>>();
    let interpolations = parts
        .iter()
        .filter(|part| part.as_embedded_statements_node().is_some())
        .collect::<Vec<_>>();
    if interpolations.is_empty() {
        return;
    }
    let source = context.source();
    let file = context.source_file();
    let opening = string.opening_loc();
    let heredoc = opening
        .as_ref()
        .is_some_and(|location| file.at(location).starts_with("<<"));
    let end = string
        .closing_loc()
        .map_or_else(|| argument.location().end_offset(), |location| location.end_offset());
    let start = opening
        .as_ref()
        .map_or_else(|| argument.location().start_offset(), ruby_prism::Location::start_offset);
    let inline_documented = interpolations.iter().all(|interpolation| {
        let line_start = file.line_start(interpolation.location().start_offset());
        let line_end = file.line_end(interpolation.location().end_offset());
        comment_text(&source[line_start..line_end]).is_some()
    });
    if inline_documented {
        return;
    }
    if !heredoc {
        let literal = source.get(start..end).unwrap_or_default();
        if comment_text(literal).is_some() {
            return;
        }
        context.report_selector(node, "Add a comment block showing its appearance if interpolated.");
        return;
    }

    let call_start = node.location().start_offset();
    let call_end = node.location().end_offset().max(end);
    let comments = source
        .get(call_start..call_end)
        .unwrap_or_default()
        .lines()
        .filter_map(comment_text)
        .collect::<Vec<_>>()
        .join("\n");
    let normalized_comments = normalize_documentation(&comments).replace('\\', "");
    let documented = !comments.is_empty()
        && parts.iter().filter_map(Node::as_string_node).all(|part| {
            let required = normalize_documentation(file.at(&part.content_loc())).replace('\\', "");
            required.is_empty() || normalized_comments.contains(&required)
        });
    if !documented {
        context.report_selector(node, "Add a comment block showing its appearance if interpolated.");
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

fn normalize_documentation(source: &str) -> String {
    source
        .lines()
        .map(|line| comment_text(line).map_or(line, |_| {
            line.split_once(" #").map_or(line, |(code, _)| code)
        }))
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

fn module_function(
    node: &ruby_prism::ModuleNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let Some(statements) = node.body().and_then(|body| body.as_statements_node()) else {
        return;
    };
    let children = statements.body().iter().collect::<Vec<_>>();
    if children.len() < 2 {
        return;
    }
    let calls = children
        .iter()
        .filter_map(Node::as_call_node)
        .filter(|call| call.receiver().is_none())
        .collect::<Vec<_>>();
    let style = context
        .policy()
        .enforced_style("module_function")
        .to_string();
    if style == "module_function"
        && calls.iter().any(|call| call_name(call) == b"private")
    {
        return;
    }
    for call in calls {
        let extend_self = call_name(&call) == b"extend"
            && first_argument(&call).is_some_and(|argument| argument.as_self_node().is_some());
        let module_function = call_name(&call) == b"module_function"
            && argument_count(&call) == 0;
        let location = call.location();
        match (style.as_str(), extend_self, module_function) {
            ("module_function", true, _) => context.replace(
                "Use `module_function` instead of `extend self`.",
                &location,
                &location,
                "module_function",
            ),
            ("extend_self", _, true) => context.replace(
                "Use `extend self` instead of `module_function`.",
                &location,
                &location,
                "extend self",
            ),
            ("forbidden", true, _) | ("forbidden", _, true) => context.report(
                "Do not use `module_function` or `extend self`.",
                location,
            ),
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
