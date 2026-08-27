use ruby_prism::Node;

use crate::rubocop::cop::mixin::advanced::{
    frozen_string_literal as frozen_string_literal_magic_comment,
    FrozenStringLiteral as FrozenStringLiteralSetting,
};

use super::*;

define_cops! {
    MutableConstant => "Style/MutableConstant" => compatibility_prism_callbacks(MutableConstantRule, [on_casgn]),
}

impl MutableConstantRule<'_, '_, '_> {
    fn on_casgn(&mut self, node: &Node<'_>) {
        let Some(value) = constant_assignment_value(node) else {
            return;
        };
        let strict = self.policy().enforced_style("literals") == "strict";
        if strict {
            return_if!(immutable_literal(
                &value,
                self.target_ruby_version(),
                self.source()
            ));
            return_if!(operation_produces_immutable_object(&value));
        } else {
            return_unless!(mutable_literal(&value, self.target_ruby_version()));
        }
        return_if!(frozen_string_literal(
            &value,
            self.source(),
            self.target_ruby_version()
        ));
        return_if!(shareable_constant_value(
            &value,
            self.source(),
            self.target_ruby_version()
        ));

        let offense = value.location().start_offset()..value.location().end_offset();
        let replacement = frozen_replacement(&value, self.source_file(), strict);
        add_offense!(self, offense.clone(), message: "Freeze mutable objects assigned to constants.", |corrector| {
            corrector.replace(offense, replacement);
        });
    }
}

fn constant_assignment_value<'pr>(node: &Node<'pr>) -> Option<Node<'pr>> {
    if let Some(write) = node.as_constant_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_constant_path_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_constant_or_write_node() {
        Some(write.value())
    } else {
        node.as_constant_path_or_write_node()
            .map(|write| write.value())
    }
}

fn mutable_literal(node: &Node<'_>, ruby_version: RubyVersion) -> bool {
    if ruby_version.at_least(3, 0) && (regexp_literal(node) || range_expression(node)) {
        return false;
    }
    node.as_array_node().is_some()
        || node.as_hash_node().is_some()
        || node.as_string_node().is_some()
        || node.as_interpolated_string_node().is_some()
        || node.as_x_string_node().is_some()
        || node.as_interpolated_x_string_node().is_some()
        || regexp_literal(node)
        || range_expression(node)
}

fn immutable_literal(node: &Node<'_>, ruby_version: RubyVersion, source: &str) -> bool {
    node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_nil_node().is_some()
        || node.as_source_file_node().is_some()
        || ruby_version.at_least(3, 0)
            && (regexp_literal(node)
                || range_expression(node) && !parenthesized_expression(node, source))
}

fn parenthesized_expression(node: &Node<'_>, source: &str) -> bool {
    if node.as_parentheses_node().is_some() {
        return true;
    }
    let location = node.location();
    source[..location.start_offset()].trim_end().ends_with('(')
        && source[location.end_offset()..].trim_start().starts_with(')')
}

fn regexp_literal(node: &Node<'_>) -> bool {
    node.as_regular_expression_node().is_some()
        || node.as_interpolated_regular_expression_node().is_some()
}

fn operation_produces_immutable_object(node: &Node<'_>) -> bool {
    if node.as_constant_read_node().is_some() || node.as_constant_path_node().is_some() {
        return true;
    }
    if let Some(or_node) = node.as_or_node() {
        return env_index(&or_node.left());
    }
    let Some(call) = node.as_call_node() else {
        return false;
    };
    if call.name().as_slice() == b"freeze" || struct_constructor(&call) || env_index(node) {
        return true;
    }
    if matches!(call.name().as_slice(), b"count" | b"length" | b"size") {
        return true;
    }
    if matches!(
        call.name().as_slice(),
        b"==" | b"===" | b"!=" | b"<=" | b">=" | b"<" | b">"
    ) {
        return true;
    }
    if matches!(
        call.name().as_slice(),
        b"+" | b"-" | b"*" | b"**" | b"/" | b"%" | b"<<"
    ) {
        let numeric_receiver = call
            .receiver()
            .is_some_and(|receiver| numeric_literal(&receiver));
        let numeric_argument =
            only_argument(&call).is_some_and(|argument| numeric_literal(&argument));
        return numeric_receiver || call.name().as_slice() != b"<<" && numeric_argument;
    }
    false
}

fn numeric_literal(node: &Node<'_>) -> bool {
    node.as_integer_node().is_some() || node.as_float_node().is_some()
}

fn struct_constructor(call: &ruby_prism::CallNode<'_>) -> bool {
    call.name().as_slice() == b"new"
        && call.receiver().is_some_and(|receiver| {
            receiver
                .as_constant_read_node()
                .is_some_and(|constant| constant.name().as_slice() == b"Struct")
                || receiver.as_constant_path_node().is_some_and(|constant| {
                    constant
                        .name()
                        .is_some_and(|name| name.as_slice() == b"Struct")
                })
        })
}

fn env_index(node: &Node<'_>) -> bool {
    let Some(call) = node.as_call_node() else {
        return false;
    };
    call.name().as_slice() == b"[]"
        && call.receiver().is_some_and(|receiver| {
            receiver
                .as_constant_read_node()
                .is_some_and(|constant| constant.name().as_slice() == b"ENV")
                || receiver.as_constant_path_node().is_some_and(|constant| {
                    constant
                        .name()
                        .is_some_and(|name| name.as_slice() == b"ENV")
                })
        })
}

fn frozen_string_literal(node: &Node<'_>, source: &str, ruby_version: RubyVersion) -> bool {
    (node.as_string_node().is_some()
        || node
            .as_interpolated_string_node()
            .is_some_and(|string| string.is_frozen() || !ruby_version.at_least(3, 0)))
        && frozen_string_literal_magic_comment(source) == FrozenStringLiteralSetting::Enabled
}

fn shareable_constant_value(node: &Node<'_>, source: &str, ruby_version: RubyVersion) -> bool {
    if !ruby_version.at_least(3, 0) {
        return false;
    }
    source[..node.location().start_offset()]
        .lines()
        .rev()
        .find_map(|line| {
            line.trim()
                .strip_prefix("# shareable_constant_value:")
                .map(str::trim)
                .filter(|value| {
                    matches!(
                        *value,
                        "literal" | "experimental_everything" | "experimental_copy" | "none"
                    )
                })
        })
        .is_some_and(|value| value != "none")
}

fn range_expression(node: &Node<'_>) -> bool {
    if node.as_range_node().is_some() {
        return true;
    }
    node.as_parentheses_node()
        .and_then(|parentheses| parentheses.body())
        .and_then(single_expression)
        .is_some_and(|expression| expression.as_range_node().is_some())
}

fn frozen_replacement(node: &Node<'_>, file: SourceFile<'_>, strict: bool) -> String {
    let source = file.node(node);
    if let Some(array) = node.as_array_node() {
        let elements = array.elements().iter().collect::<Vec<_>>();
        if elements.len() == 1 {
            if let Some(splat) = elements[0].as_splat_node() {
                if let Some(expression) = splat.expression() {
                    if range_expression(&expression) {
                        let range = file.node(&expression).trim_matches(['(', ')']);
                        return format!("({range}).to_a.freeze");
                    }
                }
            }
        }
        if !source.starts_with('[') && !source.starts_with('%') && !source.starts_with('*') {
            return format!("[{source}].freeze");
        }
    }
    if range_expression(node) && !source.starts_with('(') {
        return format!("({source}).freeze");
    }
    if strict
        && node.as_call_node().is_some_and(|call| {
            call.call_operator_loc().is_none()
                && matches!(
                    call.name().as_slice(),
                    b"+" | b"-" | b"*" | b"/" | b"%" | b"**"
                )
        })
    {
        return format!("({source}).freeze");
    }
    format!("{source}.freeze")
}
