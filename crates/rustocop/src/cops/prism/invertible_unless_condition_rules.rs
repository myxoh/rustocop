use std::collections::HashMap;

use ruby_prism::{Node, UnlessNode};

use super::*;

define_cops! {
    InvertibleUnlessCondition => "Style/InvertibleUnlessCondition" => rubocop_callbacks(InvertibleUnlessConditionRule, [on_unless]),
}

impl InvertibleUnlessConditionRule<'_, '_, '_> {
    fn on_unless(&mut self, node: &UnlessNode<'_>) {
        let condition = node.predicate();
        let inverse_methods = self.config_map("InverseMethods").cloned().unwrap_or_default();
        let mut edits = Vec::new();
        return_unless!(invert_condition(&condition, &inverse_methods, self.source_file(), &mut edits));

        let condition_range = condition.location().start_offset()..condition.location().end_offset();
        let Some(preferred) = self.source_file().rewrite(condition_range.clone(), edits.clone()) else {
            return;
        };
        let current = self.source_file().slice(condition_range).unwrap_or_default();
        let message = format!("Prefer `if {preferred}` over `unless {current}`.");
        let keyword = node.keyword_loc();
        add_offense!(self, node.location(), message: message, |corrector| {
            corrector.replace(keyword, "if");
            for edit in edits {
                corrector.replace(edit.range, edit.replacement);
            }
        });
    }
}

fn invert_condition(
    node: &Node<'_>,
    inverse_methods: &HashMap<String, String>,
    file: SourceFile<'_>,
    edits: &mut Vec<SourceEdit>,
) -> bool {
    if let Some(parentheses) = node.as_parentheses_node() {
        return parentheses
            .body()
            .and_then(single_expression)
            .is_some_and(|inner| invert_condition(&inner, inverse_methods, file, edits));
    }
    if let Some(and) = node.as_and_node() {
        let mut nested = Vec::new();
        if !invert_condition(&and.left(), inverse_methods, file, &mut nested)
            || !invert_condition(&and.right(), inverse_methods, file, &mut nested)
        {
            return false;
        }
        edits.extend(nested);
        edits.push(SourceEdit::replace(
            and.operator_loc().start_offset()..and.operator_loc().end_offset(),
            if and.operator_loc().as_slice() == b"and" { "or" } else { "||" },
        ));
        return true;
    }
    if let Some(or) = node.as_or_node() {
        let mut nested = Vec::new();
        if !invert_condition(&or.left(), inverse_methods, file, &mut nested)
            || !invert_condition(&or.right(), inverse_methods, file, &mut nested)
        {
            return false;
        }
        edits.extend(nested);
        edits.push(SourceEdit::replace(
            or.operator_loc().start_offset()..or.operator_loc().end_offset(),
            if or.operator_loc().as_slice() == b"or" { "and" } else { "&&" },
        ));
        return true;
    }
    let Some(call) = node.as_call_node() else {
        return false;
    };
    let method = String::from_utf8_lossy(call.name().as_slice()).to_string();
    if method == "!" {
        let Some(selector) = call.message_loc() else { return false };
        edits.push(SourceEdit::remove(selector.start_offset()..selector.end_offset()));
        return true;
    }
    if method == "<" && call.first_argument().is_some_and(|argument| {
        let source = file.node(&argument);
        let short_name = source.rsplit("::").next().unwrap_or(source);
        (argument.as_constant_read_node().is_some() || argument.as_constant_path_node().is_some())
            && short_name.bytes().any(|byte| byte.is_ascii_lowercase())
    }) {
        return false;
    }
    let Some(inverse) = inverse_methods.get(&method) else {
        return false;
    };
    let Some(selector) = call.message_loc() else { return false };
    edits.push(SourceEdit::replace(
        selector.start_offset()..selector.end_offset(),
        inverse.clone(),
    ));
    true
}
