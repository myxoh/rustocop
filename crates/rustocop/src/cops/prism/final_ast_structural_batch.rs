use super::catalog_cop::custom;
use super::*;
use std::collections::HashMap;

mod registry;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    let mut cops: Vec<Box<dyn Cop>> = vec![
        Box::new(SafeNavigation),
        Box::new(SelectByKind),
        Box::new(SelectByRange),
        Box::new(RedundantTypeConversion),
        Box::new(ConditionalAssignment),
        Box::new(Debugger),
        custom("Lint/UselessAccessModifier", useless_access_modifier),
        custom("Style/ArgumentsForwarding", arguments_forwarding),
        Box::new(Void),
        custom("Lint/LiteralInInterpolation", literal_in_interpolation),
    ];
    cops.extend(registry::cops());
    cops
}

fn literal_in_interpolation(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for (opening, closing) in interpolation_ranges(source) {
        let Some((start, end)) = final_interpolation_expression(source, opening + 2, closing)
        else {
            continue;
        };
        let expression = &source[start..end];
        if !literal_interpolation_expression(expression) {
            continue;
        }
        if array_percent_interpolation(source, opening, expression) {
            continue;
        }
        if heredoc_trailing_space_interpolation(source, closing, expression)
            || regexp_array_interpolation(source, opening, expression)
        {
            continue;
        }
        context.replace(
            "Literal interpolation detected.",
            start..end,
            opening..closing + 1,
            interpolation_literal_value(expression),
        );
    }
}

fn interpolation_ranges(source: &str) -> Vec<(usize, usize)> {
    let bytes = source.as_bytes();
    let mut ranges = Vec::new();
    let mut index = 0;
    while index + 1 < bytes.len() {
        if bytes[index] == b'#' && bytes[index + 1] == b'{' {
            if let Some(closing) = interpolation_closing(bytes, index + 2) {
                ranges.push((index, closing));
            }
        }
        index += 1;
    }
    ranges
}

fn interpolation_closing(bytes: &[u8], mut index: usize) -> Option<usize> {
    let mut depth = 1usize;
    let mut quote = None;
    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(delimiter) = quote {
            if byte == b'\\' {
                index += 2;
                continue;
            }
            if byte == delimiter {
                quote = None;
            }
        } else {
            match byte {
                b'\'' | b'"' | b'`' => quote = Some(byte),
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(index);
                    }
                }
                _ => {}
            }
        }
        index += 1;
    }
    None
}

fn final_interpolation_expression(
    source: &str,
    content_start: usize,
    content_end: usize,
) -> Option<(usize, usize)> {
    let bytes = source.as_bytes();
    let mut start = content_start;
    let mut quote = None;
    let mut nesting = 0usize;
    let mut index = content_start;
    while index < content_end {
        let byte = bytes[index];
        if let Some(delimiter) = quote {
            if byte == b'\\' {
                index += 2;
                continue;
            }
            if byte == delimiter {
                quote = None;
            }
        } else {
            match byte {
                b'\'' | b'"' | b'`' => quote = Some(byte),
                b'(' | b'[' | b'{' => nesting += 1,
                b')' | b']' | b'}' => nesting = nesting.saturating_sub(1),
                b';' if nesting == 0 => start = index + 1,
                _ => {}
            }
        }
        index += 1;
    }
    while start < content_end && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    let mut end = content_end;
    while end > start && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    (start < end).then_some((start, end))
}

fn literal_interpolation_expression(expression: &str) -> bool {
    let expression = expression.trim();
    if expression.contains("#{") || expression.starts_with('`') {
        return false;
    }
    if matches!(expression, "nil" | "true" | "false") {
        return true;
    }
    if expression.starts_with(['\'', '"']) && expression.ends_with(expression.as_bytes()[0] as char)
    {
        return true;
    }
    if expression.starts_with(':') {
        return expression.len() > 1;
    }
    if expression.starts_with('%') {
        return expression.starts_with("%(")
            || matches!(
                expression.as_bytes().get(1),
                Some(b'q' | b'Q' | b'w' | b'i' | b'I')
            );
    }
    if (expression.starts_with('[') && expression.ends_with(']'))
        || (expression.starts_with('{') && expression.ends_with('}'))
    {
        return true;
    }
    let numeric = expression
        .bytes()
        .all(|byte| byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'+' | b'-' | b'e' | b'E' | b'x' | b'o' | b'b' | b'a'..=b'f' | b'A'..=b'F'));
    numeric && expression.bytes().any(|byte| byte.is_ascii_digit())
}

fn array_percent_interpolation(source: &str, opening: usize, expression: &str) -> bool {
    let line_start = source[..opening].rfind('\n').map_or(0, |at| at + 1);
    let prefix = source[line_start..opening].trim_start();
    if !(prefix.starts_with("%W[") || prefix.starts_with("%I[")) {
        return false;
    }
    let value = interpolation_literal_value(expression);
    value.is_empty() || value.bytes().any(|byte| byte.is_ascii_whitespace())
}

fn heredoc_trailing_space_interpolation(source: &str, closing: usize, expression: &str) -> bool {
    let value = interpolation_literal_value(expression);
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_whitespace()) {
        return false;
    }
    let line_end = source[closing + 1..]
        .find('\n')
        .map_or(source.len(), |at| closing + 1 + at);
    source[closing + 1..line_end].trim().is_empty()
        && source[..closing].lines().any(|line| line.contains("<<"))
}

fn regexp_array_interpolation(source: &str, opening: usize, expression: &str) -> bool {
    if !(expression.starts_with('[') || expression.starts_with("%w")) {
        return false;
    }
    let line_start = source[..opening].rfind('\n').map_or(0, |at| at + 1);
    source[line_start..opening].trim_start().starts_with('/')
}

fn interpolation_literal_value(expression: &str) -> String {
    let expression = expression.trim();
    if expression == "nil" {
        return String::new();
    }
    if expression.len() >= 2
        && matches!(expression.as_bytes()[0], b'\'' | b'"')
        && expression.as_bytes()[expression.len() - 1] == expression.as_bytes()[0]
    {
        return expression[1..expression.len() - 1].to_string();
    }
    expression.to_string()
}

struct SelectByRange;

struct SelectByKind;

struct SafeNavigation;

struct RedundantTypeConversion;

struct ConditionalAssignment;

struct Void;

struct Debugger;

impl Cop for Debugger {
    fn name(&self) -> &'static str {
        "Lint/Debugger"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        let debugger_methods = debugger_configured_entries(&cop_context, "DebuggerMethods");
        let debugger_requires = debugger_configured_entries(&cop_context, "DebuggerRequires");
        let chained_name = debugger_chained_name(&call, source);
        let debugger_method = debugger_methods
            .iter()
            .any(|method| method == &chained_name);
        let debugger_require = call.name().as_slice() == b"require"
            && call.arguments().is_some_and(|arguments| {
                let values = arguments.arguments().iter().collect::<Vec<_>>();
                values.len() == 1
                    && values[0].as_string_node().is_some_and(|string| {
                        let value = String::from_utf8_lossy(string.unescaped()).to_string();
                        debugger_requires.contains(&value)
                    })
            });
        if !debugger_method && !debugger_require {
            return;
        }
        let no_arguments = call
            .arguments()
            .is_none_or(|arguments| arguments.arguments().is_empty());
        if no_arguments {
            if let Some(parent_call) = ancestors.iter().rev().find_map(Node::as_call_node) {
                let inside_parent_block = parent_call.block().is_some_and(|block| {
                    block.location().start_offset() <= call.location().start_offset()
                        && call.location().end_offset() <= block.location().end_offset()
                });
                let inside_begin_or_lambda = ancestors
                    .iter()
                    .rev()
                    .take_while(|ancestor| {
                        ancestor.location().start_offset() >= parent_call.location().start_offset()
                    })
                    .any(|ancestor| {
                        ancestor.as_begin_node().is_some() || ancestor.as_lambda_node().is_some()
                    });
                if !inside_parent_block && !inside_begin_or_lambda {
                    return;
                }
            }
        }
        let start = call.location().start_offset();
        let mut end = call.block().map_or(call.location().end_offset(), |block| {
            block.location().start_offset()
        });
        while end > start && source.as_bytes()[end - 1].is_ascii_whitespace() {
            end -= 1;
        }
        let range = start..end;
        cop_context.report(
            format!("Remove debugger entry point `{}`.", &source[range.clone()]),
            range,
        );
    }
}

fn debugger_configured_entries(context: &CopContext<'_, '_>, key: &str) -> Vec<String> {
    let configured = context.config_values(key);
    if !configured.is_empty() {
        return configured.to_vec();
    }
    context
        .config_map(key)
        .into_iter()
        .flat_map(|groups| groups.values())
        .filter(|value| !matches!(value.as_str(), "" | "~" | "nil" | "false"))
        .flat_map(|value| value.lines().map(str::to_string))
        .collect()
}

fn debugger_chained_name(call: &CallNode<'_>, source: &str) -> String {
    let name = String::from_utf8_lossy(call.name().as_slice()).to_string();
    let Some(receiver) = call.receiver() else {
        return name;
    };
    let receiver = if let Some(receiver_call) = receiver.as_call_node() {
        debugger_chained_name(&receiver_call, source)
    } else {
        source_at(source, &receiver.location())
            .trim_start_matches("::")
            .to_string()
    };
    format!("{receiver}.{name}")
}

impl Cop for Void {
    fn name(&self) -> &'static str {
        "Lint/Void"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(statements) = node.as_statements_node() else {
            return;
        };
        let body = statements.body().iter().collect::<Vec<_>>();
        let direct_parent = ancestors.last();
        let enclosing_definition = direct_parent.and_then(Node::as_def_node);
        let ensure_body = ancestors.iter().rev().any(|ancestor| {
            ancestor.as_ensure_node().is_some_and(|ensure_node| {
                ensure_node.statements().is_some_and(|body| {
                    body.location().start_offset() == node.location().start_offset()
                        && body.location().end_offset() == node.location().end_offset()
                })
            })
        }) || source[..node.location().start_offset()]
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .is_some_and(|line| line.trim() == "ensure");
        let all_expressions = enclosing_definition.as_ref().is_some_and(|definition| {
            definition.name().as_slice() == b"initialize"
                || definition.name().as_slice().ends_with(b"=")
        }) || direct_parent.is_some_and(|parent| {
            parent.as_for_node().is_some() || parent.as_ensure_node().is_some()
        }) || ensure_body
            || direct_parent.and_then(Node::as_block_node).is_some()
                && ancestors.iter().rev().any(|ancestor| {
                    ancestor
                        .as_call_node()
                        .is_some_and(|call| call.name().as_slice() == b"tap")
                });
        let correctable = !enclosing_definition
            .as_ref()
            .is_some_and(|definition| definition.name().as_slice().ends_with(b"="));
        let count = if all_expressions {
            body.len()
        } else {
            body.len().saturating_sub(1)
        };
        for expression in body.iter().take(count) {
            check_void_expression(
                expression,
                ancestors,
                source,
                context,
                self.name(),
                correctable,
            );
        }
    }
}

fn check_void_expression(
    node: &Node<'_>,
    ancestors: &[Node<'_>],
    source: &str,
    context: &mut Context,
    cop: &'static str,
    correctable: bool,
) {
    if let Some(parentheses) = node.as_parentheses_node() {
        if let Some(expression) = parentheses.body().and_then(single_expression) {
            check_void_expression(&expression, ancestors, source, context, cop, correctable);
        }
        return;
    }
    if let Some(conditional) = node.as_if_node() {
        if let Some(statements) = conditional.statements() {
            check_void_branch_tail(&statements, ancestors, source, context, cop);
        }
        if let Some(subsequent) = conditional.subsequent() {
            if let Some(else_node) = subsequent.as_else_node() {
                if let Some(statements) = else_node.statements() {
                    check_void_branch_tail(&statements, ancestors, source, context, cop);
                }
            } else {
                check_void_expression(&subsequent, ancestors, source, context, cop, false);
            }
        }
        return;
    }
    if let Some(conditional) = node.as_unless_node() {
        if let Some(statements) = conditional.statements() {
            check_void_branch_tail(&statements, ancestors, source, context, cop);
        }
        if let Some(else_node) = conditional.else_clause() {
            if let Some(statements) = else_node.statements() {
                check_void_branch_tail(&statements, ancestors, source, context, cop);
            }
        }
        return;
    }
    if let Some(case_node) = node.as_case_node() {
        for branch in case_node.conditions().iter() {
            if let Some(statements) = branch.as_when_node().and_then(|branch| branch.statements()) {
                check_void_branch_tail(&statements, ancestors, source, context, cop);
            }
        }
        if let Some(statements) = case_node
            .else_clause()
            .and_then(|branch| branch.statements())
        {
            check_void_branch_tail(&statements, ancestors, source, context, cop);
        }
        return;
    }
    if let Some(case_node) = node.as_case_match_node() {
        for branch in case_node.conditions().iter() {
            if let Some(statements) = branch.as_in_node().and_then(|branch| branch.statements()) {
                check_void_branch_tail(&statements, ancestors, source, context, cop);
            }
        }
        if let Some(statements) = case_node
            .else_clause()
            .and_then(|branch| branch.statements())
        {
            check_void_branch_tail(&statements, ancestors, source, context, cop);
        }
        return;
    }

    let mut cop_context = context.cop_context(cop, source, ancestors);
    let range = node.location().start_offset()..node.location().end_offset();
    let expression = &source[range.clone()];
    if let Some(call) = node.as_call_node() {
        let method = call.name().as_slice();
        const OPERATORS: &[&[u8]] = &[
            b"*", b"/", b"%", b"+", b"-", b"==", b"===", b"!=", b"<", b">", b"<=", b">=", b"<=>",
            b"+@", b"-@", b"~", b"!",
        ];
        if OPERATORS.contains(&method) {
            let binary = !matches!(method, b"+@" | b"-@" | b"~" | b"!");
            if binary
                && call.call_operator_loc().is_some()
                && call
                    .arguments()
                    .is_none_or(|arguments| arguments.arguments().is_empty())
            {
                return;
            }
            if let Some(selector) = call.message_loc() {
                let selector = selector.start_offset()..selector.end_offset();
                let method = String::from_utf8_lossy(method);
                report_void(
                    &mut cop_context,
                    format!("Operator `{method}` used in void context."),
                    selector.clone(),
                    source[selector].to_string(),
                    correctable,
                );
            }
            return;
        }
        if cop_context.config_bool("CheckForMethodsWithNoSideEffects", false) {
            let suggestion = match method {
                b"collect" | b"map" => Some("each".to_string()),
                b"capitalize" | b"chomp" | b"chop" | b"compact" | b"delete_prefix"
                | b"delete_suffix" | b"downcase" | b"encode" | b"flatten" | b"gsub" | b"lstrip"
                | b"merge" | b"next" | b"reject" | b"reverse" | b"rotate" | b"rstrip"
                | b"scrub" | b"select" | b"shuffle" | b"slice" | b"sort" | b"sort_by"
                | b"squeeze" | b"strip" | b"sub" | b"succ" | b"swapcase" | b"tr" | b"tr_s"
                | b"transform_values" | b"unicode_normalize" | b"uniq" | b"upcase" => {
                    Some(format!("{}!", String::from_utf8_lossy(method)))
                }
                _ => None,
            };
            if let Some(suggestion) = suggestion {
                let method = String::from_utf8_lossy(method);
                cop_context.replace(
                    format!(
                        "Method `#{method}` used in void context. Did you mean `#{suggestion}`?"
                    ),
                    range.clone(),
                    range.clone(),
                    suggestion,
                );
                return;
            }
        }
    }
    if void_literal(node) || frozen_void_literal(node) {
        report_void(
            &mut cop_context,
            format!("Literal `{expression}` used in void context."),
            range.clone(),
            expression.to_string(),
            correctable,
        );
    } else if void_variable(node) {
        report_void(
            &mut cop_context,
            format!("Variable `{expression}` used in void context."),
            range.clone(),
            expression.to_string(),
            correctable,
        );
    } else if node.as_constant_read_node().is_some() || node.as_constant_path_node().is_some() {
        report_void(
            &mut cop_context,
            format!("Constant `{expression}` used in void context."),
            range.clone(),
            expression.to_string(),
            correctable,
        );
    } else if node.as_self_node().is_some() {
        report_void(
            &mut cop_context,
            "`self` used in void context.",
            range.clone(),
            expression.to_string(),
            correctable,
        );
    } else if node.as_defined_node().is_some()
        || node.as_lambda_node().is_some()
        || void_proc_expression(node)
    {
        report_void(
            &mut cop_context,
            format!("`{expression}` used in void context."),
            range.clone(),
            expression.to_string(),
            correctable,
        );
    } else if node.as_source_encoding_node().is_some() {
        report_void(
            &mut cop_context,
            format!("Variable `{expression}` used in void context."),
            range,
            expression.to_string(),
            correctable,
        );
    }
}

fn report_void(
    context: &mut CopContext<'_, '_>,
    message: impl Into<String>,
    range: std::ops::Range<usize>,
    source: String,
    correctable: bool,
) {
    if correctable {
        context.replace(message, range.clone(), range, source);
    } else {
        context.report(message, range);
    }
}

fn void_proc_expression(node: &Node<'_>) -> bool {
    let Some(call) = node.as_call_node() else {
        return false;
    };
    call.block().is_some()
        && (matches!(call.name().as_slice(), b"lambda" | b"proc")
            || call.name().as_slice() == b"new"
                && call.receiver().is_some_and(|receiver| {
                    receiver
                        .as_constant_read_node()
                        .is_some_and(|constant| constant.name().as_slice() == b"Proc")
                }))
}

fn frozen_void_literal(node: &Node<'_>) -> bool {
    node.as_call_node().is_some_and(|call| {
        call.name().as_slice() == b"freeze"
            && call
                .arguments()
                .is_none_or(|arguments| arguments.arguments().is_empty())
            && call.receiver().as_ref().is_some_and(void_literal)
    })
}

fn check_void_branch_tail(
    statements: &ruby_prism::StatementsNode<'_>,
    ancestors: &[Node<'_>],
    source: &str,
    context: &mut Context,
    cop: &'static str,
) {
    if let Some(last) = statements.body().last() {
        check_void_expression(&last, ancestors, source, context, cop, false);
    }
}

fn void_variable(node: &Node<'_>) -> bool {
    node.as_local_variable_read_node().is_some()
        || node.as_instance_variable_read_node().is_some()
        || node.as_class_variable_read_node().is_some()
        || node.as_global_variable_read_node().is_some()
        || node.as_back_reference_read_node().is_some()
        || node.as_numbered_reference_read_node().is_some()
}

fn void_literal(node: &Node<'_>) -> bool {
    node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_rational_node().is_some()
        || node.as_imaginary_node().is_some()
        || node.as_string_node().is_some()
        || node.as_interpolated_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_interpolated_symbol_node().is_some()
        || node.as_regular_expression_node().is_some()
        || node.as_interpolated_regular_expression_node().is_some()
        || node.as_array_node().is_some_and(|array| {
            array
                .elements()
                .iter()
                .all(|element| void_literal(&element) || frozen_void_literal(&element))
        })
        || node.as_hash_node().is_some_and(|hash| {
            hash.elements().iter().all(|element| {
                element
                    .as_assoc_node()
                    .is_some_and(|pair| void_literal(&pair.key()) && void_literal(&pair.value()))
            })
        })
}

struct ConditionalBranch<'pr> {
    tail: Node<'pr>,
    statements: usize,
}

struct ConditionalAssignmentParts<'pr> {
    lhs: String,
    kind: &'static str,
    value: Node<'pr>,
}

impl Cop for ConditionalAssignment {
    fn name(&self) -> &'static str {
        "Style/ConditionalAssignment"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        let style = cop_context.policy().enforced_style("assign_to_condition");
        let single_line_only = cop_context.config_bool("SingleLineConditionsOnly", true);
        let include_ternary = cop_context.config_bool("IncludeTernaryExpressions", true);

        if style == "assign_inside_condition" {
            let Some(assignment) = conditional_assignment_parts(node, source) else {
                return;
            };
            let Some(branches) = conditional_assignment_branches(&assignment.value, true) else {
                return;
            };
            if !include_ternary && conditional_ternary(&assignment.value)
                || single_line_only && branches.iter().any(|branch| branch.statements > 1)
            {
                return;
            }
            let range = node.location().start_offset()..node.location().end_offset();
            cop_context.replace(
                "Assign variables inside of conditionals.",
                range.clone(),
                range.clone(),
                source[range].to_string(),
            );
            return;
        }

        if node.as_if_node().is_some_and(|conditional| {
            conditional
                .if_keyword_loc()
                .is_some_and(|keyword| keyword.as_slice() == b"elsif")
        }) {
            return;
        }
        if !include_ternary && conditional_ternary(node) {
            return;
        }
        let Some(branches) = conditional_assignment_branches(node, false) else {
            return;
        };
        if single_line_only && branches.iter().any(|branch| branch.statements > 1) {
            return;
        }
        let assignments = branches
            .iter()
            .map(|branch| conditional_assignment_parts(&branch.tail, source))
            .collect::<Option<Vec<_>>>();
        let Some(assignments) = assignments else {
            return;
        };
        let Some(first) = assignments.first() else {
            return;
        };
        if first.kind == "multi" {
            return;
        }
        if assignments
            .iter()
            .any(|assignment| assignment.kind != first.kind || assignment.lhs != first.lhs)
        {
            return;
        }
        if let Some(maximum) = cop_context
            .related_config_value("Layout/LineLength", "Max")
            .and_then(|maximum| maximum.parse::<usize>().ok())
        {
            let longest = source[range_for_node(node)]
                .lines()
                .map(|line| line.replacen(&first.lhs, "", 1).len())
                .max()
                .unwrap_or(0);
            if longest + first.lhs.len() > maximum {
                return;
            }
        }
        let range = node.location().start_offset()..node.location().end_offset();
        cop_context.replace(
            "Use the return of the conditional for variable assignment and comparison.",
            range.clone(),
            range.clone(),
            source[range].to_string(),
        );
    }
}

fn conditional_assignment_branches<'pr>(
    node: &Node<'pr>,
    allow_missing_else: bool,
) -> Option<Vec<ConditionalBranch<'pr>>> {
    if let Some(parentheses) = node.as_parentheses_node() {
        return parentheses
            .body()
            .and_then(single_expression)
            .and_then(|expression| {
                conditional_assignment_branches(&expression, allow_missing_else)
            });
    }
    if let Some(conditional) = node.as_if_node() {
        let mut branches = Vec::new();
        branches.push(conditional_statement_branch(conditional.statements())?);
        let Some(mut subsequent) = conditional.subsequent() else {
            return (allow_missing_else && branches.len() > 1).then_some(branches);
        };
        loop {
            if let Some(elsif) = subsequent.as_if_node() {
                branches.push(conditional_statement_branch(elsif.statements())?);
                let Some(next) = elsif.subsequent() else {
                    return (allow_missing_else && branches.len() > 1).then_some(branches);
                };
                subsequent = next;
                continue;
            }
            let else_node = subsequent.as_else_node()?;
            branches.push(conditional_statement_branch(else_node.statements())?);
            return Some(branches);
        }
    }
    if let Some(conditional) = node.as_unless_node() {
        let first = conditional_statement_branch(conditional.statements())?;
        return if let Some(else_node) = conditional.else_clause() {
            Some(vec![
                first,
                conditional_statement_branch(else_node.statements())?,
            ])
        } else {
            allow_missing_else.then_some(vec![first])
        };
    }
    if let Some(case_node) = node.as_case_node() {
        let mut branches = case_node
            .conditions()
            .iter()
            .map(|branch| {
                branch
                    .as_when_node()
                    .and_then(|branch| conditional_statement_branch(branch.statements()))
            })
            .collect::<Option<Vec<_>>>()?;
        branches.push(conditional_statement_branch(
            case_node.else_clause()?.statements(),
        )?);
        return Some(branches);
    }
    if let Some(case_node) = node.as_case_match_node() {
        let mut branches = case_node
            .conditions()
            .iter()
            .map(|branch| {
                branch
                    .as_in_node()
                    .and_then(|branch| conditional_statement_branch(branch.statements()))
            })
            .collect::<Option<Vec<_>>>()?;
        branches.push(conditional_statement_branch(
            case_node.else_clause()?.statements(),
        )?);
        return Some(branches);
    }
    None
}

fn range_for_node(node: &Node<'_>) -> std::ops::Range<usize> {
    node.location().start_offset()..node.location().end_offset()
}

fn conditional_statement_branch<'pr>(
    statements: Option<ruby_prism::StatementsNode<'pr>>,
) -> Option<ConditionalBranch<'pr>> {
    let body = statements?.body();
    Some(ConditionalBranch {
        tail: body.last()?,
        statements: body.len(),
    })
}

fn conditional_ternary(node: &Node<'_>) -> bool {
    if let Some(expression) = node
        .as_parentheses_node()
        .and_then(|parentheses| parentheses.body().and_then(single_expression))
    {
        return expression
            .as_if_node()
            .is_some_and(|conditional| conditional.if_keyword_loc().is_none());
    }
    node.as_if_node()
        .is_some_and(|conditional| conditional.if_keyword_loc().is_none())
}

fn conditional_assignment_parts<'pr>(
    node: &Node<'pr>,
    source: &str,
) -> Option<ConditionalAssignmentParts<'pr>> {
    let (kind, value) = if let Some(write) = node.as_local_variable_write_node() {
        ("local", write.value())
    } else if let Some(write) = node.as_instance_variable_write_node() {
        ("instance", write.value())
    } else if let Some(write) = node.as_class_variable_write_node() {
        ("class", write.value())
    } else if let Some(write) = node.as_global_variable_write_node() {
        ("global", write.value())
    } else if let Some(write) = node.as_constant_write_node() {
        ("constant", write.value())
    } else if let Some(write) = node.as_constant_path_write_node() {
        ("constant_path", write.value())
    } else if let Some(write) = node.as_local_variable_operator_write_node() {
        ("local_operator", write.value())
    } else if let Some(write) = node.as_instance_variable_operator_write_node() {
        ("instance_operator", write.value())
    } else if let Some(write) = node.as_class_variable_operator_write_node() {
        ("class_operator", write.value())
    } else if let Some(write) = node.as_global_variable_operator_write_node() {
        ("global_operator", write.value())
    } else if let Some(write) = node.as_constant_operator_write_node() {
        ("constant_operator", write.value())
    } else if let Some(write) = node.as_constant_path_operator_write_node() {
        ("constant_path_operator", write.value())
    } else if let Some(write) = node.as_local_variable_or_write_node() {
        ("local_or", write.value())
    } else if let Some(write) = node.as_local_variable_and_write_node() {
        ("local_and", write.value())
    } else if let Some(write) = node.as_instance_variable_or_write_node() {
        ("instance_or", write.value())
    } else if let Some(write) = node.as_instance_variable_and_write_node() {
        ("instance_and", write.value())
    } else if let Some(write) = node.as_class_variable_or_write_node() {
        ("class_or", write.value())
    } else if let Some(write) = node.as_class_variable_and_write_node() {
        ("class_and", write.value())
    } else if let Some(write) = node.as_global_variable_or_write_node() {
        ("global_or", write.value())
    } else if let Some(write) = node.as_global_variable_and_write_node() {
        ("global_and", write.value())
    } else if let Some(write) = node.as_constant_or_write_node() {
        ("constant_or", write.value())
    } else if let Some(write) = node.as_constant_and_write_node() {
        ("constant_and", write.value())
    } else if let Some(write) = node.as_constant_path_or_write_node() {
        ("constant_path_or", write.value())
    } else if let Some(write) = node.as_constant_path_and_write_node() {
        ("constant_path_and", write.value())
    } else if let Some(write) = node.as_multi_write_node() {
        ("multi", write.value())
    } else if let Some(write) = node.as_index_operator_write_node() {
        ("index_operator", write.value())
    } else if let Some(write) = node.as_call_operator_write_node() {
        ("call_operator", write.value())
    } else if let Some(call) = node.as_call_node() {
        let name = call.name().as_slice();
        if !(name.ends_with(b"=")
            || matches!(name, b"<<" | b"=~" | b"!~" | b"<=>" | b"<" | b">" | b"!="))
        {
            return None;
        }
        let value = call.arguments()?.arguments().last()?;
        ("call", value)
    } else {
        return None;
    };
    let start = node.location().start_offset();
    let value_start = value.location().start_offset();
    if value_start < start {
        return None;
    }
    Some(ConditionalAssignmentParts {
        lhs: source[start..value_start]
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "),
        kind,
        value,
    })
}

impl Cop for RedundantTypeConversion {
    fn name(&self) -> &'static str {
        "Lint/RedundantTypeConversion"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        let method = call.name().as_slice();
        if !matches!(
            method,
            b"to_s"
                | b"to_sym"
                | b"to_i"
                | b"to_f"
                | b"to_d"
                | b"to_r"
                | b"to_c"
                | b"to_a"
                | b"to_h"
                | b"to_set"
        ) {
            return;
        }
        if call
            .arguments()
            .is_some_and(|arguments| !arguments.arguments().is_empty())
        {
            return;
        }
        if matches!(method, b"to_h" | b"to_set") && call.block().is_some() {
            return;
        }
        let Some(receiver) = call.receiver().map(unwrap_redundant_conversion_parentheses) else {
            return;
        };
        if !redundant_conversion_receiver(method, &receiver, source) {
            return;
        }
        let Some(selector) = call.message_loc() else {
            return;
        };
        let selector = selector.start_offset()..selector.end_offset();
        let method = String::from_utf8_lossy(method);
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        cop_context.replace(
            format!("Redundant `{method}` detected."),
            selector.clone(),
            selector,
            method.to_string(),
        );
    }
}

fn unwrap_redundant_conversion_parentheses(mut node: Node<'_>) -> Node<'_> {
    loop {
        let Some(parentheses) = node.as_parentheses_node() else {
            return node;
        };
        let Some(inner) = parentheses.body().and_then(single_expression) else {
            return node;
        };
        node = inner;
    }
}

fn redundant_conversion_receiver(method: &[u8], receiver: &Node<'_>, source: &str) -> bool {
    let literal = match method {
        b"to_s" => {
            receiver.as_string_node().is_some() || receiver.as_interpolated_string_node().is_some()
        }
        b"to_sym" => {
            receiver.as_symbol_node().is_some() || receiver.as_interpolated_symbol_node().is_some()
        }
        b"to_i" => receiver.as_integer_node().is_some(),
        b"to_f" => receiver.as_float_node().is_some(),
        b"to_r" => receiver.as_rational_node().is_some(),
        b"to_c" => receiver.as_imaginary_node().is_some(),
        b"to_a" => receiver.as_array_node().is_some(),
        b"to_h" => receiver.as_hash_node().is_some(),
        _ => false,
    };
    if literal {
        return true;
    }
    let Some(receiver_call) = receiver.as_call_node() else {
        return false;
    };
    if receiver_call.name().as_slice() == method {
        return true;
    }
    if method == b"to_s" && matches!(receiver_call.name().as_slice(), b"inspect" | b"to_json") {
        return true;
    }
    if source_at(source, &receiver.location()).contains("exception: false") {
        return false;
    }
    redundant_conversion_constructor(method, &receiver_call)
}

fn redundant_conversion_constructor(method: &[u8], call: &CallNode<'_>) -> bool {
    let (class, kernel_method) = match method {
        b"to_s" => (b"String".as_slice(), b"String".as_slice()),
        b"to_i" => (b"Integer".as_slice(), b"Integer".as_slice()),
        b"to_f" => (b"Float".as_slice(), b"Float".as_slice()),
        b"to_d" => (b"BigDecimal".as_slice(), b"BigDecimal".as_slice()),
        b"to_r" => (b"Rational".as_slice(), b"Rational".as_slice()),
        b"to_c" => (b"Complex".as_slice(), b"Complex".as_slice()),
        b"to_a" => (b"Array".as_slice(), b"Array".as_slice()),
        b"to_h" => (b"Hash".as_slice(), b"Hash".as_slice()),
        b"to_set" => (b"Set".as_slice(), b"Set".as_slice()),
        _ => return false,
    };
    let name = call.name().as_slice();
    if name == kernel_method {
        return call.receiver().is_none() || root_constant(call.receiver(), b"Kernel");
    }
    let allowed_constructor = match method {
        b"to_s" => name == b"new",
        b"to_a" | b"to_h" | b"to_set" => matches!(name, b"new" | b"[]"),
        _ => false,
    };
    allowed_constructor && root_constant(call.receiver(), class)
}

enum RangeBlockParameter {
    Named(Vec<u8>),
    Numbered,
    It,
}

struct RangeSelection {
    pattern: String,
    negated: bool,
}

struct KindSelection<'pr> {
    class: Node<'pr>,
    negated: bool,
}

impl Cop for SelectByKind {
    fn name(&self) -> &'static str {
        "Style/SelectByKind"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        let original = call.name().as_slice();
        if !matches!(original, b"select" | b"filter" | b"find_all" | b"reject") {
            return;
        }
        let Some(block) = call.block().and_then(|block| block.as_block_node()) else {
            return;
        };
        if call.receiver().as_ref().is_some_and(range_hash_receiver) {
            return;
        }
        let Some(parameter) = range_block_parameter(&block) else {
            return;
        };
        let Some(body) = block.body().and_then(single_expression) else {
            return;
        };
        let Some(selection) = kind_selection(body, &parameter) else {
            return;
        };
        let selecting = matches!(original, b"select" | b"filter" | b"find_all");
        let replacement = if selecting == selection.negated {
            "grep_v"
        } else {
            "grep"
        };
        let Some(selector) = call.message_loc() else {
            return;
        };
        let original = String::from_utf8_lossy(original);
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        cop_context.replace(
            format!("Prefer `{replacement}` to `{original}` with a kind check."),
            call.location(),
            selector.start_offset()..block.location().end_offset(),
            format!(
                "{replacement}({})",
                source_at(source, &selection.class.location())
            ),
        );
    }
}

fn kind_selection<'pr>(
    mut body: Node<'pr>,
    parameter: &RangeBlockParameter,
) -> Option<KindSelection<'pr>> {
    let mut negated = false;
    if let Some(negation) = body.as_call_node() {
        if negation.name().as_slice() == b"!" {
            if negation
                .arguments()
                .is_some_and(|arguments| !arguments.arguments().is_empty())
            {
                return None;
            }
            body = negation.receiver()?;
            negated = true;
        }
    }
    let call = body.as_call_node()?;
    if !matches!(call.name().as_slice(), b"is_a?" | b"kind_of?") {
        return None;
    }
    let receiver = call.receiver()?;
    if !is_range_parameter(&receiver, parameter) {
        return None;
    }
    let arguments = call.arguments()?;
    let mut arguments = arguments.arguments().iter();
    let class = arguments.next()?;
    if arguments.next().is_some() {
        return None;
    }
    Some(KindSelection { class, negated })
}

impl Cop for SelectByRange {
    fn name(&self) -> &'static str {
        "Style/SelectByRange"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        let original = call.name().as_slice();
        if !matches!(
            original,
            b"select" | b"filter" | b"find_all" | b"reject" | b"find" | b"detect"
        ) {
            return;
        }
        let Some(block) = call.block().and_then(|block| block.as_block_node()) else {
            return;
        };
        if call.receiver().as_ref().is_some_and(range_hash_receiver) {
            return;
        }
        let Some(parameter) = range_block_parameter(&block) else {
            return;
        };
        let Some(body) = block.body().and_then(single_expression) else {
            return;
        };
        let Some(selection) = range_selection(body, &parameter, source) else {
            return;
        };
        let (grep, suffix, display) = if matches!(original, b"find" | b"detect") {
            if selection.negated {
                ("grep_v", ".first", "grep_v(...).first")
            } else {
                ("grep", ".first", "grep(...).first")
            }
        } else {
            let selecting = matches!(original, b"select" | b"filter" | b"find_all");
            let grep = if selecting == selection.negated {
                "grep_v"
            } else {
                "grep"
            };
            (grep, "", grep)
        };
        let Some(selector) = call.message_loc() else {
            return;
        };
        let original = String::from_utf8_lossy(original);
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        cop_context.replace(
            format!("Prefer `{display}` to `{original}` with a range check."),
            call.location(),
            selector.start_offset()..block.location().end_offset(),
            format!("{grep}({}){suffix}", selection.pattern),
        );
    }
}

fn range_block_parameter(block: &ruby_prism::BlockNode<'_>) -> Option<RangeBlockParameter> {
    let parameters = block.parameters()?;
    if let Some(numbered) = parameters.as_numbered_parameters_node() {
        return (numbered.maximum() == 1).then_some(RangeBlockParameter::Numbered);
    }
    if parameters.as_it_parameters_node().is_some() {
        return Some(RangeBlockParameter::It);
    }
    let block_parameters = parameters.as_block_parameters_node()?;
    let parameters = block_parameters.parameters()?;
    if parameters.requireds().len() != 1
        || !parameters.optionals().is_empty()
        || parameters.rest().is_some()
        || !parameters.posts().is_empty()
        || !parameters.keywords().is_empty()
        || parameters.keyword_rest().is_some()
        || parameters.block().is_some()
    {
        return None;
    }
    let parameter = parameters
        .requireds()
        .first()?
        .as_required_parameter_node()?;
    Some(RangeBlockParameter::Named(
        parameter.name().as_slice().to_vec(),
    ))
}

fn range_selection(
    body: Node<'_>,
    parameter: &RangeBlockParameter,
    source: &str,
) -> Option<RangeSelection> {
    let (body, negated) = unwrap_range_negation(body)?;
    let call = body.as_call_node()?;
    match call.name().as_slice() {
        b"between?" => {
            let receiver = call.receiver()?;
            if !is_range_parameter(&receiver, parameter) {
                return None;
            }
            let arguments = call.arguments()?;
            let arguments = arguments.arguments().iter().collect::<Vec<_>>();
            if arguments.len() != 2 {
                return None;
            }
            Some(RangeSelection {
                pattern: format!(
                    "{}..{}",
                    source_at(source, &arguments[0].location()),
                    source_at(source, &arguments[1].location())
                ),
                negated,
            })
        }
        b"cover?" | b"include?" => {
            let receiver = unwrap_range_literal(call.receiver()?)?;
            let arguments = call.arguments()?;
            let mut arguments = arguments.arguments().iter();
            let argument = arguments.next()?;
            if arguments.next().is_some() || !is_range_parameter(&argument, parameter) {
                return None;
            }
            Some(RangeSelection {
                pattern: source_at(source, &receiver.location()).to_string(),
                negated,
            })
        }
        _ => None,
    }
}

fn unwrap_range_negation(mut node: Node<'_>) -> Option<(Node<'_>, bool)> {
    let mut negated = false;
    if let Some(call) = node.as_call_node() {
        if call.name().as_slice() == b"!" {
            if call
                .arguments()
                .is_some_and(|arguments| !arguments.arguments().is_empty())
            {
                return None;
            }
            node = call.receiver()?;
            negated = true;
        }
    }
    if let Some(parentheses) = node.as_parentheses_node() {
        node = parentheses.body().and_then(single_expression)?;
    }
    Some((node, negated))
}

fn unwrap_range_literal(mut node: Node<'_>) -> Option<Node<'_>> {
    if let Some(parentheses) = node.as_parentheses_node() {
        node = parentheses.body().and_then(single_expression)?;
    }
    node.as_range_node().map(|range| range.as_node())
}

fn is_range_parameter(node: &Node<'_>, parameter: &RangeBlockParameter) -> bool {
    match parameter {
        RangeBlockParameter::Named(name) => node
            .as_local_variable_read_node()
            .is_some_and(|read| read.name().as_slice() == name),
        RangeBlockParameter::Numbered => node
            .as_local_variable_read_node()
            .is_some_and(|read| read.name().as_slice() == b"_1"),
        RangeBlockParameter::It => node.as_it_local_variable_read_node().is_some(),
    }
}

fn range_hash_receiver(node: &Node<'_>) -> bool {
    if node.as_hash_node().is_some() || node_is_root_constant(node, b"ENV") {
        return true;
    }
    node.as_call_node().is_some_and(|call| {
        matches!(call.name().as_slice(), b"to_h" | b"to_hash")
            || matches!(call.name().as_slice(), b"new" | b"[]")
                && call
                    .receiver()
                    .as_ref()
                    .is_some_and(|receiver| node_is_root_constant(receiver, b"Hash"))
    })
}

fn useless_access_modifier(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if matches!(window[0].1.trim(), "private" | "protected" | "public")
            && window[0].1.trim() == window[1].1.trim()
        {
            context.remove(
                "Useless access modifier.",
                window[1].0..window[1].0 + window[1].1.len(),
                window[1].0..window[1].0 + window[1].1.len() + 1,
            );
        }
    }
}

struct AccessModifierDeclarations;

impl Cop for AccessModifierDeclarations {
    fn name(&self) -> &'static str {
        "Style/AccessModifierDeclarations"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        if call.receiver().is_some()
            || !matches!(
                call_name(&call),
                b"private" | b"protected" | b"public" | b"module_function"
            )
            || argument_count(&call) == 0
            || ancestors
                .iter()
                .any(|ancestor| ancestor.as_def_node().is_some())
        {
            return;
        }
        let mut context = context.cop_context(self.name(), source, ancestors);
        if context.policy().enforced_style("group") != "group"
            || allowed_inline_modifier(&call, &context)
            || right_sibling_same_inline_modifier(&call, &context)
        {
            return;
        }
        let Some(selector) = call.message_loc() else {
            return;
        };
        let modifier = context.source_file().at(&selector).to_string();
        let message = format!("`{modifier}` should not be inlined in method definitions.");
        let argument = first_argument(&call).expect("modifier has arguments");
        if let Some(definition) = argument.as_def_node() {
            let indentation = context
                .source_file()
                .indentation_text(selector.start_offset());
            context.replace(
                message,
                &selector,
                selector.end_offset()..definition.location().start_offset(),
                format!("\n{indentation}"),
            );
        } else {
            context.replace(message, &selector, &selector, modifier);
        }
    }
}

fn right_sibling_same_inline_modifier(node: &CallNode<'_>, context: &CopContext<'_, '_>) -> bool {
    let start = node.location().start_offset();
    context.ancestors().iter().rev().any(|ancestor| {
        let statements = if let Some(program) = ancestor.as_program_node() {
            Some(program.statements())
        } else if let Some(class) = ancestor.as_class_node() {
            class.body().and_then(|body| body.as_statements_node())
        } else if let Some(module) = ancestor.as_module_node() {
            module.body().and_then(|body| body.as_statements_node())
        } else if let Some(singleton) = ancestor.as_singleton_class_node() {
            singleton.body().and_then(|body| body.as_statements_node())
        } else if let Some(begin) = ancestor.as_begin_node() {
            begin.statements()
        } else {
            None
        };
        statements.is_some_and(|statements| {
            let direct_child = statements
                .body()
                .iter()
                .any(|child| child.location().start_offset() == start);
            direct_child
                && statements.body().iter().any(|sibling| {
                    let Some(call) = sibling.as_call_node() else {
                        return false;
                    };
                    call.location().start_offset() > start
                        && call.receiver().is_none()
                        && call_name(&call) == call_name(node)
                        && argument_count(&call) > 0
                        && !allowed_inline_modifier(&call, context)
                })
        })
    })
}

fn allowed_inline_modifier(node: &CallNode<'_>, context: &CopContext<'_, '_>) -> bool {
    if direct_block_parent(context.ancestors()) {
        return true;
    }
    let arguments = node
        .arguments()
        .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
        .unwrap_or_default();
    if context.config_bool("AllowModifiersOnSymbols", true)
        && arguments.iter().all(symbol_or_allowed_splat)
    {
        return true;
    }
    let Some(call) = arguments
        .first()
        .and_then(|argument| argument.as_call_node())
    else {
        return false;
    };
    call.receiver().is_none()
        && (context.config_bool("AllowModifiersOnAttrs", true)
            && matches!(
                call_name(&call),
                b"attr" | b"attr_reader" | b"attr_writer" | b"attr_accessor"
            )
            || context.config_bool("AllowModifiersOnAliasMethod", true)
                && call_name(&call) == b"alias_method")
}

fn direct_block_parent(ancestors: &[Node<'_>]) -> bool {
    let Some(parent) = ancestors.last() else {
        return false;
    };
    if parent.as_block_node().is_some() {
        return true;
    }
    let Some(statements) = parent.as_statements_node() else {
        return false;
    };
    if statements.body().len() != 1 {
        return false;
    }
    ancestors[..ancestors.len() - 1]
        .iter()
        .rev()
        .find(|ancestor| ancestor.as_statements_node().is_none())
        .is_some_and(|ancestor| ancestor.as_block_node().is_some())
}

fn symbol_or_allowed_splat(argument: &Node<'_>) -> bool {
    if argument.as_symbol_node().is_some() {
        return true;
    }
    argument
        .as_splat_node()
        .and_then(|splat| splat.expression())
        .is_some_and(|expression| {
            expression.as_array_node().is_some()
                || expression.as_constant_read_node().is_some()
                || expression.as_constant_path_node().is_some()
                || expression.as_call_node().is_some()
        })
}

fn arguments_forwarding(context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(2, 7) {
        return;
    }
    let source = context.source().to_string();
    let signature = ["*args, **kwargs, &block", "*args, &block"]
        .into_iter()
        .find(|signature| {
            source
                .lines()
                .any(|line| line.trim_start().starts_with("def ") && line.contains(signature))
        });
    let Some(signature) = signature else { return };
    if ["args =", "kwargs =", "block ="].iter().any(|assignment| {
        source
            .lines()
            .any(|line| line.trim_start().starts_with(assignment))
    }) {
        return;
    }
    let forwarding = if signature.contains("**kwargs") {
        "*args, **kwargs, &block"
    } else {
        "*args, &block"
    };
    if source.match_indices(forwarding).count() < 2 {
        return;
    }
    for start in source
        .match_indices(signature)
        .map(|(start, _)| start)
        .collect::<Vec<_>>()
    {
        context.replace(
            "Use shorthand syntax `...` for arguments forwarding.",
            start..start + signature.len(),
            start..start + signature.len(),
            "...",
        );
    }
}

struct DuplicateMethods;

#[derive(Default)]
struct DuplicateMethodsState {
    definitions: HashMap<String, SourceDefinition>,
    rescue_scopes: HashMap<&'static str, std::collections::HashSet<String>>,
}

struct SourceDefinition {
    path: String,
    line: usize,
}

impl Cop for DuplicateMethods {
    fn name(&self) -> &'static str {
        "Lint/DuplicateMethods"
    }

    fn investigation_state(&self) -> Box<dyn Any> {
        Box::new(DuplicateMethodsState::default())
    }

    fn on_new_investigation(&self, state: &mut dyn Any) {
        *state
            .downcast_mut::<DuplicateMethodsState>()
            .expect("duplicate methods state") = DuplicateMethodsState::default();
    }

    fn on_node_with_state<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
        state: &mut dyn Any,
    ) {
        if ancestors
            .iter()
            .any(|ancestor| ancestor.as_if_node().is_some() || ancestor.as_unless_node().is_some())
        {
            return;
        }
        let state = state
            .downcast_mut::<DuplicateMethodsState>()
            .expect("duplicate methods state");
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        if let Some(definition) = node.as_def_node() {
            let name = String::from_utf8_lossy(definition.name().as_slice()).into_owned();
            let Some(method) = duplicate_method_name(&definition, ancestors, source, &name) else {
                return;
            };
            let key = method_key_with_scope_id(&method, ancestors, source);
            let offense =
                definition.def_keyword_loc().start_offset()..definition.name_loc().end_offset();
            register_method(state, key, method, offense, &mut cop_context);
        } else if let Some(call) = node.as_call_node() {
            register_attribute_methods(&call, ancestors, state, &mut cop_context);
        }
    }
}

fn duplicate_method_name(
    definition: &ruby_prism::DefNode<'_>,
    ancestors: &[Node<'_>],
    source: &str,
    name: &str,
) -> Option<String> {
    match definition.receiver() {
        None => duplicate_instance_method_name(ancestors, source, name),
        Some(receiver) if receiver.as_self_node().is_some() => {
            let scope = rubocop_parent_module_name(ancestors, source)
                .or_else(|| anonymous_class_scope(ancestors, source).map(|scope| scope.0))?;
            Some(format!("{scope}.{name}"))
        }
        Some(receiver)
            if receiver.as_constant_read_node().is_some()
                || receiver.as_constant_path_node().is_some() =>
        {
            let receiver = node_text(&receiver, source).trim_start_matches("::");
            let scope = rubocop_parent_module_name(ancestors, source)?;
            let qualified = if scope == "Object" || receiver.contains("::") {
                receiver.to_string()
            } else {
                format!("{scope}::{receiver}")
            };
            Some(format!("{qualified}.{name}"))
        }
        Some(_) => None,
    }
}

fn duplicate_instance_method_name(
    ancestors: &[Node<'_>],
    source: &str,
    name: &str,
) -> Option<String> {
    if let Some(scope) = rubocop_parent_module_name(ancestors, source) {
        return Some(format!("{}{name}", humanized_method_scope(&scope)));
    }
    if let Some((scope, _scope_id)) = anonymous_class_scope(ancestors, source) {
        let singleton = ancestors
            .iter()
            .rev()
            .take_while(|ancestor| ancestor.as_block_node().is_none())
            .any(|ancestor| ancestor.as_singleton_class_node().is_some());
        let scope = if singleton {
            format!("#<Class:{scope}>")
        } else {
            scope
        };
        return Some(format!("{}{name}", humanized_method_scope(&scope)));
    }
    let singleton = ancestors
        .iter()
        .rev()
        .find_map(Node::as_singleton_class_node)?;
    let receiver = singleton.expression().as_call_node()?;
    Some(format!(
        "{}.{}",
        String::from_utf8_lossy(receiver.name().as_slice()),
        name
    ))
}

/// Mirrors rubocop-ast's `Node#parent_module_name`. In particular, an ordinary
/// block makes the lexical owner unknowable; treating its methods as members of
/// an enclosing class is the source of a large class of false duplicates.
fn rubocop_parent_module_name(ancestors: &[Node<'_>], source: &str) -> Option<String> {
    let mut parts = Vec::new();
    for (index, ancestor) in ancestors.iter().enumerate() {
        if let Some(class) = ancestor.as_class_node() {
            append_scope_part(&mut parts, node_text(&class.constant_path(), source));
        } else if let Some(module) = ancestor.as_module_node() {
            append_scope_part(&mut parts, node_text(&module.constant_path(), source));
        } else if let Some(singleton) = ancestor.as_singleton_class_node() {
            let expression = singleton.expression();
            let name = if expression.as_self_node().is_some() {
                format!("#<Class:{}>", joined_scope(&parts))
            } else if expression.as_constant_read_node().is_some()
                || expression.as_constant_path_node().is_some()
            {
                format!(
                    "#<Class:{}>",
                    node_text(&expression, source).trim_start_matches("::")
                )
            } else {
                return None;
            };
            parts.push(name);
        } else if let Some(write) = ancestor.as_constant_write_node() {
            if class_or_module_new_call(&write.value()) {
                append_scope_part(&mut parts, location_text(&write.name_loc(), source));
            }
        } else if let Some(write) = ancestor.as_constant_path_write_node() {
            if class_or_module_new_call(&write.value()) {
                append_scope_part(
                    &mut parts,
                    location_text(&write.target().location(), source),
                );
            }
        } else if ancestor.as_block_node().is_some() {
            let Some(call) = index
                .checked_sub(1)
                .and_then(|parent| ancestors[parent].as_call_node())
            else {
                return None;
            };
            if call_name(&call) == b"class_eval" {
                if let Some(receiver) = call.receiver() {
                    if receiver.as_constant_read_node().is_none()
                        && receiver.as_constant_path_node().is_none()
                    {
                        return None;
                    }
                    append_scope_part(&mut parts, node_text(&receiver, source));
                }
            } else if !class_or_module_new_call(&call.as_node())
                || !ancestors.get(index.wrapping_sub(2)).is_some_and(|parent| {
                    parent.as_constant_write_node().is_some()
                        || parent.as_constant_path_write_node().is_some()
                })
            {
                return None;
            }
        }
    }
    Some(if parts.is_empty() {
        "Object".to_string()
    } else {
        joined_scope(&parts)
    })
}

fn class_or_module_new_call(node: &Node<'_>) -> bool {
    node.as_call_node().is_some_and(|call| {
        call_name(&call) == b"new"
            && (root_constant(call.receiver(), b"Class")
                || root_constant(call.receiver(), b"Module"))
    })
}

fn append_scope_part(parts: &mut Vec<String>, raw: &str) {
    let name = raw.trim_start_matches("::");
    if name.contains("::") {
        parts.clear();
    }
    parts.push(name.to_string());
}

fn joined_scope(parts: &[String]) -> String {
    parts.join("::")
}

fn humanized_method_scope(scope: &str) -> String {
    if let Some(start) = scope.find("#<Class:") {
        if let Some(name) = scope[start + 8..].strip_suffix('>') {
            return format!("{name}.");
        }
    }
    format!("{scope}#")
}

fn anonymous_class_scope(ancestors: &[Node<'_>], source: &str) -> Option<(String, Option<String>)> {
    let block_index = ancestors
        .iter()
        .rposition(|ancestor| ancestor.as_block_node().is_some())?;
    let call_index = block_index.checked_sub(1)?;
    let call = ancestors[call_index].as_call_node()?;
    if !class_or_module_new_call(&call.as_node())
        || ancestors
            .get(call_index.wrapping_sub(1))
            .is_some_and(|parent| parent.as_local_variable_write_node().is_some())
    {
        return None;
    }
    if ancestors[block_index + 1..].iter().any(|ancestor| {
        ancestor
            .as_singleton_class_node()
            .is_some_and(|singleton| singleton.expression().as_self_node().is_none())
    }) {
        return None;
    }
    let enclosing = rubocop_parent_module_name(&ancestors[..call_index], source);
    let base = match enclosing.as_deref() {
        Some("Object") => "Object".to_string(),
        Some(enclosing) => format!("{enclosing}::Object"),
        None => "::Object".to_string(),
    };
    let named_scope_id = ancestors[..call_index]
        .iter()
        .rev()
        .find_map(Node::as_call_node)
        .and_then(|parent| {
            parent.receiver().and_then(|receiver| {
                if class_or_module_new_call(&receiver) {
                    return None;
                }
                format!(
                    "{}.{}",
                    node_text(&receiver, source),
                    String::from_utf8_lossy(parent.name().as_slice())
                )
                .into()
            })
        });
    let scope_id = named_scope_id.or_else(|| {
        (duplicate_rescue_scope(&ancestors[..call_index]) != Some("ensure"))
            .then(|| format!("anonymous: {}", call.location().start_offset()))
    });
    Some((base, scope_id))
}

fn node_text<'a>(node: &Node<'_>, source: &'a str) -> &'a str {
    let location = node.location();
    &source[location.start_offset()..location.end_offset()]
}

fn location_text<'a>(location: &ruby_prism::Location<'_>, source: &'a str) -> &'a str {
    &source[location.start_offset()..location.end_offset()]
}

fn method_key_with_scope_id(method: &str, ancestors: &[Node<'_>], source: &str) -> String {
    let mut key = nested_method_key(method, ancestors);
    if rubocop_parent_module_name(ancestors, source).is_none() {
        if let Some(scope_id) = anonymous_class_scope(ancestors, source).and_then(|scope| scope.1) {
            key.push('@');
            key.push_str(&scope_id);
        }
    }
    key
}

fn nested_method_key(method: &str, ancestors: &[Node<'_>]) -> String {
    ancestors
        .iter()
        .rev()
        .find_map(Node::as_def_node)
        .map_or_else(
            || method.to_string(),
            |definition| {
                format!(
                    "{}:{method}",
                    String::from_utf8_lossy(definition.name().as_slice())
                )
            },
        )
}

fn register_attribute_methods(
    call: &CallNode<'_>,
    ancestors: &[Node<'_>],
    state: &mut DuplicateMethodsState,
    context: &mut CopContext<'_, '_>,
) {
    if call.receiver().is_some() {
        return;
    }
    let call_method = call_name(call);
    let arguments = call
        .arguments()
        .into_iter()
        .flat_map(|arguments| arguments.arguments().iter())
        .collect::<Vec<_>>();
    let mut names = Vec::new();
    if matches!(
        call_method,
        b"attr" | b"attr_reader" | b"attr_writer" | b"attr_accessor"
    ) {
        let readable = matches!(call_method, b"attr" | b"attr_reader" | b"attr_accessor");
        let writable = matches!(call_method, b"attr_writer" | b"attr_accessor");
        for argument in &arguments {
            let Some(name) = literal_method_name(argument) else {
                continue;
            };
            if readable {
                names.push(name.clone());
            }
            if writable {
                names.push(format!("{name}="));
            }
        }
    } else if matches!(call_method, b"def_delegator" | b"def_instance_delegator") {
        if let Some(name) = arguments
            .get(if arguments.len() >= 3 { 2 } else { 1 })
            .and_then(|argument| literal_method_name(argument))
        {
            names.push(name);
        }
    } else if matches!(call_method, b"def_delegators" | b"def_instance_delegators") {
        names.extend(
            arguments
                .iter()
                .skip(1)
                .filter_map(|argument| literal_method_name(argument)),
        );
    } else {
        return;
    }
    for name in names {
        let Some(method) = duplicate_instance_method_name(ancestors, context.source(), &name)
        else {
            continue;
        };
        let key = method_key_with_scope_id(&method, ancestors, context.source());
        let location = call.location();
        register_method(
            state,
            key,
            method,
            location.start_offset()..location.end_offset(),
            context,
        );
    }
}

fn literal_method_name(node: &Node<'_>) -> Option<String> {
    if let Some(symbol) = node.as_symbol_node() {
        Some(String::from_utf8_lossy(symbol.unescaped()).into_owned())
    } else {
        node.as_string_node()
            .map(|string| String::from_utf8_lossy(string.unescaped()).into_owned())
    }
}

fn register_method(
    state: &mut DuplicateMethodsState,
    key: String,
    method: String,
    offense: std::ops::Range<usize>,
    context: &mut CopContext<'_, '_>,
) {
    let line = context.source()[..offense.start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let path = smart_source_path(context.path());
    if let Some(previous) = state.definitions.get(&key) {
        let rescue_scope = duplicate_rescue_scope(context.ancestors());
        if let Some(rescue_scope) = rescue_scope {
            if state
                .rescue_scopes
                .entry(rescue_scope)
                .or_default()
                .insert(key.clone())
            {
                state
                    .definitions
                    .insert(key, SourceDefinition { path, line });
                return;
            }
        }
        let message = format!(
            "Method `{method}` is defined at both {}:{} and {path}:{line}.",
            previous.path, previous.line
        );
        context.report(message, offense);
    } else {
        state
            .definitions
            .insert(key, SourceDefinition { path, line });
    }
}

fn duplicate_rescue_scope(ancestors: &[Node<'_>]) -> Option<&'static str> {
    ancestors.iter().rev().find_map(|ancestor| {
        if ancestor.as_rescue_node().is_some() {
            Some("rescue")
        } else if ancestor
            .as_begin_node()
            .is_some_and(|begin| begin.ensure_clause().is_some())
        {
            // Prism exposes `ensure` through its containing BeginNode rather
            // than retaining EnsureNode in the investigation ancestor stack.
            Some("ensure")
        } else {
            None
        }
    })
}

fn smart_source_path(path: &str) -> String {
    let path = std::path::Path::new(path);
    std::env::current_dir()
        .ok()
        .and_then(|current| {
            path.strip_prefix(current)
                .ok()
                .map(|path| path.to_path_buf())
        })
        .unwrap_or_else(|| path.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

const SAFE_NAVIGATION_MESSAGE: &str =
    "Use safe navigation (`&.`) instead of checking if an object exists before calling the method.";

impl Cop for SafeNavigation {
    fn name(&self) -> &'static str {
        "Style/SafeNavigation"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        if !cop_context.target_ruby_version().at_least(2, 3) {
            return;
        }
        if let Some(conditional) = node.as_if_node() {
            safe_navigation_if(&conditional, &mut cop_context);
        } else if let Some(conditional) = node.as_unless_node() {
            safe_navigation_unless(&conditional, &mut cop_context);
        } else if let Some(and_node) = node.as_and_node() {
            if !ancestors
                .iter()
                .any(|parent| parent.as_and_node().is_some())
            {
                safe_navigation_and(&and_node, &mut cop_context);
            }
        }
    }
}

fn safe_navigation_if(node: &ruby_prism::IfNode<'_>, context: &mut CopContext<'_, '_>) {
    if node
        .if_keyword_loc()
        .as_ref()
        .is_some_and(|keyword| keyword.as_slice() == b"elsif")
    {
        return;
    }
    let ternary = node.if_keyword_loc().is_none()
        && node.then_keyword_loc().is_some()
        && node.end_keyword_loc().is_none();
    let then_branch = node.statements().and_then(|body| body.body().first());
    let else_branch = node
        .subsequent()
        .and_then(|subsequent| subsequent.as_else_node())
        .and_then(|else_node| else_node.statements())
        .and_then(|body| body.body().first());
    let (checked, body) = if ternary {
        let Some(then_branch) = then_branch else {
            return;
        };
        let Some(else_branch) = else_branch else {
            return;
        };
        if else_branch.as_nil_node().is_some() {
            if let Some(checked) = non_nil_checked_receiver(&node.predicate()) {
                (checked, then_branch)
            } else if simple_truthy_check(&node.predicate()) {
                (node.predicate(), then_branch)
            } else {
                return;
            }
        } else if then_branch.as_nil_node().is_some() {
            if let Some(checked) = nil_checked_receiver(&node.predicate()) {
                (checked, else_branch)
            } else if let Some(checked) = negated_receiver(&node.predicate()) {
                (checked, else_branch)
            } else {
                return;
            }
        } else {
            return;
        }
    } else {
        if node.subsequent().is_some() {
            return;
        }
        let Some(body) = then_branch else { return };
        if let Some(checked) = non_nil_checked_receiver(&node.predicate()) {
            (checked, body)
        } else if simple_truthy_check(&node.predicate()) {
            (node.predicate(), body)
        } else {
            return;
        }
    };
    safe_navigation_conditional(node.location(), &checked, &body, ternary, context);
}

fn safe_navigation_unless(node: &ruby_prism::UnlessNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.else_clause().is_some() {
        return;
    }
    let Some(body) = node.statements().and_then(|body| body.body().first()) else {
        return;
    };
    let checked = if let Some(checked) = nil_checked_receiver(&node.predicate()) {
        checked
    } else if let Some(checked) = negated_receiver(&node.predicate()) {
        checked
    } else {
        return;
    };
    // `obj.do_something unless obj` uses the variable only as a negative
    // condition, rather than as the positive existence guard this cop targets.
    safe_navigation_conditional(node.location(), &checked, &body, false, context);
}

fn safe_navigation_conditional(
    offense: ruby_prism::Location<'_>,
    checked: &Node<'_>,
    body: &Node<'_>,
    ternary: bool,
    context: &mut CopContext<'_, '_>,
) {
    let checked_source = context.source_file().node(checked).to_string();
    let Some(chain) = safe_navigation_chain(body, &checked_source, ternary, context) else {
        return;
    };
    let mut replacement = corrected_safe_navigation_chain(body, &checked_source, &chain, context);
    let before = &context.source()[offense.start_offset()..body.location().start_offset()];
    let after = &context.source()[body.location().end_offset()..offense.end_offset()];
    let comments = before
        .lines()
        .chain(after.lines())
        .filter_map(|line| line.find('#').map(|comment| line[comment..].trim()))
        .map(|line| format!("{line}\n"))
        .collect::<String>();
    if !comments.is_empty() {
        replacement = format!("{comments}{replacement}");
    }
    let offense = offense.start_offset()..offense.end_offset();
    context.replace(
        SAFE_NAVIGATION_MESSAGE,
        offense.clone(),
        offense,
        replacement,
    );
}

fn safe_navigation_and(node: &ruby_prism::AndNode<'_>, context: &mut CopContext<'_, '_>) {
    let mut clauses = Vec::new();
    flatten_safe_navigation_and(node.as_node(), &mut clauses);
    struct Candidate {
        index: usize,
        offense: std::ops::Range<usize>,
        checked_source: String,
    }
    let mut candidates = Vec::new();
    for (index, pair) in clauses.windows(2).enumerate() {
        let lhs = &pair[0];
        let rhs = &pair[1];
        let (checked_source, non_nil) = if let Some(checked) = non_nil_checked_receiver(lhs) {
            (context.source_file().node(&checked).to_string(), true)
        } else if simple_truthy_check(lhs) {
            (context.source_file().node(lhs).to_string(), false)
        } else {
            continue;
        };
        if non_nil && !context.config_bool("ConvertCodeThatCanStartToReturnNil", false) {
            continue;
        }
        let Some(chain) = safe_navigation_chain(rhs, &checked_source, false, context) else {
            continue;
        };
        let _ = chain;
        let mut end = rhs.location().end_offset();
        let between = &context.source()[lhs.location().end_offset()..rhs.location().start_offset()];
        let opening_parentheses = between.bytes().filter(|byte| *byte == b'(').count();
        for _ in 0..opening_parentheses {
            if context.source().as_bytes().get(end) == Some(&b')') {
                end += 1;
            } else {
                break;
            }
        }
        candidates.push(Candidate {
            index,
            offense: lhs.location().start_offset()..end,
            checked_source,
        });
    }
    if candidates.is_empty() {
        safe_navigation_and_with_or(node, context);
        return;
    }

    let mut groups = Vec::<(usize, usize)>::new();
    let mut group_start = 0;
    for index in 1..candidates.len() {
        if candidates[index].index != candidates[index - 1].index + 1 {
            groups.push((group_start, index - 1));
            group_start = index;
        }
    }
    groups.push((group_start, candidates.len() - 1));

    let node_start = node.location().start_offset();
    let node_end = node.location().end_offset();
    let mut edits = Vec::new();
    for (first, last) in groups {
        let candidate = &candidates[first];
        let lhs = &clauses[candidate.index];
        let final_rhs = &clauses[candidates[last].index + 1];
        let Some(chain) =
            safe_navigation_chain(final_rhs, &candidate.checked_source, false, context)
        else {
            continue;
        };
        let corrected =
            corrected_safe_navigation_chain(final_rhs, &candidate.checked_source, &chain, context);
        let between = &context.source()
            [lhs.location().end_offset()..clauses[candidate.index + 1].location().start_offset()];
        let preserved = between
            .chars()
            .filter(|character| *character == '(')
            .collect::<String>();
        edits.push((
            lhs.location().start_offset()..final_rhs.location().end_offset(),
            format!("{preserved}{corrected}"),
        ));
    }
    edits.sort_by_key(|(range, _)| range.start);
    let mut correction = String::new();
    let mut cursor = node_start;
    for (range, replacement) in edits {
        correction.push_str(&context.source()[cursor..range.start]);
        correction.push_str(&replacement);
        cursor = range.end;
    }
    correction.push_str(&context.source()[cursor..node_end]);

    context.replace(
        SAFE_NAVIGATION_MESSAGE,
        candidates[0].offense.clone(),
        node.location(),
        correction,
    );
    if !context.autocorrect_enabled() {
        for candidate in candidates.iter().skip(1) {
            context.report(SAFE_NAVIGATION_MESSAGE, candidate.offense.clone());
        }
    }
}

fn safe_navigation_and_with_or(node: &ruby_prism::AndNode<'_>, context: &mut CopContext<'_, '_>) {
    let lhs = node.left();
    if !simple_truthy_check(&lhs) {
        return;
    }
    let checked_source = context.source_file().node(&lhs).to_string();
    let right = unwrap_safe_navigation_parentheses(node.right());
    let Some(or_node) = right.as_or_node() else {
        return;
    };
    let Some(candidate) = first_safe_navigation_and_left(or_node.right()) else {
        return;
    };
    if safe_navigation_chain(&candidate, &checked_source, false, context).is_none() {
        return;
    }
    context.report(
        SAFE_NAVIGATION_MESSAGE,
        lhs.location().start_offset()..candidate.location().end_offset(),
    );
}

fn unwrap_safe_navigation_parentheses(mut node: Node<'_>) -> Node<'_> {
    loop {
        let Some(parentheses) = node.as_parentheses_node() else {
            return node;
        };
        let Some(inner) = parentheses.body().and_then(single_expression) else {
            return node;
        };
        node = inner;
    }
}

fn first_safe_navigation_and_left(node: Node<'_>) -> Option<Node<'_>> {
    let node = unwrap_safe_navigation_parentheses(node);
    if let Some(and_node) = node.as_and_node() {
        return Some(and_node.left());
    }
    let or_node = node.as_or_node()?;
    first_safe_navigation_and_left(or_node.left())
        .or_else(|| first_safe_navigation_and_left(or_node.right()))
}

fn flatten_safe_navigation_and<'pr>(node: Node<'pr>, clauses: &mut Vec<Node<'pr>>) {
    if let Some(parentheses) = node.as_parentheses_node() {
        if let Some(inner) = parentheses.body().and_then(single_expression) {
            flatten_safe_navigation_and(inner, clauses);
            return;
        }
    }
    if let Some(and_node) = node.as_and_node() {
        flatten_safe_navigation_and(and_node.left(), clauses);
        flatten_safe_navigation_and(and_node.right(), clauses);
    } else {
        clauses.push(node);
    }
}

fn safe_navigation_chain<'pr>(
    body: &Node<'pr>,
    checked_source: &str,
    ternary: bool,
    context: &CopContext<'_, '_>,
) -> Option<Vec<CallNode<'pr>>> {
    let mut calls = Vec::new();
    let mut call = body.as_call_node()?;
    loop {
        if call_name(&call) == b"!" {
            return None;
        }
        let receiver = call.receiver()?;
        calls.push(call);
        if safe_navigation_source_matches(
            source_at(context.source(), &receiver.location()),
            checked_source,
        ) {
            break;
        }
        call = receiver.as_call_node()?;
    }
    calls.reverse();
    if calls.len() > context.config_usize("MaxChainLength", 2) {
        return None;
    }
    if calls.len() > 1
        && context.related_config_value("Lint/SafeNavigationChain", "Enabled") == Some("false")
    {
        return None;
    }
    let first = calls.first()?;
    let first_operator = first.call_operator_loc()?;
    if first_operator.as_slice() == b"::" || (!ternary && unsafe_safe_navigation_call(first)) {
        return None;
    }
    if body
        .as_call_node()
        .is_some_and(|call| call_name(&call) == b"empty?")
    {
        return None;
    }
    for call in calls.iter().skip(1) {
        if unsafe_safe_navigation_call(call)
            || safe_navigation_nil_method(call_name(call))
            || safe_navigation_allowed_method(call_name(call), context)
        {
            return None;
        }
    }
    Some(calls)
}

fn safe_navigation_allowed_method(name: &[u8], context: &CopContext<'_, '_>) -> bool {
    matches!(
        name,
        b"present?" | b"blank?" | b"presence" | b"try" | b"try!"
    ) || context.policy().allows_method(name)
}

fn corrected_safe_navigation_chain(
    body: &Node<'_>,
    checked_source: &str,
    calls: &[CallNode<'_>],
    context: &CopContext<'_, '_>,
) -> String {
    let body_start = body.location().start_offset();
    let body_end = body.location().end_offset();
    let mut edits = Vec::new();
    let matched = calls.first().and_then(CallNode::receiver);
    if let Some(matched) = matched {
        let matched_source = context.source_file().node(&matched);
        if checked_source != matched_source {
            edits.push((
                matched.location().start_offset()..matched.location().end_offset(),
                checked_source.to_string(),
            ));
        }
    }
    for call in calls {
        if let Some(operator) = call.call_operator_loc() {
            if operator.as_slice() == b"." {
                edits.push((
                    operator.start_offset()..operator.start_offset(),
                    "&".to_string(),
                ));
            }
        }
    }
    edits.sort_by_key(|(range, _)| range.start);
    let mut rendered = String::new();
    let mut cursor = body_start;
    for (range, replacement) in edits {
        if range.start < cursor || range.end > body_end {
            continue;
        }
        rendered.push_str(&context.source()[cursor..range.start]);
        rendered.push_str(&replacement);
        cursor = range.end;
    }
    rendered.push_str(&context.source()[cursor..body_end]);
    rendered
}

fn simple_truthy_check(node: &Node<'_>) -> bool {
    node.as_call_node()
        .is_none_or(|call| !matches!(call_name(&call), b"!" | b"nil?" | b"respond_to?"))
        && node.as_and_node().is_none()
        && node.as_or_node().is_none()
}

fn nil_checked_receiver<'pr>(node: &Node<'pr>) -> Option<Node<'pr>> {
    let call = node.as_call_node()?;
    if call_name(&call) == b"nil?" && argument_count(&call) == 0 {
        call.receiver()
    } else {
        None
    }
}

fn non_nil_checked_receiver<'pr>(node: &Node<'pr>) -> Option<Node<'pr>> {
    let receiver = negated_receiver(node)?;
    nil_checked_receiver(&receiver)
}

fn negated_receiver<'pr>(node: &Node<'pr>) -> Option<Node<'pr>> {
    let call = node.as_call_node()?;
    if call_name(&call) == b"!" && argument_count(&call) == 0 {
        call.receiver()
    } else {
        None
    }
}

fn safe_navigation_source_matches(left: &str, right: &str) -> bool {
    normalize_safe_navigation_source(left) == normalize_safe_navigation_source(right)
}

fn normalize_safe_navigation_source(source: &str) -> String {
    #[derive(Clone, Copy)]
    enum Literal {
        Quote(char),
        Percent {
            open: char,
            close: char,
            depth: usize,
        },
    }
    let characters = source.chars().collect::<Vec<_>>();
    let mut normalized = String::new();
    let mut literal = None;
    let mut escaped = false;
    let mut index = 0;
    while index < characters.len() {
        let character = characters[index];
        if let Some(state) = literal {
            normalized.push(character);
            if escaped {
                escaped = false;
                index += 1;
                continue;
            }
            if character == '\\' {
                escaped = true;
                index += 1;
                continue;
            }
            match state {
                Literal::Quote(close) if character == close => literal = None,
                Literal::Percent {
                    open,
                    close,
                    mut depth,
                } => {
                    if character == open && open != close {
                        depth += 1;
                        literal = Some(Literal::Percent { open, close, depth });
                    } else if character == close {
                        if depth == 0 {
                            literal = None;
                        } else {
                            literal = Some(Literal::Percent {
                                open,
                                close,
                                depth: depth - 1,
                            });
                        }
                    }
                }
                _ => {}
            }
            index += 1;
            continue;
        }
        if character.is_whitespace() {
            index += 1;
            continue;
        }
        if character == '&' && characters.get(index + 1) == Some(&'.') {
            normalized.push('.');
            index += 2;
            continue;
        }
        if matches!(character, '\'' | '"' | '`' | '/') {
            normalized.push(character);
            literal = Some(Literal::Quote(character));
            index += 1;
            continue;
        }
        if character == '%' {
            let delimiter_index = if characters.get(index + 1).is_some_and(|kind| {
                matches!(kind, 'q' | 'Q' | 'r' | 'w' | 'W' | 'i' | 'I' | 'x' | 's')
            }) {
                index + 2
            } else {
                index + 1
            };
            if let Some(&open) = characters.get(delimiter_index) {
                if !open.is_alphanumeric() && !open.is_whitespace() {
                    for value in &characters[index..=delimiter_index] {
                        normalized.push(*value);
                    }
                    let close = match open {
                        '(' => ')',
                        '[' => ']',
                        '{' => '}',
                        '<' => '>',
                        other => other,
                    };
                    literal = Some(Literal::Percent {
                        open,
                        close,
                        depth: 0,
                    });
                    index = delimiter_index + 1;
                    continue;
                }
            }
        }
        normalized.push(character);
        index += 1;
    }
    normalized
}

fn unsafe_safe_navigation_call(call: &CallNode<'_>) -> bool {
    let name = call_name(call);
    let assignment = name.ends_with(b"=")
        && !matches!(
            name,
            b"==" | b"!=" | b"<=" | b">=" | b"===" | b"=~" | b"!~" | b"<=>"
        );
    assignment
        || call.call_operator_loc().is_none()
        || call
            .call_operator_loc()
            .is_some_and(|operator| operator.as_slice() == b"::")
}

fn safe_navigation_nil_method(name: &[u8]) -> bool {
    matches!(
        name,
        b"nil?"
            | b"to_s"
            | b"to_i"
            | b"to_f"
            | b"to_a"
            | b"to_h"
            | b"to_c"
            | b"to_r"
            | b"inspect"
            | b"hash"
            | b"object_id"
            | b"class"
            | b"itself"
            | b"freeze"
            | b"frozen?"
    )
}
