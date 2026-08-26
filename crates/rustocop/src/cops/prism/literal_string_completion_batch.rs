use super::*;
use crate::rubocop::ast::node::core::NodeRef as RubocopNodeRef;
use crate::rubocop::ast::prism::convert as convert_rubocop_ast;

#[derive(Default)]
struct WordArrayState {
    matrix_of_complex_content: std::collections::HashMap<(usize, usize), bool>,
}

define_stateful_rule!(WordArrayRule, WordArrayState);

const PERCENT_MSG: &str = "Use `%w` or `%W` for an array of words.";

define_cops! {
    RedundantCapitalW => "Style/RedundantCapitalW" => node(as_array_node, redundant_capital_w),
    SymbolArray => "Style/SymbolArray" => node(as_array_node, symbol_array),
    FetchEnvVar => "Style/FetchEnvVar" => source(fetch_env_var),
    StringConcatenation => "Style/StringConcatenation" => call(string_concatenation),
    WordArray => "Style/WordArray" => stateful_node_rule(as_array_node, WordArrayRule, WordArrayState, on_array),
}

fn redundant_capital_w(node: &ruby_prism::ArrayNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(opening) = node.opening_loc() else {
        return;
    };
    if !context.source_file().at(&opening).starts_with("%W") {
        return;
    }
    let range = context.source_file().node_range(&node.as_node());
    let literal = &context.source()[range.clone()];
    if literal.contains("#{") || literal.contains('\\') {
        return;
    }
    context.replace(
        "Do not use `%W` unless interpolation is needed. If not, use `%w`.",
        range,
        opening.start_offset() + 1..opening.start_offset() + 2,
        "w",
    );
}

fn symbol_array(node: &ruby_prism::ArrayNode<'_>, context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(2, 0) {
        return;
    }
    let Some(opening) = node.opening_loc() else {
        return;
    };
    let opening_source = context.source_file().at(&opening);
    let elements = node.elements().iter().collect::<Vec<_>>();
    if elements.len() < context.config_usize("MinSize", 2) {
        return;
    }

    if opening_source == "[" {
        if context.policy().enforced_style("percent") != "percent"
            || elements.is_empty()
            || elements
                .iter()
                .any(|element| element.as_symbol_node().is_none())
            || bracket_array_has_comment(node, context)
            || invalid_percent_array_context(node, context)
            || complex_symbol_content(&elements, false, context)
        {
            return;
        }
        let Some(replacement) = percent_symbol_array(node, &elements, context) else {
            return;
        };
        context.replace(
            "Use `%i` or `%I` for an array of symbols.",
            node.location(),
            node.location(),
            replacement,
        );
        return;
    }

    if !opening_source.starts_with("%i") && !opening_source.starts_with("%I") {
        return;
    }
    let invalid = complex_symbol_content(&elements, true, context);
    if context.policy().enforced_style("percent") == "percent" && !invalid {
        return;
    }
    let replacement = bracketed_symbol_array(node, &elements, context);
    let message = if replacement == "[]" {
        "Use `[]` for an array of symbols.".to_string()
    } else if replacement.contains('\n') {
        "Use an array literal `[...]` for an array of symbols.".to_string()
    } else {
        format!("Use `{replacement}` for an array of symbols.")
    };
    context.replace(message, node.location(), node.location(), replacement);
}

fn invalid_percent_array_context(
    node: &ruby_prism::ArrayNode<'_>,
    context: &CopContext<'_, '_>,
) -> bool {
    let location = node.location();
    context.ancestors().iter().rev().any(|ancestor| {
        ancestor.as_call_node().is_some_and(|call| {
            call.opening_loc().is_none()
                && call
                    .block()
                    .and_then(|block| block.as_block_node())
                    .is_some()
                && call.arguments().is_some_and(|arguments| {
                    arguments.arguments().iter().any(|argument| {
                        argument.location().start_offset() == location.start_offset()
                            && argument.location().end_offset() == location.end_offset()
                    })
                })
        })
    })
}

fn static_symbol_value(node: &Node<'_>) -> Option<String> {
    let symbol = node.as_symbol_node()?;
    std::str::from_utf8(symbol.unescaped())
        .ok()
        .map(str::to_string)
}

fn complex_symbol_content(
    elements: &[Node<'_>],
    percent: bool,
    context: &CopContext<'_, '_>,
) -> bool {
    for element in elements {
        let source = context.source_file().node(element);
        if percent && matches!(source, "[" | "]" | "(" | ")") {
            return false;
        }
        let Some(value) = static_symbol_value(element) else {
            continue;
        };
        let reduced = remove_balanced_symbol_delimiters(&value);
        if value.contains(' ')
            || reduced
                .chars()
                .any(|character| matches!(character, '[' | ']' | '(' | ')'))
        {
            return true;
        }
    }
    false
}

fn remove_balanced_symbol_delimiters(value: &str) -> String {
    let mut output = String::new();
    let characters = value.chars().collect::<Vec<_>>();
    let mut index = 0;
    while index < characters.len() {
        let open = characters[index];
        let close = match open {
            '[' => ']',
            '(' => ')',
            _ => {
                output.push(open);
                index += 1;
                continue;
            }
        };
        let mut end = index + 1;
        while end < characters.len()
            && characters[end] != close
            && characters[end] != open
            && !characters[end].is_whitespace()
        {
            end += 1;
        }
        if end < characters.len() && characters[end] == close {
            index = end + 1;
        } else {
            output.push(open);
            index += 1;
        }
    }
    output
}

fn percent_symbol_array(
    node: &ruby_prism::ArrayNode<'_>,
    elements: &[Node<'_>],
    context: &CopContext<'_, '_>,
) -> Option<String> {
    let wide = elements.iter().any(|element| {
        static_symbol_value(element).is_some_and(|value| value.chars().any(char::is_control))
    });
    let kind = if wide { "%I" } else { "%i" };
    let delimiters = context
        .related_config_map("Style/PercentLiteralDelimiters", "PreferredDelimiters")
        .and_then(|values| values.get(kind))
        .map(String::as_str)
        .unwrap_or("[]");
    let (open, close) = delimiters.split_at(1);
    let words = elements
        .iter()
        .map(|element| {
            static_symbol_value(element)
                .map(|value| escape_percent_symbol(&value, wide, open, close))
        })
        .collect::<Option<Vec<_>>>()?;
    let body = if context.source_file().node(&node.as_node()).contains('\n') {
        format_percent_multiline(node, elements, &words, context)
    } else {
        words.join(" ")
    };
    Some(format!("{kind}{open}{body}{close}"))
}

fn escape_percent_symbol(value: &str, wide: bool, open: &str, close: &str) -> String {
    let mut rendered = String::new();
    let mut balance = 0usize;
    let mut closings = value.matches(close).count();
    for character in value.chars() {
        let token = match character {
            '\n' if wide => "\\n".to_string(),
            '\t' if wide => "\\t".to_string(),
            '\r' if wide => "\\r".to_string(),
            character => character.to_string(),
        };
        if token == open && open != close && closings > balance {
            balance += 1;
            rendered.push_str(&token);
        } else if token == close && open != close && balance > 0 {
            balance -= 1;
            closings = closings.saturating_sub(1);
            rendered.push_str(&token);
        } else if token == open || token == close {
            closings = closings.saturating_sub(1);
            rendered.push('\\');
            rendered.push_str(&token);
        } else {
            rendered.push_str(&token);
        }
    }
    rendered
}

fn bracketed_symbol_array(
    node: &ruby_prism::ArrayNode<'_>,
    elements: &[Node<'_>],
    context: &CopContext<'_, '_>,
) -> String {
    if elements.is_empty() {
        return "[]".to_string();
    }
    let symbols = elements
        .iter()
        .map(|element| bracketed_symbol(element, context))
        .collect::<Vec<_>>();
    let source = context.source_file().node(&node.as_node());
    if !source.contains('\n') {
        return format!("[{}]", symbols.join(", "));
    }
    let opening_end = node
        .opening_loc()
        .map_or(node.location().start_offset() + 3, |location| {
            location.end_offset()
        });
    let closing_start = node
        .closing_loc()
        .map_or(node.location().end_offset().saturating_sub(1), |location| {
            location.start_offset()
        });
    let prefix = &context.source()[opening_end..elements[0].location().start_offset()];
    let indent = prefix.rsplit('\n').next().unwrap_or("");
    let closing_indent = &context.source()
        [elements.last().unwrap().location().end_offset()..closing_start]
        .rsplit('\n')
        .next()
        .unwrap_or("");
    format!(
        "[\n{indent}{}\n{closing_indent}]",
        symbols.join(&format!(",\n{indent}"))
    )
}

fn bracketed_symbol(node: &Node<'_>, context: &CopContext<'_, '_>) -> String {
    if let Some(value) = static_symbol_value(node) {
        return symbol_literal(&value);
    }
    let source = context.source_file().node(node);
    format!(":\"{}\"", source.replace('"', "\\\""))
}

fn symbol_literal(value: &str) -> String {
    if bare_symbol(value) {
        return format!(":{value}");
    }
    if value.contains('\'') || value.chars().any(char::is_control) {
        return format!(":\"{}\"", escape_double_quoted(value));
    }
    format!(":'{}'", value.replace('\\', "\\\\"))
}

fn bare_symbol(value: &str) -> bool {
    let bytes = value.as_bytes();
    let identifier = |value: &[u8]| {
        !value.is_empty()
            && (value[0].is_ascii_alphabetic() || value[0] == b'_')
            && value[1..]
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
    };
    let method = if bytes
        .last()
        .is_some_and(|last| matches!(last, b'!' | b'?'))
    {
        &bytes[..bytes.len() - 1]
    } else {
        bytes
    };
    if identifier(method) {
        return true;
    }
    if let Some(variable) = value.strip_prefix("@@").or_else(|| value.strip_prefix('@')) {
        return identifier(variable.as_bytes());
    }
    if let Some(global) = value.strip_prefix('$') {
        return identifier(global.as_bytes())
            || !global.is_empty()
                && global.bytes().all(|byte| byte.is_ascii_digit())
                && global.as_bytes()[0] != b'0'
            || matches!(
                global,
                "!" | "\""
                    | "$"
                    | "&"
                    | "'"
                    | "*"
                    | "+"
                    | ","
                    | "/"
                    | ";"
                    | ":"
                    | "."
                    | "<"
                    | "="
                    | ">"
                    | "?"
                    | "@"
                    | "\\"
                    | "_"
                    | "`"
                    | "~"
                    | "0"
            )
            || global.starts_with('-');
    }
    matches!(
        value,
        "|" | "^"
            | "&"
            | "<=>"
            | "=="
            | "==="
            | "=~"
            | ">"
            | ">="
            | "<"
            | "<="
            | "<<"
            | ">>"
            | "+"
            | "-"
            | "*"
            | "/"
            | "%"
            | "**"
            | "~"
            | "+@"
            | "-@"
            | "[]"
            | "[]="
            | "`"
            | "!"
            | "!="
            | "!~"
    )
}

fn fetch_env_var(context: &mut CopContext<'_, '_>) {
    let source = context.source().to_string();
    let default_to_nil = context.config_bool("DefaultToNil", true);
    let allowed = context.config_values("AllowedVars").to_vec();
    let parsed = ruby_prism::parse(source.as_bytes());
    let (ast, root) = convert_rubocop_ast(&source, &parsed.node());
    let Some(root) = root.map(|root| ast.node(root)) else {
        return;
    };

    for node in root.each_node(&["send"]) {
        if node.method_name() != Some("[]") {
            continue;
        }
        let Some(receiver) = node.receiver() else {
            continue;
        };
        if receiver.kind() != "const" || receiver.const_name().as_deref() != Some("ENV") {
            continue;
        }
        let arguments = node.arguments();
        let [name_node] = arguments.as_slice() else {
            continue;
        };
        if name_node
            .str_content()
            .is_some_and(|name| allowed.iter().any(|allowed| allowed == name))
            || fetch_env_allowable_use(node)
        {
            continue;
        }
        let (Some(node_range), Some(name_range)) = (node.source_range(), name_node.source_range())
        else {
            continue;
        };
        let key = source
            .chars()
            .skip(name_range.start)
            .take(name_range.end - name_range.start)
            .collect::<String>();
        let original = source
            .chars()
            .skip(node_range.start)
            .take(node_range.end - node_range.start)
            .collect::<String>();
        let default = if default_to_nil { ", nil" } else { "" };
        let byte_range = fetch_env_character_range_to_byte(&source, node_range);
        context.replace(
            format!("Use `ENV.fetch({key}{default})` instead of `{original}`."),
            byte_range.clone(),
            byte_range,
            format!("ENV.fetch({key}{default})"),
        );
    }
}

fn fetch_env_allowable_use(node: RubocopNodeRef<'_>) -> bool {
    fetch_env_used_as_flag(node)
        || fetch_env_message_chained_with_dot(node)
        || fetch_env_assigned(node)
        || fetch_env_or_lhs(node)
}

fn fetch_env_used_as_flag(node: RubocopNodeRef<'_>) -> bool {
    if node.root() {
        return false;
    }
    if fetch_env_used_if_condition_in_body(node) {
        return true;
    }
    node.parent().is_some_and(|parent| {
        parent.kind() == "send" && (fetch_env_prefix_bang(parent, node) || parent.comparison_method())
    })
}

fn fetch_env_prefix_bang(parent: RubocopNodeRef<'_>, node: RubocopNodeRef<'_>) -> bool {
    parent.method_name() == Some("!")
        && parent.receiver() == Some(node)
        && parent.loc_is("selector", "!")
}

fn fetch_env_used_if_condition_in_body(node: RubocopNodeRef<'_>) -> bool {
    let Some(condition) = node
        .ancestors()
        .into_iter()
        .find(|ancestor| ancestor.kind() == "if")
        .and_then(RubocopNodeRef::condition)
    else {
        return false;
    };
    if condition.kind() == "send"
        && fetch_env_node_lists_equal(&condition.child_nodes(), &node.child_nodes())
    {
        return true;
    }
    fetch_env_used_in_condition(node, condition)
}

fn fetch_env_used_in_condition(
    node: RubocopNodeRef<'_>,
    condition: RubocopNodeRef<'_>,
) -> bool {
    if condition.kind() == "send" {
        if condition.assignment_method() && fetch_env_partial_match(node, condition) {
            return true;
        }
        if !condition.comparison_method() && !condition.predicate_method() {
            return false;
        }
    }
    condition
        .child_nodes()
        .into_iter()
        .any(|child| child.structurally_equal(node))
}

fn fetch_env_partial_match(node: RubocopNodeRef<'_>, condition: RubocopNodeRef<'_>) -> bool {
    let condition_children = condition.child_nodes();
    node.child_nodes()
        .into_iter()
        .all(|child| {
            condition_children
                .iter()
                .any(|candidate| candidate.structurally_equal(child))
        })
}

fn fetch_env_node_lists_equal(left: &[RubocopNodeRef<'_>], right: &[RubocopNodeRef<'_>]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| left.structurally_equal(*right))
}

fn fetch_env_message_chained_with_dot(node: RubocopNodeRef<'_>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    parent.call_type()
        && parent.receiver() == Some(node)
        && (parent.loc("dot").is_some() || parent.kind() == "csend")
}

fn fetch_env_assigned(node: RubocopNodeRef<'_>) -> bool {
    node.parent().is_some_and(|parent| {
        matches!(parent.kind(), "op_asgn" | "and_asgn" | "or_asgn" | "masgn")
            && parent.lhs() == Some(node)
    })
}

fn fetch_env_or_lhs(node: RubocopNodeRef<'_>) -> bool {
    node.parent().is_some_and(|parent| {
        parent.kind() == "or"
            && (parent.lhs() == Some(node)
                || parent.parent().is_some_and(|grandparent| grandparent.kind() == "or"))
    })
}

fn fetch_env_character_range_to_byte(
    source: &str,
    range: std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    let start = source
        .char_indices()
        .nth(range.start)
        .map_or(source.len(), |(byte, _)| byte);
    let end = source
        .char_indices()
        .nth(range.end)
        .map_or(source.len(), |(byte, _)| byte);
    start..end
}

fn string_concatenation(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.is_safe_navigation() || !plus_call(node) {
        return;
    }
    let Some(receiver) = node.receiver() else {
        return;
    };
    let Some(argument) = only_argument(node) else {
        return;
    };
    let receiver_string = rubocop_plain_string(&receiver, context);
    let argument_string = rubocop_plain_string(&argument, context);
    if !(receiver_string || argument_string) {
        return;
    }
    let between =
        &context.source()[receiver.location().end_offset()..argument.location().start_offset()];
    if receiver_string && argument_string && between.contains('\n')
    {
        return;
    }

    let mut topmost = node.as_node();
    for ancestor in context.ancestors().iter().rev() {
        let Some(call) = ancestor.as_call_node().filter(plus_call) else {
            break;
        };
        topmost = call.as_node();
        let ancestor_matches = call.receiver().is_some_and(|part| rubocop_plain_string(&part, context))
            || only_argument(&call).is_some_and(|part| rubocop_plain_string(&part, context));
        if ancestor_matches {
            // The ancestor callback owns the one offense for this chain.
            return;
        }
    }

    let topmost_location = topmost.location();
    let topmost_range = topmost_location.start_offset()..topmost_location.end_offset();
    let mut parts = Vec::new();
    collect_concatenation_parts(topmost, &mut parts);
    if !parts.iter().any(|part| part.as_string_node().is_some()) {
        return;
    }
    if context.config_value("Mode").unwrap_or("aggressive") == "conservative"
        && parts
            .first().is_none_or(|part| part.as_string_node().is_none())
    {
        return;
    }
    let message = "Prefer string interpolation to string concatenation.";
    if context.ancestors().iter().any(|ancestor| {
        ancestor.as_call_node().is_some_and(|call| plus_call(&call))
            && ancestor.location().start_offset() < topmost_range.start
    }) {
        context.report(message, topmost_range);
        return;
    }
    if parts
        .iter()
        .any(|part| uncorrectable_concatenation_part(part, context))
    {
        context.report(message, topmost_range);
        return;
    }
    let mut body = String::new();
    for part in parts {
        body.push_str(&interpolated_part(&part, context));
    }
    context.replace(
        message,
        topmost_range.clone(),
        topmost_range,
        format!("\"{body}\""),
    );
}

fn rubocop_plain_string(node: &Node<'_>, context: &CopContext<'_, '_>) -> bool {
    let Some(string) = node.as_string_node() else {
        return false;
    };
    // RuboCop's Prism translation represents a physical multiline literal as
    // `dstr` (one string child per source line), even without interpolation.
    // `StringConcatenation` deliberately matches only Parser `str` nodes.
    context
        .source_file()
        .at(&string.content_loc())
        .lines()
        .count()
        <= 1
}

fn plus_call(node: &CallNode<'_>) -> bool {
    call_name(node) == b"+" && argument_count(node) == 1
}

fn string_part(node: &Node<'_>) -> bool {
    node.as_string_node().is_some() || node.as_interpolated_string_node().is_some()
}

fn collect_concatenation_parts<'pr>(node: Node<'pr>, parts: &mut Vec<Node<'pr>>) {
    if let Some(call) = node.as_call_node().filter(plus_call) {
        if let (Some(receiver), Some(argument)) = (call.receiver(), only_argument(&call)) {
            collect_concatenation_parts(receiver, parts);
            collect_concatenation_parts(argument, parts);
            return;
        }
    }
    parts.push(node);
}

fn uncorrectable_concatenation_part(node: &Node<'_>, context: &CopContext<'_, '_>) -> bool {
    let source = context.source_file().node(node);
    if source.contains('\n') || string_part(node) && source.trim_start().starts_with("<<") {
        return true;
    }
    let mut finder = ConcatenationBlockFinder(false);
    finder.visit(node);
    finder.0
}

struct ConcatenationBlockFinder(bool);

impl<'pr> Visit<'pr> for ConcatenationBlockFinder {
    fn visit_block_node(&mut self, _node: &ruby_prism::BlockNode<'pr>) {
        self.0 = true;
    }
}

fn interpolated_part(node: &Node<'_>, context: &CopContext<'_, '_>) -> String {
    if let Some(string) = node.as_string_node() {
        let value = String::from_utf8_lossy(string.unescaped());
        let single = string
            .opening_loc()
            .is_some_and(|opening| opening.as_slice() == b"'");
        return escape_interpolated_text(&value, single);
    }
    if let Some(string) = node.as_interpolated_string_node() {
        if string.opening_loc().is_none() {
            return string
                .parts()
                .iter()
                .map(|part| interpolated_part(&part, context))
                .collect();
        }
        return string
            .parts()
            .iter()
            .map(|part| {
                if part.as_string_node().is_some() {
                    interpolated_part(&part, context)
                } else {
                    context.source_file().node(&part).to_string()
                }
            })
            .collect();
    }
    format!("#{{{}}}", interpolation_expression(node, context))
}

fn interpolation_expression(node: &Node<'_>, context: &CopContext<'_, '_>) -> String {
    let expression = node
        .as_parentheses_node()
        .and_then(|parentheses| parentheses.body())
        .and_then(|body| {
            body.as_statements_node()
                .and_then(|statements| statements.body().first())
                .or(Some(body))
        });
    if let Some(expression) = expression {
        render_interpolation_expression(&expression, context)
    } else {
        render_interpolation_expression(node, context)
    }
}

fn render_interpolation_expression(expression: &Node<'_>, context: &CopContext<'_, '_>) -> String {
    let range = expression.location().start_offset()..expression.location().end_offset();
    let mut finder = NestedConcatenationFinder {
        context,
        edits: Vec::new(),
    };
    finder.visit(expression);
    let mut rendered = context.source_file().node(expression).to_string();
    finder.edits.sort_by_key(|(edit, _)| (edit.start, edit.end));
    for (edit, replacement) in finder.edits.into_iter().rev() {
        rendered.replace_range(
            edit.start - range.start..edit.end - range.start,
            &replacement,
        );
    }
    rendered
}

struct NestedConcatenationFinder<'a, 'context, 'pr> {
    context: &'a CopContext<'context, 'pr>,
    edits: Vec<(std::ops::Range<usize>, String)>,
}

impl<'context, 'pr> Visit<'pr> for NestedConcatenationFinder<'_, 'context, 'pr> {
    fn visit_call_node(&mut self, node: &CallNode<'pr>) {
        if let Some(replacement) = nested_concatenation_replacement(node, self.context) {
            self.edits.push((
                node.location().start_offset()..node.location().end_offset(),
                replacement,
            ));
            return;
        }
        ruby_prism::visit_call_node(self, node);
    }
}

fn nested_concatenation_replacement(
    node: &CallNode<'_>,
    context: &CopContext<'_, '_>,
) -> Option<String> {
    if !plus_call(node) {
        return None;
    }
    let mut parts = Vec::new();
    collect_concatenation_parts(node.as_node(), &mut parts);
    if !parts.iter().any(string_part)
        || context.config_value("Mode").unwrap_or("aggressive") == "conservative"
            && !parts.first().is_some_and(|part| string_part(part))
        || parts
            .iter()
            .any(|part| uncorrectable_concatenation_part(part, context))
    {
        return None;
    }
    let body = parts
        .iter()
        .map(|part| interpolated_part(part, context))
        .collect::<String>();
    Some(format!("\"{body}\""))
}

fn escape_interpolated_text(value: &str, single_quoted: bool) -> String {
    let escaped = concatenation_inspect_string(value);
    let mut body = escaped[1..escaped.len() - 1].to_string();
    if single_quoted {
        body = body
            .replace("#{", "\\#{")
            .replace("#@", "\\#@")
            .replace("#$", "\\#$");
    }
    body
}

fn concatenation_inspect_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{escaped}\"")
}

impl WordArrayRule<'_, '_, '_> {
    fn on_array(&mut self, node: &ruby_prism::ArrayNode<'_>) {
        let Some(opening) = node.opening_loc() else {
            return;
        };
        let opening_source = self.source_file().at(&opening);
        let elements = node.elements().iter().collect::<Vec<_>>();
        if opening_source == "[" {
            let minimum = self.config_usize("MinSize", 0);
            return_unless!(elements.len() >= minimum);
            return_if!(
                elements.iter().any(|element| element.as_string_node().is_none())
                    || complex_content(&elements, self)
                    || self.within_matrix_of_complex_content()
                    || invalid_percent_array_context(node, self)
            );
            self.check_bracketed_array(node, &elements);
        } else if opening_source.starts_with("%w") || opening_source.starts_with("%W") {
            self.check_percent_array(node, &elements);
        }
    }

    fn check_bracketed_array(
        &mut self,
        node: &ruby_prism::ArrayNode<'_>,
        elements: &[Node<'_>],
    ) {
        return_unless!(self.policy().enforced_style("percent") == "percent");
        return_if!(elements.is_empty() || bracket_array_has_comment(node, self));
        let Some(replacement) = percent_word_array(node, elements, self) else {
            return;
        };
        add_offense!(
            self,
            node.location(),
            message: PERCENT_MSG,
            |corrector| {
                corrector.replace(node.location(), replacement);
            }
        );
    }

    fn check_percent_array(
        &mut self,
        node: &ruby_prism::ArrayNode<'_>,
        elements: &[Node<'_>],
    ) {
        return_if!(
            self.policy().enforced_style("percent") == "percent"
                && !invalid_percent_array_contents(elements, self)
        );
        let replacement = build_bracketed_array(node, elements, self);
        let message = bracketed_word_array_message(&replacement);
        add_offense!(self, node.location(), message: message, |corrector| {
            corrector.replace(node.location(), replacement);
        });
    }

    fn within_matrix_of_complex_content(&mut self) -> bool {
        let Some(parent) = self.parent().and_then(Node::as_array_node) else {
            return false;
        };
        let key = (
            parent.location().start_offset(),
            parent.location().end_offset(),
        );
        if let Some(cached) = self.state.matrix_of_complex_content.get(&key) {
            return *cached;
        }
        let complex = matrix_of_complex_content(&parent, self);
        self.state.matrix_of_complex_content.insert(key, complex);
        complex
    }
}

fn complex_content(elements: &[Node<'_>], context: &CopContext<'_, '_>) -> bool {
    elements
        .iter()
        .filter(|element| element.as_string_node().is_some())
        .any(|element| !simple_word(element, context))
}

fn invalid_percent_array_contents(
    elements: &[Node<'_>],
    context: &CopContext<'_, '_>,
) -> bool {
    elements.iter().any(|element| {
        element.as_string_node().is_some_and(|string| {
            !valid_utf8(string.unescaped())
                || string.unescaped().contains(&b' ')
                // Prism preserves the escape marker for whitespace in a `%w`
                // element. RuboCop asks whether the semantic string contains
                // whitespace, so inspect that spelling as well.
                || context.source_file().node(element).contains("\\ ")
        })
    })
}

fn bracketed_word_array_message(replacement: &str) -> String {
    if replacement == "[]" {
        "Use `[]` for an array of words.".to_string()
    } else if replacement.contains('\n') {
        "Use an array literal `[...]` for an array of words.".to_string()
    } else {
        format!("Use `{replacement}` for an array of words.")
    }
}

fn valid_utf8(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes).is_ok()
}

fn simple_word(node: &Node<'_>, context: &CopContext<'_, '_>) -> bool {
    let Some(string) = node.as_string_node() else {
        return false;
    };
    let Ok(value) = std::str::from_utf8(string.unescaped()) else {
        return false;
    };
    if value.is_empty() || value.contains(' ') {
        return false;
    }
    if let Some(regex) = word_regex(context) {
        if regex.contains("\\p{Word}") {
            static DEFAULT_WORD: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
            let regex = DEFAULT_WORD.get_or_init(|| {
                regex::Regex::new(r"\A(?:[\w\n\t]|\w-\w)+\z")
                    .expect("the pinned Style/WordArray expression is valid")
            });
            return regex.is_match(value);
        }
        if regex.contains("\\S+") {
            return !value.chars().any(char::is_whitespace);
        }
        if regex.contains("@.") {
            return value.chars().all(|character| {
                character.is_alphanumeric()
                    || matches!(character, '_' | '@' | '.' | '-' | '\n' | '\t')
            });
        }
        if regex.contains("\\[") && regex.contains("\\]") {
            return value.chars().all(|character| {
                character.is_alphanumeric()
                    || matches!(character, '_' | '[' | ']' | '(' | ')' | ' ')
            });
        }
    }
    let mut previous_word = false;
    for character in value.chars() {
        if character == '-' {
            if !previous_word {
                return false;
            }
            previous_word = false;
        } else if character.is_alphanumeric()
            || character == '_'
            || matches!(character, '\n' | '\t')
        {
            previous_word = true;
        } else {
            return false;
        }
    }
    previous_word
}

fn word_regex<'a>(context: &'a CopContext<'_, '_>) -> Option<&'a str> {
    context
        .config_map("WordRegex")
        .and_then(|values| values.get("$regexp"))
        .map(String::as_str)
        .or_else(|| context.config_value("WordRegex"))
}

fn bracket_array_has_comment(
    node: &ruby_prism::ArrayNode<'_>,
    context: &CopContext<'_, '_>,
) -> bool {
    let location = node.location();
    context
        .source_file()
        .comment_ranges()
        .iter()
        .any(|comment| comment.start < location.end_offset() && comment.end > location.start_offset())
}

fn matrix_of_complex_content(
    array: &ruby_prism::ArrayNode<'_>,
    context: &CopContext<'_, '_>,
) -> bool {
    let rows = array.elements().iter().collect::<Vec<_>>();
    !rows.is_empty()
        && rows.iter().all(|row| row.as_array_node().is_some())
        && rows.iter().any(|row| {
            row.as_array_node().is_some_and(|array| {
                let elements = array.elements().iter().collect::<Vec<_>>();
                complex_content(&elements, context)
            })
        })
}

fn percent_word_array(
    node: &ruby_prism::ArrayNode<'_>,
    elements: &[Node<'_>],
    context: &CopContext<'_, '_>,
) -> Option<String> {
    let mut wide = false;
    let mut words = Vec::with_capacity(elements.len());
    for element in elements {
        let string = element.as_string_node()?;
        let value = std::str::from_utf8(string.unescaped()).ok()?;
        if value.chars().any(char::is_control)
            || context.source_encoding() == SourceEncoding::UsAscii && !value.is_ascii()
        {
            wide = true;
        }
        words.push(escape_percent_word(value, wide, context));
    }
    if wide {
        words = elements
            .iter()
            .map(|element| {
                let string = element.as_string_node().expect("validated string element");
                escape_percent_word(
                    std::str::from_utf8(string.unescaped()).unwrap_or(""),
                    true,
                    context,
                )
            })
            .collect();
    }
    let delimiters = context
        .related_config_map("Style/PercentLiteralDelimiters", "PreferredDelimiters")
        .and_then(|values| values.get(if wide { "%W" } else { "%w" }))
        .map(String::as_str)
        .unwrap_or("[]");
    let (open, close) = delimiters.split_at(1);
    let source = context.source_file().node(&node.as_node());
    let multiline = source.contains('\n');
    let body = if multiline {
        format_percent_multiline(node, elements, &words, context)
    } else {
        words.join(" ")
    };
    Some(format!(
        "%{}{open}{body}{close}",
        if wide { 'W' } else { 'w' }
    ))
}

fn escape_percent_word(value: &str, wide: bool, context: &CopContext<'_, '_>) -> String {
    let delimiters = context
        .related_config_map("Style/PercentLiteralDelimiters", "PreferredDelimiters")
        .and_then(|values| values.get(if wide { "%W" } else { "%w" }))
        .map(String::as_str)
        .unwrap_or("[]");
    let (open, close) = delimiters.split_at(1);
    let mut result = String::new();
    let mut balance = 0usize;
    let mut closing_delimiters = value.matches(close).count();
    for character in value.chars() {
        let escaped = match character {
            '\n' if wide => "\\n".to_string(),
            '\t' if wide => "\\t".to_string(),
            '\r' if wide => "\\r".to_string(),
            '\u{08}' if wide => "\\b".to_string(),
            '\u{0b}' if wide => "\\v".to_string(),
            '\u{0c}' if wide => "\\f".to_string(),
            character
                if context.source_encoding() == SourceEncoding::UsAscii
                    && !character.is_ascii() =>
            {
                if character as u32 <= 0xffff {
                    format!("\\u{:04X}", character as u32)
                } else {
                    format!("\\u{{{:X}}}", character as u32)
                }
            }
            character if character.is_control() => format!("\\u{:04X}", character as u32),
            character => character.to_string(),
        };
        if escaped == open {
            if open != close && closing_delimiters > balance {
                balance += 1;
                result.push_str(&escaped);
            } else {
                result.push('\\');
                result.push_str(&escaped);
            }
        } else if escaped == close && balance > 0 && open != close {
            balance -= 1;
            closing_delimiters = closing_delimiters.saturating_sub(1);
            result.push_str(&escaped);
        } else if escaped == close || escaped == open && open == close {
            closing_delimiters = closing_delimiters.saturating_sub(1);
            result.push('\\');
            result.push_str(&escaped);
        } else {
            result.push_str(&escaped);
        }
    }
    result
}

fn format_percent_multiline(
    node: &ruby_prism::ArrayNode<'_>,
    elements: &[Node<'_>],
    words: &[String],
    context: &CopContext<'_, '_>,
) -> String {
    let opening_end = node
        .opening_loc()
        .map_or(node.location().start_offset() + 1, |loc| loc.end_offset());
    let closing_start = node
        .closing_loc()
        .map_or(node.location().end_offset().saturating_sub(1), |loc| {
            loc.start_offset()
        });
    let mut output = String::new();
    let prefix = &context.source()[opening_end..elements[0].location().start_offset()];
    if prefix.contains('\n') {
        output.push_str(prefix.rsplit_once('\n').map_or("\n", |(_, indent)| {
            if indent.is_empty() {
                "\n"
            } else {
                ""
            }
        }));
        if !prefix.ends_with('\n') {
            output.push('\n');
            output.push_str(prefix.rsplit('\n').next().unwrap_or(""));
        }
    }
    output.push_str(&words[0]);
    for (index, pair) in elements.windows(2).enumerate() {
        let gap =
            &context.source()[pair[0].location().end_offset()..pair[1].location().start_offset()];
        if let Some((_, indent)) = gap.rsplit_once('\n') {
            output.push('\n');
            output.push_str(indent);
        } else {
            output.push(' ');
        }
        output.push_str(&words[index + 1]);
    }
    let suffix = &context.source()[elements.last().unwrap().location().end_offset()..closing_start];
    if let Some((_, indent)) = suffix.rsplit_once('\n') {
        output.push('\n');
        output.push_str(indent);
    }
    output
}

fn build_bracketed_array(
    node: &ruby_prism::ArrayNode<'_>,
    elements: &[Node<'_>],
    context: &CopContext<'_, '_>,
) -> String {
    if elements.is_empty() {
        return "[]".to_string();
    }
    let words = elements
        .iter()
        .map(|element| bracketed_word(element, context))
        .collect::<Vec<_>>();
    let opening_end = node
        .opening_loc()
        .map_or(node.location().start_offset() + 3, |loc| loc.end_offset());
    let closing_start = node
        .closing_loc()
        .map_or(node.location().end_offset().saturating_sub(1), |loc| {
            loc.start_offset()
        });
    let mut replacement = String::from("[");
    replacement.push_str(
        &context.source()[opening_end..elements[0].location().start_offset()],
    );
    replacement.push_str(&words[0]);
    for (pair, word) in elements.windows(2).zip(words.iter().skip(1)) {
        replacement.push(',');
        replacement.push_str(
            &context.source()[pair[0].location().end_offset()..pair[1].location().start_offset()],
        );
        replacement.push_str(word);
    }
    replacement.push_str(
        &context.source()[elements.last().unwrap().location().end_offset()..closing_start],
    );
    replacement.push(']');
    replacement
}

fn bracketed_word(node: &Node<'_>, context: &CopContext<'_, '_>) -> String {
    if let Some(string) = node.as_string_node() {
        let value = String::from_utf8_lossy(string.unescaped());
        if value.contains('\'') || value.chars().any(char::is_control) {
            return format!("\"{}\"", escape_double_quoted(&value));
        }
        return format!("'{}'", value.replace('\\', "\\\\"));
    }
    let source = context.source_file().node(node);
    format!("\"{}\"", source.replace('"', "\\\""))
}

fn escape_double_quoted(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
        .replace('\u{08}', "\\b")
        .replace('\u{0b}', "\\v")
        .replace('\u{0c}', "\\f")
}
