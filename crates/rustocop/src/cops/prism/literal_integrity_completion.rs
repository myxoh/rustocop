use std::collections::HashSet;

use super::*;
use crate::rubocop::ast::node::core::NodeRef as RubocopNodeRef;
use crate::rubocop::ast::prism::convert as convert_rubocop_ast;

mod helpers;
use helpers::*;

define_cops! {
    DuplicateSetElement => "Lint/DuplicateSetElement" => compatibility_source(duplicate_set_element),
    NumericOperationWithConstantResult => "Lint/NumericOperationWithConstantResult" => compatibility_source(numeric_constant_result),
    SymbolConversion => "Lint/SymbolConversion" => compatibility_source(symbol_conversion),
    DoubleNegation => "Style/DoubleNegation" => compatibility_prism_call(double_negation),
    EmptyLiteral => "Style/EmptyLiteral" => compatibility_source(empty_literal),
}

fn duplicate_set_element(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    let source = context.source();
    report_duplicate_percent_symbol_sets(source, context);
    let comments = context.source_file().comment_ranges();
    let literals = context.source_file().literal_ranges();
    let mut inspected = HashSet::new();
    for (open, _) in source.match_indices('[') {
        if source[..open].ends_with("%i")
            || !inspected.insert(open)
            || ranges_contain(&comments, open)
            || ranges_contain(&literals, open)
        {
            continue;
        }
        let Some(close) = super::source_syntax::matching_delimiter(source, open, b'[', b']') else {
            continue;
        };
        let Some(name) = set_constructor_name(source, open, close) else {
            continue;
        };
        report_duplicate_set_entries(source, open, close, name, context);
    }
}

fn report_duplicate_percent_symbol_sets(
    source: &str,
    context: &mut CompatibilityCopContext<'_, '_, '_>,
) {
    let comments = context.source_file().comment_ranges();
    let literals = context.source_file().literal_ranges();
    let mut search = 0;
    while let Some(relative) = source[search..].find("%i[") {
        let literal_start = search + relative;
        if ranges_contain(&comments, literal_start)
            || literals
                .iter()
                .any(|range| range.start < literal_start && literal_start < range.end)
        {
            search = literal_start + 3;
            continue;
        }
        let open = search + relative + 2;
        let Some(close_relative) = source[open + 1..].find(']') else {
            break;
        };
        let close = open + 1 + close_relative;
        let before = &source[..open.saturating_sub(2)];
        let after = &source[close + 1..];
        let name = if before.ends_with("SortedSet.new(") {
            "SortedSet"
        } else if before.ends_with("Set.new(")
            || after.starts_with(".to_set")
            || after.starts_with("&.to_set")
        {
            "Set"
        } else {
            search = close + 1;
            continue;
        };
        let body = &source[open + 1..close];
        let mut seen = Vec::<&str>::new();
        let mut cursor = 0;
        for value in body.split_whitespace() {
            let relative_start = body[cursor..].find(value).unwrap_or(0) + cursor;
            let value_start = open + 1 + relative_start;
            if seen.contains(&value) {
                let removal_start = source[..value_start]
                    .rfind(char::is_whitespace)
                    .unwrap_or(value_start);
                context.remove(
                    format!("Remove the duplicate element in {name}."),
                    value_start..value_start + value.len(),
                    removal_start..value_start + value.len(),
                );
            } else {
                seen.push(value);
            }
            cursor = relative_start + value.len();
        }
        search = close + 1;
    }
}

fn ranges_contain(ranges: &[std::ops::Range<usize>], offset: usize) -> bool {
    ranges
        .iter()
        .any(|range| range.start <= offset && offset < range.end)
}

fn set_constructor_name(source: &str, open: usize, close: usize) -> Option<&'static str> {
    let before = &source[..open];
    if before.ends_with("SortedSet") || before.ends_with("SortedSet.new(") {
        Some("SortedSet")
    } else if before.ends_with("Set") || before.ends_with("Set.new(") {
        let boundary = before
            .len()
            .saturating_sub(if before.ends_with("Set.new(") {
                "Set.new(".len()
            } else {
                "Set".len()
            });
        if boundary > 0 && source.as_bytes()[boundary - 1].is_ascii_alphanumeric() {
            None
        } else {
            Some("Set")
        }
    } else {
        let after = &source[close + 1..];
        (after.starts_with(".to_set") || after.starts_with("&.to_set")).then_some("Set")
    }
}

fn report_duplicate_set_entries(
    source: &str,
    open: usize,
    close: usize,
    name: &str,
    context: &mut CompatibilityCopContext<'_, '_, '_>,
) {
    let body = &source[open + 1..close];
    let mut seen = Vec::new();
    for (position, entry) in top_level_entries(body) {
        let value = entry.trim();
        let leading = entry.len() - entry.trim_start().len();
        if !stable_set_element(value, &source[..open]) {
            continue;
        }
        if seen.contains(&value) {
            let value_start = open + 1 + position + leading;
            let comma_start = source[..value_start].rfind(',').unwrap_or(value_start);
            context.remove(
                format!("Remove the duplicate element in {name}."),
                value_start..value_start + value.len(),
                comma_start..value_start + value.len(),
            );
        } else {
            seen.push(value);
        }
    }
}

fn stable_set_element(value: &str, preceding_source: &str) -> bool {
    if value.is_empty()
        || value.contains("#{")
        || value.contains("&.")
        || value.contains(" ? ")
        || value.contains(['(', ')'])
    {
        return false;
    }
    let first = value.as_bytes()[0];
    if matches!(first, b':' | b'@' | b'\'' | b'"')
        || first.is_ascii_uppercase()
        || first.is_ascii_digit()
        || matches!(value, "true" | "false" | "nil")
    {
        return true;
    }
    preceding_source.lines().any(|line| {
        let line = line.trim_start();
        line.strip_prefix(value)
            .is_some_and(|tail| tail.trim_start().starts_with('='))
    })
}

fn numeric_constant_result(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    let literal_ranges = context.source_file().literal_ranges();
    for (offset, line) in context.source_file().lines() {
        let code = line.split('#').next().unwrap_or(line).trim();
        let start = offset + line.find(code).unwrap_or(0);
        if ranges_contain(&literal_ranges, start) {
            continue;
        }
        let replacement = if let Some(left) = code.strip_suffix(" * 0") {
            (!left.trim().bytes().all(|b| b.is_ascii_digit())).then(|| "0".to_string())
        } else if let Some(right) = code.strip_prefix("0 * ") {
            (!right.trim().bytes().all(|b| b.is_ascii_digit())).then(|| "0".to_string())
        } else if code.ends_with(" ** 0") {
            Some("1".to_string())
        } else if code.ends_with(" & 0") {
            Some("0".to_string())
        } else if let Some((left, right)) = code.split_once(" / ") {
            (left.trim() == right.trim()).then(|| "1".to_string())
        } else if let Some(left) = code.strip_suffix(" *= 0") {
            (!left.trim().is_empty()).then(|| format!("{} = 0", left.trim()))
        } else if let Some((left, right)) = code.split_once(" /= ") {
            (left.trim() == right.trim()).then(|| format!("{} = 1", left.trim()))
        } else if code.ends_with(" **= 0") {
            Some(format!("{} = 1", code.trim_end_matches(" **= 0").trim()))
        } else if code.ends_with(".*(0)") || code.ends_with("&.*(0)") {
            Some("0".to_string())
        } else if let Some((left, right)) =
            code.split_once("&./(").or_else(|| code.split_once("./("))
        {
            (left.trim() == right.trim_end_matches(')').trim()).then(|| "1".to_string())
        } else {
            None
        };
        if let Some(replacement) = replacement {
            context.replace(
                "Numeric operation with a constant result detected.",
                start..start + code.len(),
                start..start + code.len(),
                replacement,
            );
        }
    }
}

fn symbol_conversion(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    #[derive(Default)]
    struct Symbols {
        calls: Vec<(std::ops::Range<usize>, std::ops::Range<usize>, bool)>,
        symbols: Vec<(std::ops::Range<usize>, Vec<u8>)>,
    }
    impl<'pr> Visit<'pr> for Symbols {
        fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
            if matches!(node.name().as_slice(), b"to_sym" | b"intern") {
                if let Some(receiver) = node.receiver().filter(|receiver| {
                    receiver.as_string_node().is_some()
                        || receiver.as_interpolated_string_node().is_some()
                        || receiver.as_symbol_node().is_some()
                }) {
                    self.calls.push((
                        node.location().start_offset()..node.location().end_offset(),
                        receiver.location().start_offset()..receiver.location().end_offset(),
                        receiver.as_interpolated_string_node().is_some(),
                    ));
                }
            }
            ruby_prism::visit_call_node(self, node);
        }

        fn visit_symbol_node(&mut self, node: &ruby_prism::SymbolNode<'pr>) {
            self.symbols.push((
                node.location().start_offset()..node.location().end_offset(),
                node.unescaped().to_vec(),
            ));
            ruby_prism::visit_symbol_node(self, node);
        }
    }

    let source = context.source();
    let mut nodes = Symbols::default();
    nodes.visit(&parse(source.as_bytes()).node());
    let call_receivers = nodes
        .calls
        .iter()
        .map(|(_, receiver, _)| receiver.clone())
        .collect::<Vec<_>>();
    for (call, receiver, interpolated) in nodes.calls {
        let receiver_source = &source[receiver.clone()];
        let replacement = if interpolated {
            format!(":{}", joined_interpolated_string(receiver_source))
        } else {
            let value = nodes
                .symbols
                .iter()
                .find(|(range, _)| *range == receiver)
                .map(|(_, value)| String::from_utf8_lossy(value).into_owned())
                .or_else(|| {
                    let parsed = parse(receiver_source.as_bytes());
                    parsed
                        .node()
                        .as_program_node()
                        .and_then(|program| program.statements().body().iter().next())
                        .and_then(|node| node.as_string_node())
                        .map(|string| String::from_utf8_lossy(string.unescaped()).into_owned())
                })
                .unwrap_or_default();
            symbol_inspect(&value)
        };
        context.replace(
            format!("Unnecessary symbol conversion; use `{replacement}` instead."),
            call.clone(),
            call,
            replacement,
        );
    }

    for (range, value) in nodes.symbols {
        if call_receivers.contains(&range) {
            continue;
        }
        let raw = &source[range.clone()];
        let hash_label = (raw.starts_with('\'') || raw.starts_with('"')) && raw.ends_with(':');
        if hash_label && context.policy().enforced_style("strict") == "consistent" {
            continue;
        }
        let value = String::from_utf8_lossy(&value);
        if value.ends_with('=') {
            continue;
        }
        let replacement = if hash_label {
            if value.ends_with('=') {
                continue;
            }
            if !value.chars().next().is_some_and(|character| character.is_alphanumeric() || character == '_') {
                continue;
            }
            symbol_inspect(&value).trim_start_matches(':').to_string()
        } else {
            symbol_inspect(&value)
        };
        if replacement.contains('"') {
            continue;
        }
        let displayed = if hash_label { format!("{replacement}:") } else { replacement.clone() };
        let correction = if hash_label { replacement } else { displayed.clone() };
        let correction_range = if hash_label {
            range.start..range.end - 1
        } else {
            range.clone()
        };
        if source[correction_range.clone()] == correction
            || (!hash_label && !raw.starts_with(":\'") && !raw.starts_with(":\"")) {
            continue;
        }
        context.replace(
            format!("Unnecessary symbol conversion; use `{displayed}` instead."),
            correction_range.clone(),
            correction_range,
            correction,
        );
    }

    if context.policy().enforced_style("strict") == "consistent" {
        check_symbol_hash_labels(context);
    }
}

fn symbol_inspect(value: &str) -> String {
    if bare_symbol_syntax(value) {
        format!(":{value}")
    } else {
        format!(":\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

fn joined_interpolated_string(source: &str) -> String {
    let lines = source.lines().map(str::trim).collect::<Vec<_>>();
    if lines.len() <= 1 {
        return source.to_string();
    }
    let quote = lines.first().and_then(|line| line.chars().next()).unwrap_or('"');
    let mut body = String::new();
    for (index, line) in lines.iter().enumerate() {
        let mut part = line.trim_end_matches('\\');
        if index > 0 {
            part = part.strip_prefix(quote).unwrap_or(part);
        }
        if index + 1 < lines.len() {
            part = part.strip_suffix(quote).unwrap_or(part);
        }
        body.push_str(part);
    }
    body
}

fn bare_symbol_syntax(value: &str) -> bool {
    const OPERATORS: &[&str] = &[
        "==", "===", "!=", "=~", "!~", "<=>", "<", "<=", ">", ">=", "+", "-", "*",
        "/", "%", "**", "<<", ">>", "&", "|", "^", "~", "!", "[]", "[]=", "+@", "-@",
        "`",
    ];
    if OPERATORS.contains(&value) {
        return true;
    }
    let mut characters = value.chars();
    let Some(first) = characters.next() else { return false };
    if !(first.is_alphabetic() || matches!(first, '_' | '@' | '$')) {
        return false;
    }
    let mut rest = characters.collect::<String>();
    if matches!(first, '@' | '$') {
        let Some(variable_start) = rest.chars().next() else { return false };
        if !(variable_start.is_alphabetic() || variable_start == '_') {
            return false;
        }
    }
    if rest.ends_with(['!', '?', '=']) {
        rest.pop();
    }
    rest.chars().all(|character| character.is_alphanumeric() || character == '_')
}

fn bare_symbol_name(value: &str, allow_suffix: bool) -> bool {
    !value.is_empty()
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphanumeric()
                || byte == b'_'
                || (allow_suffix && index + 1 == value.len() && matches!(byte, b'!' | b'?' | b'='))
        })
}

fn check_symbol_hash_labels(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    let source = context.source();
    let style = context.policy().enforced_style("strict");
    let quoted = quoted_hash_labels(source);
    let quote_all = style == "consistent"
        && quoted
            .iter()
            .any(|label| !bare_symbol_name(&source[label.content.clone()], false));

    if !quote_all {
        for label in quoted {
            let value = &source[label.content.clone()];
            if !bare_symbol_name(value, true) || value.ends_with('=') {
                continue;
            }
            let replacement = format!("{value}:");
            context.replace(
                format!("Unnecessary symbol conversion; use `{replacement}` instead."),
                label.start..label.close + 1,
                label.start..label.close + 2,
                replacement,
            );
        }
        return;
    }

    for label in unquoted_hash_labels(source) {
        let value = &source[label.start..label.end];
        let replacement = format!("\"{value}\":");
        context.replace(
            format!(
                "Symbol hash key should be quoted for consistency; use `{replacement}` instead."
            ),
            label.start..label.end,
            label.start..label.end + 1,
            replacement,
        );
    }
}

struct QuotedHashLabel {
    start: usize,
    close: usize,
    content: std::ops::Range<usize>,
}

fn quoted_hash_labels(source: &str) -> Vec<QuotedHashLabel> {
    let bytes = source.as_bytes();
    let mut labels = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let quote = bytes[at];
        if !matches!(quote, b'\'' | b'"') {
            at += 1;
            continue;
        }
        let mut close = at + 1;
        while close < bytes.len() && bytes[close] != quote {
            close += 1;
        }
        if (at == 0 || bytes[at - 1] != b':') && bytes.get(close + 1) == Some(&b':') {
            labels.push(QuotedHashLabel {
                start: at,
                close,
                content: at + 1..close,
            });
        }
        at = close.saturating_add(1);
    }
    labels
}

struct UnquotedHashLabel {
    start: usize,
    end: usize,
}

fn unquoted_hash_labels(source: &str) -> Vec<UnquotedHashLabel> {
    let bytes = source.as_bytes();
    let mut labels = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        if !bytes[at].is_ascii_alphabetic()
            || at > 0 && !matches!(bytes[at - 1], b'{' | b',' | b' ' | b'\t' | b'\n')
        {
            at += 1;
            continue;
        }
        let start = at;
        at += 1;
        while at < bytes.len()
            && (bytes[at].is_ascii_alphanumeric() || matches!(bytes[at], b'_' | b'!' | b'?'))
        {
            at += 1;
        }
        if bytes.get(at) == Some(&b':') && bytes.get(at + 1) != Some(&b':') {
            labels.push(UnquotedHashLabel { start, end: at });
        }
    }
    labels
}

fn double_negation(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) != b"!" || argument_count(node) != 0 {
        return;
    }
    let Some(inner) = node.receiver().and_then(|receiver| receiver.as_call_node()) else {
        return;
    };
    if call_name(&inner) != b"!" || argument_count(&inner) != 0 {
        return;
    }
    let Some(selector) = node.message_loc() else {
        return;
    };
    if selector.as_slice() != b"!" || selector.start_offset() != node.location().start_offset() {
        return;
    }
    let allowed_style = context.policy().enforced_style("allowed_in_returns") == "allowed_in_returns";
    if allowed_style && double_negation_return_value(node, context) {
        return;
    }
    let Some(value) = inner.receiver() else {
        return;
    };
    context.replace(
        "Avoid the use of double negation (`!!`).",
        selector.start_offset()..selector.end_offset(),
        node.location(),
        format!("!{}.nil?", context.source_file().node(&value)),
    );
}

fn double_negation_return_value(node: &CallNode<'_>, context: &CopContext<'_, '_>) -> bool {
    if context
        .parent()
        .is_some_and(|parent| parent.as_return_node().is_some())
    {
        return true;
    }
    let definition_body = context
        .ancestors()
        .iter()
        .rev()
        .find_map(Node::as_def_node)
        .and_then(|definition| definition.body());
    let define_method_body = context
        .ancestors()
        .iter()
        .rev()
        .find_map(Node::as_block_node)
        .filter(|block| {
            let start = block.location().start_offset();
            let line_start = context.source()[..start]
                .rfind('\n')
                .map_or(0, |offset| offset + 1);
            let prefix = &context.source()[line_start..start];
            prefix.contains("define_method") || prefix.contains("define_singleton_method")
        })
        .and_then(|block| block.body());
    let Some(body) = define_method_body.or(definition_body) else {
        return false;
    };
    let Some((last, sequence_body)) = return_body_last_with_sequence(body) else {
        return false;
    };
    let conditional = context.ancestors().iter().rev().find(|ancestor| {
        (ancestor.as_if_node().is_some()
            || ancestor.as_unless_node().is_some()
            || ancestor.as_case_node().is_some()
            || ancestor.as_case_match_node().is_some())
            && ancestor.location().start_offset() <= node.location().start_offset()
            && node.location().end_offset() <= ancestor.location().end_offset()
    });
    if let Some(conditional) = conditional {
        let reference_end = if sequence_body {
            last.location().end_offset()
        } else {
            let fallback_end = last.location().end_offset();
            double_negation_last_child(last)
                .map_or(fallback_end, |(child, _)| child.location().end_offset())
        };
        let line_start = context.source()[..node.location().start_offset()]
            .rfind('\n')
            .map_or(0, |offset| offset + 1);
        let prefix = context.source()[line_start..node.location().start_offset()].trim();
        let nested_in_call = context
            .ancestors()
            .iter()
            .rev()
            .take_while(|ancestor| {
                ancestor.location().start_offset() >= conditional.location().start_offset()
            })
            .any(|ancestor| {
                ancestor.as_arguments_node().is_some() || ancestor.as_call_node().is_some()
            });
        let standalone_or_collection = (prefix.is_empty()
            || prefix.starts_with('[')
            || prefix.starts_with('{'))
            && !nested_in_call;
        let nested_elsif = context.source()
            [conditional.location().start_offset()..node.location().start_offset()]
            .lines()
            .any(|line| line.trim_start().starts_with("elsif "));
        let conditional_last_line = line_at(context.source(), conditional.location().end_offset())
            .saturating_sub(usize::from(nested_elsif && sequence_body));
        (!standalone_or_collection
            || conditional_branch_tail(
                context.source(),
                node.location().end_offset(),
                conditional.location().end_offset(),
            ))
            && line_at(context.source(), reference_end) <= conditional_last_line
    } else {
        let child = if sequence_body {
            Some((last, false))
        } else {
            double_negation_last_child(last)
        };
        let Some((last_child, collection_child)) = child else {
            return false;
        };
        !collection_child
            && last_child.as_hash_node().is_none()
            && line_at(context.source(), last_child.location().start_offset())
            <= line_at(context.source(), node.location().start_offset())
    }
}

fn double_negation_last_child(node: Node<'_>) -> Option<(Node<'_>, bool)> {
    if let Some(call) = node.as_call_node() {
        if let Some(block) = call.block().and_then(|block| block.as_block_node()) {
            return block.body().map(|body| (body, false));
        }
        if let Some(argument) = call
            .arguments()
            .and_then(|arguments| arguments.arguments().iter().last())
        {
            if let Some(hash) = argument.as_keyword_hash_node() {
                return hash
                    .elements()
                    .iter()
                    .last()
                    .map(|element| (element, true));
            }
            return Some((argument, false));
        }
        return call.receiver().map(|receiver| (receiver, false));
    }
    if let Some(and) = node.as_and_node() {
        return Some((and.right(), false));
    }
    if let Some(or) = node.as_or_node() {
        return Some((or.right(), false));
    }
    if let Some(hash) = node.as_hash_node() {
        return hash.elements().iter().last().map(|element| (element, true));
    }
    if let Some(hash) = node.as_keyword_hash_node() {
        return hash.elements().iter().last().map(|element| (element, true));
    }
    if let Some(array) = node.as_array_node() {
        return array.elements().iter().last().map(|element| (element, true));
    }
    if let Some(pair) = node.as_assoc_node() {
        return Some((pair.value(), false));
    }
    if let Some(write) = node.as_instance_variable_or_write_node() {
        return Some((write.value(), false));
    }
    if let Some(parentheses) = node.as_parentheses_node() {
        return parentheses.body().and_then(return_body_last).map(|child| (child, false));
    }
    Some((node, false))
}

fn conditional_branch_tail(source: &str, node_end: usize, conditional_end: usize) -> bool {
    let next_line = source[node_end..]
        .find('\n')
        .map_or(node_end, |offset| node_end + offset + 1);
    for line in source[next_line.min(conditional_end)..conditional_end].lines() {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with('#')
            || line
                .bytes()
                .all(|byte| matches!(byte, b']' | b'}' | b')' | b','))
        {
            continue;
        }
        return line == "end"
            || line.starts_with("else")
            || line.starts_with("elsif ")
            || line.starts_with("when ")
            || line.starts_with("in ");
    }
    true
}

fn return_body_last(body: Node<'_>) -> Option<Node<'_>> {
    if let Some(statements) = body.as_statements_node() {
        return statements.body().iter().last();
    }
    if let Some(begin) = body.as_begin_node() {
        return begin
            .statements()
            .and_then(|statements| statements.body().iter().last());
    }
    Some(body)
}

fn return_body_last_with_sequence(body: Node<'_>) -> Option<(Node<'_>, bool)> {
    if let Some(statements) = body.as_statements_node() {
        let expressions = statements.body().iter().collect::<Vec<_>>();
        let sequence = expressions.len() > 1;
        return expressions.into_iter().last().map(|last| (last, sequence));
    }
    if let Some(begin) = body.as_begin_node() {
        let expressions = begin
            .statements()
            .map(|statements| statements.body().iter().collect::<Vec<_>>())
            .unwrap_or_default();
        let sequence = expressions.len() > 1;
        return expressions.into_iter().last().map(|last| (last, sequence));
    }
    Some((body, false))
}

fn line_at(source: &str, offset: usize) -> usize {
    source[..offset].bytes().filter(|byte| *byte == b'\n').count()
}

fn empty_literal(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    let source = context.source();
    let string_literal = if context.related_config_value("Style/StringLiterals", "EnforcedStyle")
        == Some("double_quotes")
    {
        "\"\""
    } else {
        "''"
    };
    let frozen_comment = source.lines().take(2).find_map(|line| {
        let value = line.trim().strip_prefix("# frozen_string_literal:")?.trim();
        matches!(value, "true" | "false").then_some(value == "true")
    });
    let frozen_strings = frozen_comment.unwrap_or_else(|| {
        match context.related_config_value("AllCops", "StringLiteralsFrozenByDefault") {
            Some("true") => true,
            Some("false") => false,
            _ => {
                context.related_config_value("Style/FrozenStringLiteralComment", "Enabled")
                    == Some("true")
                    && (context.related_config_explicit(
                        "Style/FrozenStringLiteralComment",
                        "Enabled",
                    ) || context.related_config_value("AllCops", "DisabledByDefault")
                        != Some("true"))
            }
        }
    });
    let parsed = ruby_prism::parse(source.as_bytes());
    let (ast, root) = convert_rubocop_ast(source, &parsed.node());
    let Some(root) = root.map(|root| ast.node(root)) else { return };
    for node in root.each_node(&["send"]) {
        let Some((kind, literal)) = empty_literal_kind(node, frozen_strings, string_literal) else {
            continue;
        };
        let current = node.source().unwrap_or_default();
        let message = if kind == "string" {
            format!("Use string literal `{literal}` instead of `String.new`.")
        } else {
            format!("Use {kind} literal `{literal}` instead of `{current}`.")
        };
        let Some(chars) = node.source_range() else { continue };
        let offense = empty_literal_character_range_to_byte(source, chars);
        if kind == "hash" && empty_literal_unparenthesized_first_argument(node) {
            let Some(parent) = node.parent() else { continue };
            let arguments = parent.arguments();
            let Some(first) = arguments.first().and_then(|argument| argument.source_range()) else { continue };
            let Some(last) = arguments.last().and_then(|argument| argument.source_range()) else { continue };
            let replacement_chars = first.start.saturating_sub(1)..last.end;
            let replacement_range = empty_literal_character_range_to_byte(source, replacement_chars);
            let tail = arguments
                .iter()
                .skip(1)
                .filter_map(|argument| argument.source())
                .collect::<Vec<_>>();
            let replacement = if tail.is_empty() {
                "({})".to_string()
            } else {
                format!("({{}}, {})", tail.join(", "))
            };
            context.replace(message, offense, replacement_range, replacement);
        } else {
            context.replace(message, offense.clone(), offense, literal);
        }
    }
}

fn empty_literal_kind(
    node: RubocopNodeRef<'_>,
    frozen_strings: bool,
    string_literal: &'static str,
) -> Option<(&'static str, &'static str)> {
    let method = node.method_name()?;
    let receiver = node.receiver();
    let arguments = node.arguments();
    let parent_is_block = node.parent().is_some_and(|parent| {
        matches!(parent.kind(), "block" | "numblock" | "itblock")
            && parent.send_node() == Some(node)
    });
    let empty_array = |argument: RubocopNodeRef<'_>| {
        argument.kind() == "array" && argument.child_nodes().is_empty()
    };
    if receiver.is_some_and(|receiver| receiver.global_const("Array")) {
        if method == "new"
            && !parent_is_block
            && (arguments.is_empty()
                || arguments.len() == 1 && empty_array(arguments[0]))
            || method == "[]" && arguments.is_empty()
        {
            return Some(("array", "[]"));
        }
    } else if receiver.is_none() && method == "Array" && arguments.len() == 1 && empty_array(arguments[0]) {
        return Some(("array", "[]"));
    }
    if receiver.is_some_and(|receiver| receiver.global_const("Hash")) {
        if method == "new" && !parent_is_block && arguments.is_empty()
            || method == "[]" && arguments.is_empty()
        {
            return Some(("hash", "{}"));
        }
    } else if receiver.is_none() && method == "Hash" && arguments.len() == 1 && empty_array(arguments[0]) {
        return Some(("hash", "{}"));
    }
    if !frozen_strings
        && receiver.is_some_and(|receiver| receiver.global_const("String"))
        && method == "new"
        && !parent_is_block
        && arguments.is_empty()
    {
        return Some(("string", string_literal));
    }
    None
}

fn empty_literal_unparenthesized_first_argument(node: RubocopNodeRef<'_>) -> bool {
    let Some(parent) = node.parent() else { return false };
    matches!(parent.kind(), "send" | "super" | "zsuper")
        && parent.first_argument() == Some(node)
        && !parent.parenthesized_call()
}

fn empty_literal_character_range_to_byte(
    source: &str,
    range: std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    let byte = |offset| source.char_indices().nth(offset).map_or(source.len(), |(byte, _)| byte);
    byte(range.start)..byte(range.end)
}
