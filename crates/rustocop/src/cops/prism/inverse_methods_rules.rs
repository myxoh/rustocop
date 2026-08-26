use std::collections::HashMap;

use ruby_prism::{BlockNode, CallNode, Node};

use super::*;

define_cops! {
    InverseMethods => "Style/InverseMethods" => rubocop_callbacks(InverseMethodsRule, [on_send, on_block]),
}

impl InverseMethodsRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        return_unless!(node.name().as_slice() == b"!");
        return_if!(self.parent().and_then(Node::as_call_node).is_some_and(|parent| parent.name().as_slice() == b"!"));

        let inverse_blocks = bidirectional(self.config_symbol_map("InverseBlocks"));
        return_if!(inside_inverse_block(node.as_node(), self.ancestors(), &inverse_blocks));
        let inverse_methods = bidirectional(self.config_symbol_map("InverseMethods"));
        let Some((method_call, _wrapped)) = inverse_candidate(node) else { return };
        let method = String::from_utf8_lossy(method_call.name().as_slice()).to_string();
        let Some(inverse) = inverse_methods.get(&method).cloned() else { return };
        return_if!(safe_navigation_incompatible(&method_call, &method));
        return_if!(possible_class_hierarchy_check(&method_call, &method, self.source_file()));

        let Some(selector) = method_call.message_loc() else { return };
        let inner_range = method_call.location().start_offset()..method_call.location().end_offset();
        let inner = self.source_file().slice(inner_range.clone()).unwrap_or_default();
        let selector_range = selector.start_offset()..selector.end_offset();
        let relative = selector_range.start - inner_range.start..selector_range.end - inner_range.start;
        let mut replacement = inner.to_string();
        replacement.replace_range(relative, &inverse);
        let message = format!("Use `{inverse}` instead of inverting `{method}`.");
        let offense = node.location();
        add_offense!(self, offense, message: message, |corrector| {
            corrector.replace(node.location(), replacement);
        });
    }

    fn on_block(&mut self, node: &BlockNode<'_>) {
        let Some(call) = self.parent().and_then(Node::as_call_node) else { return };
        return_unless!(call.receiver().is_some());
        let enclosing_negations = self
            .ancestors()
            .iter()
            .rev()
            .filter_map(Node::as_call_node)
            .filter(|ancestor| ancestor.name().as_slice() == b"!" && ancestor.location().start_offset() <= call.location().start_offset() && call.location().end_offset() <= ancestor.location().end_offset())
            .count();
        return_if!(enclosing_negations >= 2);
        let inverse_blocks = bidirectional(self.config_symbol_map("InverseBlocks"));
        let method = String::from_utf8_lossy(call.name().as_slice()).to_string();
        let Some(inverse) = inverse_blocks.get(&method).cloned() else { return };
        return_if!(self.source_file().node(&node.as_node()).split(|character: char| !character.is_alphanumeric() && character != '_').any(|word| word == "next"));
        let Some(last) = last_block_expression(node) else { return };
        let Some(body_edit) = inverse_block_edit(&last) else { return };
        let Some(selector) = call.message_loc() else { return };
        let selector_range = selector.start_offset()..selector.end_offset();
        let message = format!("Use `{inverse}` instead of inverting `{method}`.");
        add_offense!(self, call.location(), message: message, |corrector| {
            corrector.replace(selector_range, inverse);
            corrector.replace(body_edit.range, body_edit.replacement);
        });
    }
}

fn bidirectional(config: Option<&HashMap<String, String>>) -> HashMap<String, String> {
    let mut pairs = config.cloned().unwrap_or_default();
    for (method, inverse) in pairs.clone() {
        pairs.entry(inverse).or_insert(method);
    }
    pairs
}

fn inverse_candidate<'pr>(bang: &CallNode<'pr>) -> Option<(CallNode<'pr>, bool)> {
    let receiver = bang.receiver()?;
    if let Some(call) = receiver.as_call_node() {
        return Some((call, false));
    }
    let parentheses = receiver.as_parentheses_node()?;
    let inner = parentheses.body().and_then(single_expression)?;
    Some((inner.as_call_node()?, true))
}

fn safe_navigation_incompatible(call: &CallNode<'_>, method: &str) -> bool {
    let safe_navigation = call.call_operator_loc().is_some_and(|operator| operator.as_slice() == b"&.");
    safe_navigation && matches!(method, "any?" | "none?" | "<" | ">" | "<=" | ">=")
}

fn possible_class_hierarchy_check(call: &CallNode<'_>, method: &str, file: SourceFile<'_>) -> bool {
    if !matches!(method, "<" | ">" | "<=" | ">=") {
        return false;
    }
    call.receiver().is_some_and(|receiver| camel_case_constant(&receiver, file))
        || call.first_argument().is_some_and(|argument| camel_case_constant(&argument, file))
}

fn camel_case_constant(node: &Node<'_>, file: SourceFile<'_>) -> bool {
    (node.as_constant_read_node().is_some() || node.as_constant_path_node().is_some())
        && file.node(node).as_bytes().windows(2).any(|pair| pair[0].is_ascii_uppercase() && pair[1].is_ascii_lowercase())
}

fn inside_inverse_block(node: Node<'_>, ancestors: &[Node<'_>], inverse_blocks: &HashMap<String, String>) -> bool {
    let Some((index, block)) = ancestors.iter().enumerate().rev().find_map(|(index, ancestor)| ancestor.as_block_node().map(|block| (index, block))) else {
        return false;
    };
    let parent_call = ancestors.get(index.wrapping_sub(1)).and_then(Node::as_call_node)
        .or_else(|| ancestors.get(index + 1).and_then(Node::as_call_node));
    let Some(parent_call) = parent_call else { return false };
    if !inverse_blocks.contains_key(&String::from_utf8_lossy(parent_call.name().as_slice()).to_string()) {
        return false;
    }
    last_block_expression(&block).is_some_and(|last| {
        inverse_block_edit(&last).is_some()
            && last.location().start_offset() <= node.location().start_offset()
            && node.location().end_offset() <= last.location().end_offset()
    })
}

fn last_block_expression<'pr>(block: &BlockNode<'pr>) -> Option<Node<'pr>> {
    last_expression(block.body()?)
}

fn last_expression<'pr>(node: Node<'pr>) -> Option<Node<'pr>> {
    if let Some(statements) = node.as_statements_node() {
        return statements.body().iter().last();
    }
    if let Some(begin) = node.as_begin_node() {
        return begin.statements().and_then(|statements| statements.body().iter().last());
    }
    Some(node)
}

fn inverse_block_edit(node: &Node<'_>) -> Option<SourceEdit> {
    if let Some(parentheses) = node.as_parentheses_node() {
        return parentheses.body().and_then(single_expression).and_then(|inner| inverse_block_edit(&inner));
    }
    let call = node.as_call_node()?;
    let method = call.name().as_slice();
    let selector = call.message_loc()?;
    if matches!(method, b"!=" | b"!~") {
        return Some(SourceEdit::replace(
            selector.start_offset()..selector.end_offset(),
            if method == b"!=" { "==" } else { "=~" },
        ));
    }
    if method != b"!" {
        return None;
    }
    let receiver = call.receiver()?;
    let selector_range = selector.start_offset()..selector.end_offset();
    if selector_range.start <= receiver.location().start_offset() {
        Some(SourceEdit::remove(selector_range))
    } else {
        Some(SourceEdit::remove(receiver.location().end_offset()..call.location().end_offset()))
    }
}
