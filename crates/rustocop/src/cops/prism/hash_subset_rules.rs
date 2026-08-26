use ruby_prism::{BlockNode, CallNode, Node};

use super::*;

define_cops! {
    HashSlice => "Style/HashSlice" => rubocop_callbacks(HashSliceRule, [on_send restrict [b"reject", b"select", b"filter"]]),
    HashExcept => "Style/HashExcept" => rubocop_callbacks(HashExceptRule, [on_send restrict [b"reject", b"select", b"filter"]]),
}

impl HashSliceRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        return_if!(!self.target_ruby_version().at_least(2, 5));
        check_subset(self, node, true);
    }
}

impl HashExceptRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        return_if!(!self.target_ruby_version().at_least(3, 0));
        check_subset(self, node, false);
    }
}

fn check_subset(context: &mut CopContext<'_, '_>, node: &CallNode<'_>, slice: bool) {
    let Some(block) = node.block().and_then(|block| block.as_block_node()) else { return };
    let Some((key, value)) = subset_parameters(&block, context.source_file()) else { return };
    let Some(body) = block.body().and_then(single_expression) else { return };
    let active_support = context
        .related_config_value("AllCops", "ActiveSupportExtensionsEnabled")
        == Some("true");
    let Some((keys, targeted)) = subset_condition(
        &body,
        &key,
        &value,
        active_support,
        slice,
        context.source_file(),
    ) else { return };
    let keeps = matches!(node.name().as_slice(), b"select" | b"filter");
    return_unless!((keeps == targeted) == slice);
    let preferred = if slice { "slice" } else { "except" };
    let replacement = format!("{preferred}({keys})");
    let Some(selector) = node.message_loc() else { return };
    let edit = selector.start_offset()..block.closing_loc().end_offset();
    add_offense!(context, edit.clone(), message: format!("Use `{replacement}` instead."), |corrector| {
        corrector.replace(edit, replacement);
    });
}

fn subset_parameters(block: &BlockNode<'_>, file: SourceFile<'_>) -> Option<(String, String)> {
    let parameters = block.parameters()?.as_block_parameters_node()?;
    let parameters = parameters.parameters()?;
    let required = parameters.requireds().iter().collect::<Vec<_>>();
    let [key, value] = required.as_slice() else { return None };
    Some((file.node(key).to_string(), file.node(value).to_string()))
}

fn subset_condition(
    body: &Node<'_>,
    key: &str,
    value: &str,
    active_support: bool,
    slice: bool,
    file: SourceFile<'_>,
) -> Option<(String, bool)> {
    use crate::rubocop::cop::mixin::hash_subset::{HashSubset, HashSubsetPreference};

    let (call, negated) = if let Some(outer) = body.as_call_node() {
        if outer.name().as_slice() == b"!" {
            (outer.receiver()?.as_call_node()?, true)
        } else {
            (outer, false)
        }
    } else {
        return None;
    };
    let method = call.name().as_slice();
    let method_name = std::str::from_utf8(method).ok()?;
    let subset = HashSubset {
        active_support_extensions_enabled: active_support,
        preference: if slice {
            HashSubsetPreference::Slice
        } else {
            HashSubsetPreference::Except
        },
    };
    if !subset.supported_subset_method(method_name) {
        return None;
    }
    let (keys, mut targeted) = match method {
        b"==" | b"!=" => {
            let receiver = call.receiver()?;
            let argument = only_argument(&call)?;
            let other = if file.node(&receiver) == key {
                argument
            } else if file.node(&argument) == key {
                receiver
            } else {
                return None;
            };
            if other.as_symbol_node().is_none() && other.as_string_node().is_none() {
                return None;
            }
            (file.node(&other).to_string(), method == b"==")
        }
        b"eql?" => {
            let receiver = call.receiver()?;
            return_unless!(file.node(&receiver) == key, None);
            let argument = only_argument(&call)?;
            (file.node(&argument).to_string(), true)
        }
        b"include?" | b"exclude?" => {
            return_if!(method == b"exclude?" && !active_support, None);
            let receiver = call.receiver()?;
            let argument = only_argument(&call)?;
            return_unless!(file.node(&argument) == key, None);
            return_if!(range_expression(&receiver), None);
            (subset_key_source(&receiver, file), method == b"include?")
        }
        b"in?" if active_support => {
            let receiver = call.receiver()?;
            return_unless!(file.node(&receiver) == key, None);
            let argument = only_argument(&call)?;
            return_if!(range_expression(&argument), None);
            (subset_key_source(&argument, file), true)
        }
        _ => return None,
    };
    return_if!(contains_word(&keys, value), None);
    if negated {
        targeted = !targeted;
    }
    Some((keys, targeted))
}

fn range_expression(node: &Node<'_>) -> bool {
    if node.as_range_node().is_some() {
        return true;
    }
    node.as_parentheses_node()
        .and_then(|parentheses| parentheses.body())
        .and_then(single_expression)
        .is_some_and(|body| range_expression(&body))
}

fn subset_key_source(node: &Node<'_>, file: SourceFile<'_>) -> String {
    let Some(array) = node.as_array_node() else {
        return format!("*{}", file.node(node));
    };
    let array_source = file.node(node).trim_start();
    array
        .elements()
        .iter()
        .map(|element| decorate_element(&element, array_source, file))
        .collect::<Vec<_>>()
        .join(", ")
}

fn decorate_element(node: &Node<'_>, array_source: &str, file: SourceFile<'_>) -> String {
    let source = file.node(node);
    if array_source.starts_with("%i") {
        format!(":{source}")
    } else if array_source.starts_with("%I") {
        if source.contains("#{") {
            format!(":\"{source}\"")
        } else {
            format!(":{source}")
        }
    } else if array_source.starts_with("%w") {
        format!("'{source}'")
    } else if array_source.starts_with("%W") {
        if source.contains("#{") {
            format!("\"{source}\"")
        } else {
            format!("'{source}'")
        }
    } else {
        source.to_string()
    }
}

fn contains_word(source: &str, name: &str) -> bool {
    source
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .any(|word| word == name)
}
