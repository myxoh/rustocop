use ruby_prism::{BlockNode, CallNode, IfNode, Node, UnlessNode};

use super::*;

define_cops! {
    MapCompactWithConditionalBlock => "Style/MapCompactWithConditionalBlock" => rubocop_callbacks(
        MapCompactWithConditionalBlockRule,
        [on_send restrict [b"compact", b"filter_map"]]
    ),
}

impl MapCompactWithConditionalBlockRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        let (map_call, compact, range_end) = if node.name().as_slice() == b"compact" {
            return_unless!(argument_count(node) == 0);
            let Some(map) = node.receiver().and_then(|receiver| receiver.as_call_node()) else { return };
            return_unless!(matches!(map.name().as_slice(), b"map" | b"filter_map"));
            (map, true, node.location().end_offset())
        } else {
            return_if!(self.parent().and_then(Node::as_call_node).is_some_and(|parent| {
                parent.name().as_slice() == b"compact"
                    && parent.receiver().is_some_and(|receiver| receiver.location().start_offset() == node.location().start_offset())
            }));
            (node.as_node().as_call_node().expect("call node round trip"), false, node.location().end_offset())
        };
        let Some(block) = map_call.block().and_then(|block| block.as_block_node()) else { return };
        let Some(parameter) = single_block_parameter(&block, self.source_file()) else { return };
        let Some((condition, method)) =
            conditional_selection(&block, &parameter, self.source_file(), compact)
        else {
            return;
        };
        let Some(selector) = map_call.message_loc() else { return };
        let range = selector.start_offset()..range_end;
        let map_name = String::from_utf8_lossy(map_call.name().as_slice());
        let current = if compact {
            format!("{map_name} {{ ... }}.compact")
        } else {
            "filter_map { ... }".to_string()
        };
        let message = format!("Replace `{current}` with `{method}`.");
        let replacement = format!("{method} {{ |{parameter}| {} }}", self.source_file().node(&condition));
        add_offense!(self, range.clone(), message: message, |corrector| {
            corrector.replace(range, replacement);
        });
    }
}

fn single_block_parameter(block: &BlockNode<'_>, file: SourceFile<'_>) -> Option<String> {
    let parameters = block.parameters()?;
    let source = file.node(&parameters).trim();
    let parameter = source.trim_matches('|').trim();
    (!parameter.is_empty() && !parameter.contains([',', ';'])).then(|| parameter.to_string())
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum BranchValue {
    Returned,
    Skipped,
    Other,
}

fn conditional_selection<'pr>(
    block: &BlockNode<'pr>,
    parameter: &str,
    file: SourceFile<'_>,
    explicit_nil_is_skipped: bool,
) -> Option<(Node<'pr>, &'static str)> {
    let body = block.body()?;
    let Some(statements) = body.as_statements_node() else {
        return conditional_expression(&body, parameter, file, explicit_nil_is_skipped);
    };
    let expressions = statements.body().iter().collect::<Vec<_>>();
    if expressions.len() == 1 {
        return conditional_expression(
            &expressions[0],
            parameter,
            file,
            explicit_nil_is_skipped,
        );
    }
    if expressions.len() == 2 {
        let last = branch_value(&expressions[1], parameter, file, true);
        if let Some(if_node) = expressions[0].as_if_node() {
            let next = if_node.statements().and_then(|statements| statements.body().iter().next())?;
            if next.as_next_node().is_some() && if_node.subsequent().is_none() {
                let has_argument = next.as_next_node().and_then(|next| next.arguments()).is_some();
                let method = if has_argument { "select" } else { "reject" };
                if matches!(last, BranchValue::Returned | BranchValue::Skipped) {
                    return Some((if_node.predicate(), method));
                }
            }
        }
        if let Some(unless) = expressions[0].as_unless_node() {
            let next = unless.statements().and_then(|statements| statements.body().iter().next())?;
            if next.as_next_node().is_some() && unless.else_clause().is_none() {
                let has_argument = next.as_next_node().and_then(|next| next.arguments()).is_some();
                let method = if has_argument { "reject" } else { "select" };
                if matches!(last, BranchValue::Returned | BranchValue::Skipped) {
                    return Some((unless.predicate(), method));
                }
            }
        }
    }
    None
}

fn conditional_expression<'pr>(
    node: &Node<'pr>,
    parameter: &str,
    file: SourceFile<'_>,
    explicit_nil_is_skipped: bool,
) -> Option<(Node<'pr>, &'static str)> {
    if let Some(if_node) = node.as_if_node() {
        return if_selection(&if_node, parameter, file, explicit_nil_is_skipped);
    }
    if let Some(unless) = node.as_unless_node() {
        return unless_selection(&unless, parameter, file, explicit_nil_is_skipped);
    }
    None
}

fn if_selection<'pr>(
    node: &IfNode<'pr>,
    parameter: &str,
    file: SourceFile<'_>,
    explicit_nil_is_skipped: bool,
) -> Option<(Node<'pr>, &'static str)> {
    let truthy = node.statements().and_then(|statements| statements.body().iter().last())
        .map_or(BranchValue::Skipped, |value| branch_value(&value, parameter, file, explicit_nil_is_skipped));
    let falsey = match node.subsequent() {
        None => BranchValue::Skipped,
        Some(subsequent) if subsequent.as_if_node().is_some() => return None,
        Some(subsequent) => subsequent.as_else_node().and_then(|else_node| else_node.statements())
            .and_then(|statements| statements.body().iter().last())
            .map_or(BranchValue::Skipped, |value| branch_value(&value, parameter, file, explicit_nil_is_skipped)),
    };
    match (truthy, falsey) {
        (BranchValue::Returned, BranchValue::Skipped) => Some((node.predicate(), "select")),
        (BranchValue::Skipped, BranchValue::Returned) => Some((node.predicate(), "reject")),
        _ => None,
    }
}

fn unless_selection<'pr>(
    node: &UnlessNode<'pr>,
    parameter: &str,
    file: SourceFile<'_>,
    explicit_nil_is_skipped: bool,
) -> Option<(Node<'pr>, &'static str)> {
    let false_condition = node.statements().and_then(|statements| statements.body().iter().last())
        .map_or(BranchValue::Skipped, |value| branch_value(&value, parameter, file, explicit_nil_is_skipped));
    let true_condition = node.else_clause().and_then(|else_node| else_node.statements())
        .and_then(|statements| statements.body().iter().last())
        .map_or(BranchValue::Skipped, |value| branch_value(&value, parameter, file, explicit_nil_is_skipped));
    match (true_condition, false_condition) {
        (BranchValue::Returned, BranchValue::Skipped) => Some((node.predicate(), "select")),
        (BranchValue::Skipped, BranchValue::Returned) => Some((node.predicate(), "reject")),
        _ => None,
    }
}

fn branch_value(
    node: &Node<'_>,
    parameter: &str,
    file: SourceFile<'_>,
    explicit_nil_is_skipped: bool,
) -> BranchValue {
    if explicit_nil_is_skipped && node.as_nil_node().is_some() {
        return BranchValue::Skipped;
    }
    if let Some(next) = node.as_next_node() {
        let Some(argument) = next.arguments().and_then(|arguments| arguments.arguments().iter().next()) else {
            return BranchValue::Skipped;
        };
        return if file.node(&argument) == parameter {
            BranchValue::Returned
        } else if argument.as_nil_node().is_some() {
            BranchValue::Skipped
        } else {
            BranchValue::Other
        };
    }
    if file.node(node) == parameter {
        BranchValue::Returned
    } else {
        BranchValue::Other
    }
}
