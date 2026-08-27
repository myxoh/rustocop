use std::collections::HashSet;

use ruby_prism::{
    DefNode, HashNode, InterpolatedRegularExpressionNode, Node, RescueNode, ReturnNode, StringNode,
};

use super::source_syntax::top_level_elements;
use super::*;

define_cops! {
    RedundantConstantBase => "Style/RedundantConstantBase" => any_node(redundant_constant_base),
    ArrayLiteralInRegexp => "Lint/ArrayLiteralInRegexp" => node(as_interpolated_regular_expression_node, array_literal_in_regexp),
    LiteralAssignmentInCondition => "Lint/LiteralAssignmentInCondition" => any_node(literal_assignment_in_condition),
    NoReturnInBeginEndBlocks => "Lint/NoReturnInBeginEndBlocks" => node(as_return_node, no_return_in_begin_end_blocks),
    RescueType => "Lint/RescueType" => compatibility_callbacks(RescueTypeRule, [on_resbody]),
    FirstMethodParameterLineBreak => "Layout/FirstMethodParameterLineBreak" => node(as_def_node, first_method_parameter_line_break),
}

define_compatibility_rule!(RescueTypeRule);


fn array_literal_in_regexp(
    node: &InterpolatedRegularExpressionNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    for part in node.parts().iter() {
        let Some(embedded) = part.as_embedded_statements_node() else {
            continue;
        };
        let Some(array) = embedded
            .statements()
            .and_then(|statements| statements.body().last())
            .and_then(|expression| expression.as_array_node())
        else {
            continue;
        };

        let values = array
            .elements()
            .iter()
            .map(|element| regexp_literal_value(&element, context))
            .collect::<Option<Vec<_>>>();
        let (message, replacement) = if let Some(values) = values {
            let escaped = values
                .iter()
                .map(|value| regexp_escape(value))
                .collect::<Vec<_>>();
            if values.iter().all(|value| value.chars().count() == 1) {
                (
                    "Use a character class instead of interpolating an array in a regexp.",
                    Some(format!("[{}]", escaped.join(""))),
                )
            } else {
                (
                    "Use alternation instead of interpolating an array in a regexp.",
                    Some(format!("(?:{})", escaped.join("|"))),
                )
            }
        } else {
            (
                "Use alternation or a character class instead of interpolating an array in a regexp.",
                None,
            )
        };
        let location = embedded.location();
        if let Some(replacement) = replacement {
            context.replace(message, &location, &location, replacement);
        } else {
            context.report(message, location);
        }
    }
}

fn regexp_literal_value(node: &Node<'_>, context: &CopContext<'_, '_>) -> Option<String> {
    if let Some(string) = node.as_string_node() {
        return Some(String::from_utf8_lossy(string.unescaped()).into_owned());
    }
    if let Some(symbol) = node.as_symbol_node() {
        return Some(String::from_utf8_lossy(symbol.unescaped()).into_owned());
    }
    if node.as_integer_node().is_some() || node.as_float_node().is_some() {
        return Some(context.source_file().node(node).to_string());
    }
    if node.as_true_node().is_some() {
        return Some("true".to_string());
    }
    if node.as_false_node().is_some() {
        return Some("false".to_string());
    }
    node.as_nil_node().map(|_| "nil".to_string())
}

fn regexp_escape(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{000C}' => escaped.push_str("\\f"),
            ' ' => escaped.push_str("\\ "),
            '\\' | '.' | '+' | '*' | '?' | '[' | ']' | '^' | '$' | '(' | ')' | '{' | '}'
            | '|' | '-' | '#' => {
                escaped.push('\\');
                escaped.push(character);
            }
            _ => escaped.push(character),
        }
    }
    escaped
}

fn duplicate_rescue_exception(node: &RescueNode<'_>, context: &mut CopContext<'_, '_>) {
    let mut seen = HashSet::new();
    for ancestor in context.ancestors() {
        let Some(rescue) = ancestor.as_rescue_node() else {
            continue;
        };
        let current = node.location();
        if !rescue.subsequent().is_some_and(|subsequent| {
            let subsequent = subsequent.location();
            subsequent.start_offset() <= current.start_offset()
                && current.end_offset() <= subsequent.end_offset()
        }) {
            continue;
        }
        for exception in rescue.exceptions().iter() {
            seen.insert(context.source_file().node(&exception).to_string());
        }
    }
    for exception in node.exceptions().iter() {
        let source = context.source_file().node(&exception).to_string();
        if !seen.insert(source) {
            context.report("Duplicate `rescue` exception detected.", exception.location());
        }
    }
}

fn literal_assignment_in_condition(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let Some((operator, value)) = simple_assignment(node) else {
        return;
    };
    if !condition_contains_assignment(node, context) || !condition_literal(&value) {
        return;
    }
    let literal = context.source_file().node(&value);
    context.report(
        format!(
            "Don't use literal assignment `= {literal}` in conditional, should be `==` or non-literal operand."
        ),
        operator.start_offset()..value.location().end_offset(),
    );
}

fn simple_assignment<'pr>(node: &Node<'pr>) -> Option<(ruby_prism::Location<'pr>, Node<'pr>)> {
    macro_rules! assignment {
        ($($cast:ident),+ $(,)?) => {$ (
            if let Some(write) = node.$cast() {
                return Some((write.operator_loc(), write.value()));
            }
        )+ };
    }
    assignment!(
        as_local_variable_write_node,
        as_instance_variable_write_node,
        as_class_variable_write_node,
        as_global_variable_write_node,
        as_constant_write_node,
        as_constant_path_write_node,
    );
    None
}

fn condition_contains_assignment(node: &Node<'_>, context: &CopContext<'_, '_>) -> bool {
    let location = node.location();
    for ancestor in context.ancestors().iter().rev() {
        let predicate = if let Some(condition) = ancestor.as_if_node() {
            Some(condition.predicate())
        } else if let Some(condition) = ancestor.as_unless_node() {
            Some(condition.predicate())
        } else if let Some(condition) = ancestor.as_while_node() {
            Some(condition.predicate())
        } else { ancestor.as_until_node().map(|condition| condition.predicate()) };
        if let Some(predicate) = predicate {
            let predicate = predicate.location();
            return predicate.start_offset() <= location.start_offset()
                && location.end_offset() <= predicate.end_offset();
        }
    }
    false
}

fn condition_literal(node: &Node<'_>) -> bool {
    if node.as_interpolated_string_node().is_some()
        || node.as_interpolated_x_string_node().is_some()
        || node.as_x_string_node().is_some()
    {
        return false;
    }
    if node.as_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_rational_node().is_some()
        || node.as_imaginary_node().is_some()
        || node.as_regular_expression_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_nil_node().is_some()
    {
        return true;
    }
    if let Some(array) = node.as_array_node() {
        return array.elements().iter().all(|element| {
            element.as_splat_node().is_none() && condition_literal(&element)
        });
    }
    if let Some(hash) = node.as_hash_node() {
        return hash.elements().iter().all(|element| {
            element.as_assoc_node().is_some_and(|pair| {
                condition_literal(&pair.key()) && condition_literal(&pair.value())
            })
        });
    }
    false
}

fn no_return_in_begin_end_blocks(node: &ReturnNode<'_>, context: &mut CopContext<'_, '_>) {
    let mut inside_explicit_begin = false;
    for ancestor in context.ancestors().iter().rev() {
        if ancestor
            .as_begin_node()
            .is_some_and(|begin| begin.begin_keyword_loc().is_some())
        {
            inside_explicit_begin = true;
            continue;
        }
        if inside_explicit_begin && assignment_context(ancestor) {
            context.report(
                "Do not `return` in `begin..end` blocks in assignment contexts.",
                node.location(),
            );
            return;
        }
    }
}

fn assignment_context(node: &Node<'_>) -> bool {
    macro_rules! any_assignment {
        ($($cast:ident),+ $(,)?) => {
            $(if node.$cast().is_some() { return true; })+
        };
    }
    any_assignment!(
        as_local_variable_write_node,
        as_instance_variable_write_node,
        as_class_variable_write_node,
        as_global_variable_write_node,
        as_constant_write_node,
        as_constant_path_write_node,
        as_local_variable_or_write_node,
        as_instance_variable_or_write_node,
        as_class_variable_or_write_node,
        as_global_variable_or_write_node,
        as_constant_or_write_node,
        as_constant_path_or_write_node,
        as_local_variable_operator_write_node,
        as_instance_variable_operator_write_node,
        as_class_variable_operator_write_node,
        as_global_variable_operator_write_node,
        as_constant_operator_write_node,
        as_constant_path_operator_write_node,
    );
    false
}

const RESCUE_TYPE_MSG: &str = "Rescuing from `{invalid_exceptions}` will raise a `TypeError` instead of catching the actual exception.";

impl RescueTypeRule<'_, '_, '_, '_> {
    fn on_resbody(&mut self, node: crate::rubocop::ast::node::core::NodeRef<'_>) {
        let exceptions = node.exceptions();
        let invalid_exceptions = self.invalid_exceptions(&exceptions);
        return_if!(invalid_exceptions.is_empty());

        let invalid_sources = invalid_exceptions
            .iter()
            .filter_map(|exception| exception.source())
            .collect::<Vec<_>>()
            .join(", ");
        let (Some((keyword, _)), Some(rescued)) = (node.loc("keyword"), node.node_child(0)) else {
            return;
        };
        let Some(rescued_range) = self.source_range(rescued) else { return; };
        let offense = self.owned_range(crate::rubocop::ast::source::SourceRange::new(
            self.source_buffer(), keyword.start, rescued_range.end_pos()
        ));
        let edit = self.owned_range(crate::rubocop::ast::source::SourceRange::new(
            self.source_buffer(), keyword.end, rescued_range.end_pos()
        ));
        let replacement = self.correction(&exceptions);
        let message = RESCUE_TYPE_MSG.replace("{invalid_exceptions}", &invalid_sources);
        add_offense!(self, offense, message: message, |corrector| {
            corrector.replace(edit, replacement);
        });
    }

    fn correction(&self, exceptions: &[crate::rubocop::ast::node::core::NodeRef<'_>]) -> String {
        let correction = self
            .valid_exceptions(exceptions)
            .iter()
            .filter_map(|exception| exception.source())
            .collect::<Vec<_>>()
            .join(", ");
        if correction.is_empty() {
            correction
        } else {
            format!(" {correction}")
        }
    }

    fn valid_exceptions<'node>(&self, exceptions: &'node [crate::rubocop::ast::node::core::NodeRef<'node>]) -> Vec<crate::rubocop::ast::node::core::NodeRef<'node>> {
        exceptions
            .iter()
            .copied()
            .filter(|exception| !invalid_rescue_type_compatibility(*exception))
            .collect()
    }

    fn invalid_exceptions<'node>(&self, exceptions: &'node [crate::rubocop::ast::node::core::NodeRef<'node>]) -> Vec<crate::rubocop::ast::node::core::NodeRef<'node>> {
        exceptions
            .iter()
            .copied()
            .filter(|exception| invalid_rescue_type_compatibility(*exception))
            .collect()
    }
}

fn invalid_rescue_type_compatibility(node: crate::rubocop::ast::node::core::NodeRef<'_>) -> bool {
    matches!(node.kind(), "array" | "dstr" | "float" | "hash" | "nil" | "int" | "str" | "sym")
}

fn first_method_parameter_line_break(node: &DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let (Some(opening), Some(closing)) = (node.lparen_loc(), node.rparen_loc()) else {
        return;
    };
    let source = context.source();
    if !source[opening.start_offset()..closing.end_offset()].contains('\n')
        || source[opening.end_offset()..].starts_with('\n')
    {
        return;
    }
    let first_line = source[opening.end_offset()..closing.start_offset()]
        .split_once('\n')
        .map_or(
            &source[opening.end_offset()..closing.start_offset()],
            |(line, _)| line,
        );
    if first_line.trim_start().starts_with('#') {
        return;
    }
    let elements = top_level_elements(source, opening.end_offset(), closing.start_offset());
    let Some(first) = elements.first() else {
        return;
    };
    if context.config_bool("AllowMultilineFinalElement", false)
        && elements
            .last()
            .is_some_and(|last| source[last.clone()].contains('\n'))
    {
        return;
    }
    let start = restored_leading_code_offset(source, first.start, first.end);
    if source[opening.end_offset()..start].contains('\n') {
        return;
    }
    let end = first.end - (source[first.clone()].len() - source[first.clone()].trim_end().len());
    context.insert(
        "Add a line break before the first parameter of a multi-line method parameter list.",
        start..end.max(start),
        start,
        "\n",
    );
}

fn restored_leading_code_offset(source: &str, mut start: usize, end: usize) -> usize {
    while start < end {
        start += source[start..end].len() - source[start..end].trim_start().len();
        if source.as_bytes().get(start) != Some(&b'#') {
            break;
        }
        start = source[start..end]
            .find('\n')
            .map_or(end, |newline| start + newline + 1);
    }
    start
}

fn duplicate_hash_key(node: &HashNode<'_>, context: &mut CopContext<'_, '_>) {
    let mut seen = HashSet::new();
    for element in node.elements().iter() {
        let Some(pair) = element.as_assoc_node() else {
            continue;
        };
        let key = pair.key();
        if !duplicate_hash_key_candidate(&key) {
            continue;
        }
        let fingerprint = duplicate_hash_key_fingerprint(&key, context);
        if !seen.insert(fingerprint) {
            let label_value = key.as_symbol_node().and_then(|symbol| {
                symbol.opening_loc().is_none().then(|| symbol.value_loc()).flatten()
            });
            context.report(
                "Duplicated key in hash literal.",
                label_value.unwrap_or_else(|| key.location()),
            );
        }
    }
}

fn duplicate_hash_key_fingerprint(node: &Node<'_>, context: &CopContext<'_, '_>) -> String {
    if let Some(symbol) = node.as_symbol_node() {
        return format!("symbol:{:?}", symbol.unescaped());
    }
    if let Some(string) = node.as_string_node() {
        return format!("string:{:?}", string.unescaped());
    }
    if node.as_true_node().is_some() {
        return "true".to_string();
    }
    if node.as_false_node().is_some() {
        return "false".to_string();
    }
    if node.as_nil_node().is_some() {
        return "nil".to_string();
    }
    context.source_file().node(node).to_string()
}

fn duplicate_hash_key_candidate(node: &Node<'_>) -> bool {
    const STATIC_LITERAL: u16 = 0x2;
    if node.as_constant_read_node().is_some() || node.as_constant_path_node().is_some() {
        return true;
    }
    if node.as_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_rational_node().is_some()
        || node.as_imaginary_node().is_some()
        || node.as_regular_expression_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_nil_node().is_some()
    {
        return true;
    }
    if let Some(call) = node.as_call_node() {
        let literal_operator = matches!(
            call.name().as_slice(),
            b"!" | b"<=>" | b"==" | b"!=" | b"<" | b">" | b"<=" | b">="
                | b"+" | b"-" | b"*" | b"/" | b"%" | b"**" | b"~"
        );
        return literal_operator
            && call.receiver().as_ref().is_some_and(duplicate_hash_key_candidate)
            && call.arguments().is_none_or(|arguments| {
                arguments.arguments().iter().all(|argument| {
                    duplicate_hash_key_candidate(&argument)
                })
            });
    }
    if let Some(array) = node.as_array_node() {
        return array.elements().iter().all(|element| {
            duplicate_hash_key_candidate(&element)
        });
    }
    if let Some(hash) = node.as_hash_node() {
        return hash.elements().iter().all(|element| {
            element.as_assoc_node().is_some_and(|pair| {
                duplicate_hash_key_candidate(&pair.key())
                    && duplicate_hash_key_candidate(&pair.value())
            })
        });
    }
    if let Some(parentheses) = node.as_parentheses_node() {
        return parentheses
            .body()
            .and_then(|body| body.as_statements_node())
            .is_some_and(|statements| {
                statements.body().len() == 1
                    && statements
                        .body()
                        .first()
                        .as_ref()
                        .is_some_and(duplicate_hash_key_candidate)
            });
    }
    if let Some(and_node) = node.as_and_node() {
        return duplicate_hash_key_candidate(&and_node.left())
            && duplicate_hash_key_candidate(&and_node.right());
    }
    if let Some(or_node) = node.as_or_node() {
        return duplicate_hash_key_candidate(&or_node.left())
            && duplicate_hash_key_candidate(&or_node.right());
    }
    if let Some(string) = node.as_interpolated_string_node() {
        return interpolated_parts_are_static(string.parts().iter());
    }
    if let Some(symbol) = node.as_interpolated_symbol_node() {
        return interpolated_parts_are_static(symbol.parts().iter());
    }
    if let Some(regexp) = node.as_interpolated_regular_expression_node() {
        return interpolated_parts_are_static(regexp.parts().iter());
    }
    macro_rules! has_static_literal_flag {
        ($($cast:ident),+ $(,)?) => {$ (
            if let Some(node) = node.$cast() {
                return node.flags() & STATIC_LITERAL != 0;
            }
        )+ };
    }
    has_static_literal_flag!(
        as_false_node,
        as_float_node,
        as_imaginary_node,
        as_integer_node,
        as_interpolated_regular_expression_node,
        as_interpolated_string_node,
        as_interpolated_symbol_node,
        as_nil_node,
        as_range_node,
        as_rational_node,
        as_regular_expression_node,
        as_string_node,
        as_symbol_node,
        as_true_node,
    );
    false
}

fn interpolated_parts_are_static<'pr>(mut parts: impl Iterator<Item = Node<'pr>>) -> bool {
    parts.all(|part| {
        if part.as_string_node().is_some() {
            return true;
        }
        let Some(embedded) = part.as_embedded_statements_node() else {
            return false;
        };
        embedded.statements().is_some_and(|statements| {
            statements.body().len() == 1
                && statements
                    .body()
                    .first()
                    .as_ref()
                    .is_some_and(duplicate_hash_key_candidate)
        })
    })
}

fn interpolation_check(node: &StringNode<'_>, context: &mut CopContext<'_, '_>) {
    const MESSAGE: &str = "Interpolation in single quoted string detected. Use double quoted strings if you need interpolation.";
    let location = node.location();
    let source = context.source_file().at(&location);
    if !source.starts_with('\'')
        || !source.ends_with('\'')
        || source.contains('\n')
        || !contains_unescaped_interpolation(source)
        || context.ancestors().iter().any(|ancestor| {
            ancestor.as_regular_expression_node().is_some()
                || ancestor.as_interpolated_regular_expression_node().is_some()
        })
    {
        return;
    }

    let replacement = if source.contains('"') {
        format!("%{{{}}}", &source[1..source.len() - 1])
    } else {
        format!("\"{}\"", &source[1..source.len() - 1])
    };
    let probe = format!("def __rustocop_interpolation_probe__; {replacement}; end");
    let valid = {
        let parsed = ruby_prism::parse(probe.as_bytes());
        parsed.errors().next().is_none() && contains_interpolated_string(&parsed.node())
    };
    if !valid {
        return;
    }
    context.replace(MESSAGE, &location, &location, replacement);
}

fn contains_unescaped_interpolation(source: &str) -> bool {
    source
        .match_indices("#{")
        .any(|(at, _)| source.as_bytes().get(at.wrapping_sub(1)) != Some(&b'\\'))
}

fn contains_interpolated_string(root: &Node<'_>) -> bool {
    struct Finder(bool);
    impl<'pr> Visit<'pr> for Finder {
        fn visit_interpolated_string_node(
            &mut self,
            _node: &ruby_prism::InterpolatedStringNode<'pr>,
        ) {
            self.0 = true;
        }
    }
    let mut finder = Finder(false);
    finder.visit(root);
    finder.0
}

fn top_level_return_with_argument(node: &ReturnNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.arguments().is_none()
        || context.ancestors().iter().any(|ancestor| {
            ancestor.as_block_node().is_some()
                || ancestor.as_def_node().is_some()
                || ancestor.as_lambda_node().is_some()
        })
    {
        return;
    }
    let location = node.location();
    context.replace(
        "Top level return with argument detected.",
        &location,
        &location,
        "return",
    );
}

fn redundant_constant_base(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let path = if let Some(path) = node.as_constant_path_node() {
        path
    } else if let Some(write) = node.as_constant_path_write_node() {
        write.target()
    } else if let Some(write) = node.as_constant_path_and_write_node() {
        write.target()
    } else if let Some(write) = node.as_constant_path_or_write_node() {
        write.target()
    } else if let Some(write) = node.as_constant_path_operator_write_node() {
        write.target()
    } else {
        return;
    };
    if path.parent().is_some()
        || path.delimiter_loc().as_slice() != b"::"
        || context.related_config_value("Lint/ConstantResolution", "Enabled") == Some("true")
    {
        return;
    }
    let location = path.location();
    for ancestor in context.ancestors().iter().rev() {
        if ancestor.as_module_node().is_some() {
            return;
        }
        if let Some(class) = ancestor.as_class_node() {
            let in_superclass = class.superclass().is_some_and(|superclass| {
                let superclass = superclass.location();
                superclass.start_offset() <= location.start_offset()
                    && location.end_offset() <= superclass.end_offset()
            });
            if !in_superclass {
                return;
            }
        }
    }
    let delimiter = path.delimiter_loc();
    context.remove("Remove redundant `::`.", &delimiter, &delimiter);
}
