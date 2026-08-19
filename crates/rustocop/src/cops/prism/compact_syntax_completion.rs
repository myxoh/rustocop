use super::*;

define_rule!(WhileUntilModifierRule);

const WHILE_UNTIL_MODIFIER_MSG: &str =
    "Favor modifier `{keyword}` usage when having a single-line body.";

define_cops! {
    FileRead => "Style/FileRead" => source(file_read),
    FileWrite => "Style/FileWrite" => source(file_write),
    IfWithSemicolon => "Style/IfWithSemicolon" => source(if_with_semicolon),
    MethodDefParentheses => "Style/MethodDefParentheses" => source(method_def_parentheses),
    WhileUntilModifier => "Style/WhileUntilModifier" => node_rule_aliases(WhileUntilModifierRule, on_while => [as_while_node, as_until_node]),
}

fn file_read(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for (open_method, read_method, preferred) in [
        ("File.open(", ").read", "File.read"),
        ("File.open(", ").binread", "File.binread"),
    ] {
        let mut search = 0;
        while let Some(relative) = source[search..].find(open_method) {
            let start = search + relative;
            let offense_start = if source.get(start.saturating_sub(2)..start) == Some("::") {
                start - 2
            } else {
                start
            };
            let Some(close_relative) = source[start + open_method.len()..].find(read_method) else {
                break;
            };
            let call_end = start + open_method.len() + close_relative + read_method.len();
            let arguments =
                &source[start + open_method.len()..start + open_method.len() + close_relative];
            if arguments.contains(',') {
                search = call_end;
                continue;
            }
            context.replace(
                format!("Use `{preferred}`."),
                offense_start..call_end,
                offense_start..call_end,
                format!(
                    "{}{preferred}({arguments})",
                    if offense_start < start { "::" } else { "" }
                ),
            );
            search = call_end;
        }
    }
}

fn file_write(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let mut search = 0;
    while let Some(relative) = source[search..].find("File.open(") {
        let start = search + relative;
        let offense_start = if source.get(start.saturating_sub(2)..start) == Some("::") {
            start - 2
        } else {
            start
        };
        let Some(write_relative) = source[start..].find(").write(") else {
            break;
        };
        let write = start + write_relative;
        let Some(end_relative) = source[write + ").write(".len()..].find(')') else {
            break;
        };
        let end = write + ").write(".len() + end_relative + 1;
        let open_args = &source[start + "File.open(".len()..write];
        let Some((path, mode)) = open_args.rsplit_once(',') else {
            search = end;
            continue;
        };
        let mode = mode.trim().trim_matches(['\'', '"']);
        let preferred = if mode.contains('b') {
            "File.binwrite"
        } else {
            "File.write"
        };
        if !mode.starts_with('w') && !mode.starts_with('a') {
            search = end;
            continue;
        }
        let content = &source[write + ").write(".len()..end - 1];
        context.replace(
            format!("Use `{preferred}`."),
            offense_start..end,
            offense_start..end,
            format!(
                "{}{preferred}({}, {content})",
                if offense_start < start { "::" } else { "" },
                path.trim()
            ),
        );
        search = end;
    }
}

fn if_with_semicolon(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let code = line.trim();
        let (keyword, condition_start) = if code.starts_with("if ") {
            ("if", 3)
        } else if code.starts_with("unless ") {
            ("unless", 7)
        } else {
            continue;
        };
        if !code.ends_with(" end") {
            continue;
        }
        let Some((condition, rest)) = code[condition_start..].split_once(';') else {
            continue;
        };
        let Some(body) = rest.trim().strip_suffix(" end") else {
            continue;
        };
        let Some((mut truthy, mut falsey)) = body
            .split_once(" else ")
            .or_else(|| body.strip_prefix("else ").map(|falsey| ("", falsey)))
        else {
            continue;
        };
        if keyword == "unless" {
            std::mem::swap(&mut truthy, &mut falsey);
        }
        let replacement = format!(
            "{} ? {} : {}",
            condition.trim(),
            if truthy.trim().is_empty() {
                "nil"
            } else {
                truthy.trim()
            },
            if falsey.trim().is_empty() {
                "nil"
            } else {
                falsey.trim()
            }
        );
        let start = offset + line.find(code).unwrap_or(0);
        context.replace(
            format!(
                "Do not use `{keyword} {};` - use a ternary operator instead.",
                condition.trim()
            ),
            start..start + code.len(),
            start..start + code.len(),
            replacement,
        );
    }
}

fn method_def_parentheses(context: &mut CopContext<'_, '_>) {
    let style = context
        .policy()
        .enforced_style("require_parentheses")
        .to_string();
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        let Some(signature) = trimmed.strip_prefix("def ") else {
            continue;
        };
        let name_end = signature.find([' ', '(']).unwrap_or(signature.len());
        let raw_parameters = signature[name_end..].trim_start();
        let parameters = raw_parameters
            .split_once(';')
            .map_or(raw_parameters, |(parameters, _)| parameters)
            .trim_end();
        if parameters.is_empty() {
            continue;
        }
        let indent = line.len() - trimmed.len();
        if style == "require_parentheses" && !parameters.starts_with('(') {
            let gap = signature[name_end..].len() - raw_parameters.len();
            let start = offset + indent + "def ".len() + name_end + gap;
            context.replace(
                "Use def with parentheses when there are parameters.",
                start..start + parameters.len(),
                start - gap..start + parameters.len(),
                format!("({parameters})"),
            );
        } else if style != "require_parentheses"
            && parameters.starts_with('(')
            && parameters.ends_with(')')
        {
            let start = offset + line.find('(').unwrap_or(0);
            let end = offset + line.rfind(')').unwrap_or(line.len() - 1) + 1;
            context.replace(
                "Do not use parentheses for method parameters.",
                start..end,
                start..end,
                parameters.trim_matches(['(', ')']),
            );
        }
    }
}

struct ModifierForm {
    keyword: &'static str,
    offense: std::ops::Range<usize>,
    replacement: String,
}

impl WhileUntilModifierRule<'_, '_, '_> {
    fn on_while(&mut self, node: &Node<'_>) {
        let Some(form) = single_line_as_modifier(node, self) else {
            return;
        };
        let message = WHILE_UNTIL_MODIFIER_MSG.replace("{keyword}", form.keyword);
        add_offense!(self, form.offense, message: message, |corrector| {
            corrector.replace(node.location(), form.replacement);
        });
    }
}

fn single_line_as_modifier(
    node: &Node<'_>,
    context: &CopContext<'_, '_>,
) -> Option<ModifierForm> {
    let parts = if let Some(loop_node) = node.as_while_node() {
        Some((
            "while",
            loop_node.keyword_loc(),
            loop_node.closing_loc(),
            loop_node.predicate(),
            loop_node.statements(),
        ))
    } else {
        node.as_until_node().map(|loop_node| {
            (
            "until",
            loop_node.keyword_loc(),
            loop_node.closing_loc(),
            loop_node.predicate(),
            loop_node.statements(),
            )
        })
    };
    let Some((keyword, keyword_loc, Some(closing), predicate, Some(statements))) = parts else {
        return None;
    };
    let body_nodes = statements.body().iter().collect::<Vec<_>>();
    return_unless!(body_nodes.len() == 1, None);
    let body = &body_nodes[0];
    return_if!(non_eligible_body(body), None);
    let expression = context.source_file().node(node);
    return_if!(
        expression.lines().filter(|line| !line.trim().is_empty()).count() > 3,
        None
    );
    let body_source = context.source_file().node(body);
    return_if!(
        body_source.trim().is_empty()
            || body_source.contains('\n')
            || source_has_comment(body_source),
        None
    );
    let mut assignment = LocalAssignmentVisitor::default();
    assignment.visit(&predicate);
    return_if!(assignment.found, None);

    let source = context.source();
    let first_line_end = source[keyword_loc.start_offset()..]
        .find('\n')
        .map_or(source.len(), |offset| keyword_loc.start_offset() + offset);
    let header_tail = &source[predicate.location().end_offset()..first_line_end];
    let first_line_comment = header_tail
        .find('#')
        .map(|offset| header_tail[offset..].trim_end());
    let last_line_end = source[closing.end_offset()..]
        .find('\n')
        .map_or(source.len(), |offset| closing.end_offset() + offset);
    let code_after = &source[closing.end_offset()..last_line_end];
    return_if!(
        code_after.trim_start().starts_with('#')
            || first_line_comment.is_some() && !code_after.trim().is_empty(),
        None
    );

    let condition = context.source_file().node(&predicate);
    let replacement = to_modifier_form(
        body_source,
        keyword,
        condition,
        parenthesize_modifier(context.parent()),
        first_line_comment,
    );

    let line_start = context.source_file().line_start(keyword_loc.start_offset());
    let code_before = &source[line_start..keyword_loc.start_offset()];
    let line_length_enabled = context
        .related_config_value("Layout/LineLength", "Enabled")
        != Some("false");
    let maximum = context
        .related_config_value("Layout/LineLength", "Max")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(120);
    return_if!(
        line_length_enabled
            && format!("{code_before}{replacement}{code_after}").chars().count() > maximum,
        None
    );
    Some(ModifierForm {
        keyword,
        offense: keyword_loc.start_offset()..keyword_loc.end_offset(),
        replacement,
    })
}

fn non_eligible_body(node: &Node<'_>) -> bool {
    node.as_if_node().is_some()
        || node.as_unless_node().is_some()
        || node.as_while_node().is_some()
        || node.as_until_node().is_some()
        || node.as_case_node().is_some()
        || node.as_case_match_node().is_some()
}

fn to_modifier_form(
    body: &str,
    keyword: &str,
    condition: &str,
    parenthesize: bool,
    comment: Option<&str>,
) -> String {
    let mut replacement = format!("{body} {keyword} {condition}");
    if parenthesize {
        replacement = format!("({replacement})");
    }
    if let Some(comment) = comment {
        replacement.push(' ');
        replacement.push_str(comment);
    }
    replacement
}

fn source_has_comment(source: &str) -> bool {
    source
        .lines()
        .any(|line| line.trim_start().starts_with('#') || line.contains(" #"))
}

fn parenthesize_modifier(parent: Option<&Node<'_>>) -> bool {
    parent.is_some_and(|parent| {
        parent.as_local_variable_write_node().is_some()
            || parent.as_instance_variable_write_node().is_some()
            || parent.as_class_variable_write_node().is_some()
            || parent.as_constant_write_node().is_some()
            || parent.as_constant_path_write_node().is_some()
            || parent.as_array_node().is_some()
            || parent.as_assoc_node().is_some()
            || parent.as_and_node().is_some()
            || parent.as_or_node().is_some()
            || parent.as_call_node().is_some()
    })
}

#[derive(Default)]
struct LocalAssignmentVisitor {
    found: bool,
}

impl<'pr> Visit<'pr> for LocalAssignmentVisitor {
    fn visit_local_variable_write_node(&mut self, _node: &ruby_prism::LocalVariableWriteNode<'pr>) {
        self.found = true;
    }

    fn visit_local_variable_and_write_node(
        &mut self,
        _node: &ruby_prism::LocalVariableAndWriteNode<'pr>,
    ) {
        self.found = true;
    }

    fn visit_local_variable_or_write_node(
        &mut self,
        _node: &ruby_prism::LocalVariableOrWriteNode<'pr>,
    ) {
        self.found = true;
    }

    fn visit_local_variable_operator_write_node(
        &mut self,
        _node: &ruby_prism::LocalVariableOperatorWriteNode<'pr>,
    ) {
        self.found = true;
    }
}
