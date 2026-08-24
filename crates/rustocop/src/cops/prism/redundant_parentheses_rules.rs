use ruby_prism::{Node, ParenthesesNode};

use super::*;

define_rule!(RedundantParenthesesRule);

define_cops! {
    RedundantParentheses => "Style/RedundantParentheses" => node_rule_aliases(
        RedundantParenthesesRule,
        on_begin => [as_parentheses_node, as_pinned_expression_node]
    ),
}

impl RedundantParenthesesRule<'_, '_, '_> {
    fn on_begin(&mut self, node: &Node<'_>) {
        if let Some(pin) = node.as_pinned_expression_node() {
            self.on_pin(&pin);
        } else if let Some(parentheses) = node.as_parentheses_node() {
            self.on_parentheses(&parentheses);
        }
    }

    fn on_pin(&mut self, node: &ruby_prism::PinnedExpressionNode<'_>) {
        let expression = node.expression();
        return_unless!(variable_node(&expression));
        let location = node.lparen_loc().start_offset()..node.rparen_loc().end_offset();
        let replacement = self.source_file().node(&expression).to_string();
        add_offense!(self, location.clone(), message: "Don't use parentheses around a variable.", |corrector| {
            corrector.replace(location, replacement);
        });
    }

    fn on_parentheses(&mut self, node: &ParenthesesNode<'_>) {
        let expressions = parenthesized_expressions(node);
        return_if!(expressions.is_empty() || self.parens_allowed(node, &expressions));
        let Some(message) = self.find_offense_message(node, &expressions) else {
            return;
        };
        self.offense(node, message);
    }

    #[allow(clippy::too_many_lines)]
    fn parens_allowed(&self, node: &ParenthesesNode<'_>, expressions: &[Node<'_>]) -> bool {
        let parent = semantic_parent(self.ancestors());
        let first = &expressions[0];
        let inner_source = self
            .source()
            .get(node.opening_loc().end_offset()..node.closing_loc().start_offset())
            .unwrap_or_default();
        let suffix = self
            .source()
            .get(node.closing_loc().end_offset()..)
            .unwrap_or_default();
        if (suffix.trim_start().starts_with('.') || suffix.trim_start().starts_with("&."))
            && first.as_call_node().is_some_and(|call| {
                argument_count(&call) > 0 && operator_method(call.name().as_slice())
            })
        {
            return true;
        }
        let whitespace_before = self.source()[..node.opening_loc().start_offset()]
            .chars()
            .last()
            .is_some_and(char::is_whitespace);
        let previous_code = self.source()[..node.opening_loc().start_offset()]
            .trim_end()
            .chars()
            .last();
        if inner_source.trim_start().starts_with("not ")
            || inner_source.trim_start().starts_with('{')
                && whitespace_before
                && previous_code != Some(',')
        {
            return true;
        }
        if self.touches_keyword(node)
            || self.multiline_keyword_argument(node, inner_source)
            || self.negative_numeric_power(node, inner_source)
            || self.keyword_argument_parentheses(node)
            || self.like_method_argument_parentheses(node)
            || self.do_end_block_in_method_chain(inner_source, parent)
        {
            return true;
        }
        if first.as_call_node().is_some_and(|call| {
            call.opening_loc().is_none()
                && call
                    .arguments()
                    .is_some_and(|arguments| !arguments.arguments().is_empty())
                && (call.receiver().is_none() || call.call_operator_loc().is_some())
        }) && parent.is_some_and(|ancestor| ancestor.as_block_node().is_none())
        {
            return true;
        }
        if expressions.len() > 1
            && parent.is_some_and(|ancestor| {
                ancestor.as_block_node().is_none() && ancestor.as_def_node().is_none()
            })
        {
            return true;
        }
        if pattern_matching_node(first)
            && (parent.is_some_and(|ancestor| {
                assignment_node(ancestor)
                    || ancestor.as_and_node().is_some()
                    || ancestor.as_or_node().is_some()
                    || ancestor.as_call_node().is_some()
            }) || self.ancestors().iter().any(|ancestor| {
                ancestor
                    .as_def_node()
                    .is_some_and(|definition| definition.equal_loc().is_some())
            }))
        {
            return true;
        }
        if parent.is_some_and(|ancestor| ancestor.as_range_node().is_some()) {
            return true;
        }
        if parent.is_some_and(|ancestor| {
            (ancestor.as_splat_node().is_some() || ancestor.as_assoc_splat_node().is_some())
                && first.as_call_node().is_none_or(|call| call.block().is_none())
        }) {
            return true;
        }
        let opening_offset = node.opening_loc().start_offset();
        let closing_offset = node.closing_loc().end_offset();
        let prefix_line = self.source()[..opening_offset]
            .rsplit_once('\n')
            .map_or(&self.source()[..opening_offset], |(_, line)| line);
        let suffix_line = self
            .source()
            .get(closing_offset..)
            .unwrap_or_default()
            .split('\n')
            .next()
            .unwrap_or_default();
        if inner_source.trim_start().starts_with('!')
            && (prefix_line.contains("&&")
                || prefix_line.contains("||")
                || suffix_line.contains("&&")
                || suffix_line.contains("||")
                || suffix_line.trim_start().starts_with('.'))
        {
            return true;
        }
        if first.as_call_node().is_some_and(|call| {
            matches!(call.name().as_slice(), b"&" | b"|" | b"^")
        }) && suffix.trim_start().starts_with('.')
        {
            return true;
        }
        if first
            .as_call_node()
            .is_some_and(|call| call_chain_starts_with_integer(&call))
            && self.source()[..opening_offset]
            .chars()
            .last()
            .is_some_and(|character| matches!(character, '+' | '-'))
        {
            return true;
        }
        if first.as_call_node().is_some_and(|call| {
            call.opening_loc().is_none()
                && argument_count(&call) > 0
                && operator_method(call.name().as_slice())
        }) && parent.is_some_and(|ancestor| {
            ancestor
                .as_call_node()
                .is_some_and(|call| operator_method(call.name().as_slice()))
        })
        {
            return true;
        }
        if first.as_call_node().is_some()
            && ["==", "===", "!=", ">", ">=", "<", "<=", "<=>", "=~", "!~"]
                .iter()
                .any(|operator| suffix.trim_start().starts_with(operator))
        {
            return true;
        }
        if first.as_call_node().is_some_and(|call| {
            call.opening_loc().is_none()
                && call
                    .arguments()
                    .is_some_and(|arguments| !arguments.arguments().is_empty())
        }) && (prefix_line.contains("&&")
            || prefix_line.contains("||")
            || suffix_line.contains("&&")
            || suffix_line.contains("||"))
        {
            return true;
        }
        if assignment_node(first)
            && parent.is_some_and(|ancestor| {
                ancestor.as_if_node().is_some()
                    || ancestor.as_unless_node().is_some()
                    || ancestor.as_while_node().is_some()
                    || ancestor.as_until_node().is_some()
            })
        {
            return true;
        }
        if assignment_node(first)
            && parent.is_some_and(|ancestor| {
                ancestor.as_and_node().is_some() || ancestor.as_or_node().is_some()
            })
        {
            return true;
        }
        if parent.is_some_and(|ancestor| ancestor.as_if_node().is_some())
            && self.ternary_parentheses_required(node)
        {
            return true;
        }
        if self.inside_ternary_branch(node) {
            return true;
        }
        if first.as_rescue_modifier_node().is_some()
            && self.rescue_parentheses_allowed(node, parent)
        {
            return true;
        }
        if parent.is_some_and(definition_parameter_node) {
            return true;
        }
        if self.first_arg_begins_with_hash_literal(node, first) {
            return true;
        }
        if let Some(range) = first.as_range_node() {
            if self.non_terminal_body_range(node, &range) {
                return true;
            }
        }
        false
    }

    fn rescue_parentheses_allowed(
        &self,
        node: &ParenthesesNode<'_>,
        parent: Option<&Node<'_>>,
    ) -> bool {
        if parent.is_some_and(|ancestor| {
            ancestor.as_call_node().is_some()
                || ancestor.as_array_node().is_some()
                || ancestor.as_assoc_node().is_some()
                || ancestor.as_hash_node().is_some()
                || ancestor.as_case_node().is_some()
                || ancestor.as_while_node().is_some()
                || ancestor.as_until_node().is_some()
        }) {
            return true;
        }
        if parent.is_some_and(|ancestor| ancestor.as_parentheses_node().is_some())
            && self
                .ancestors()
                .iter()
                .any(|ancestor| ancestor.as_call_node().is_some())
        {
            return true;
        }
        parent.and_then(Node::as_if_node).is_some_and(|conditional| {
            let ternary = conditional.if_keyword_loc().is_none()
                && conditional.then_keyword_loc().is_some()
                && conditional.end_keyword_loc().is_none();
            ternary || same_location(&conditional.predicate().location(), &node.location())
        })
    }

    fn inside_ternary_branch(&self, node: &ParenthesesNode<'_>) -> bool {
        self.ancestors().iter().find_map(Node::as_if_node).is_some_and(|conditional| {
            let ternary = conditional.if_keyword_loc().is_none()
                && conditional.then_keyword_loc().is_some()
                && conditional.end_keyword_loc().is_none();
            ternary && !same_location(&conditional.predicate().location(), &node.location())
        })
    }

    fn non_terminal_body_range(
        &self,
        node: &ParenthesesNode<'_>,
        range: &ruby_prism::RangeNode<'_>,
    ) -> bool {
        let Some(statements) = self
            .ancestors()
            .iter()
            .rev()
            .find_map(Node::as_statements_node)
        else {
            return false;
        };
        let body = statements.body().iter().collect::<Vec<_>>();
        let Some(index) = body
            .iter()
            .position(|statement| same_location(&statement.location(), &node.location()))
        else {
            return false;
        };
        range.left().is_none() && index > 0
            || range.right().is_none() && index + 1 < body.len()
    }

    fn touches_keyword(&self, node: &ParenthesesNode<'_>) -> bool {
        let opening = node.opening_loc().start_offset();
        let closing = node.closing_loc().end_offset();
        let before = self.source()[..opening].chars().last();
        let after = self.source().get(closing..).and_then(|source| source.chars().next());
        before.is_some_and(|character| character.is_ascii_alphabetic())
            || after.is_some_and(|character| character.is_ascii_alphabetic())
    }

    fn multiline_keyword_argument(&self, node: &ParenthesesNode<'_>, inner: &str) -> bool {
        if !inner.contains('\n') {
            return false;
        }
        let prefix = self.source()[..node.opening_loc().start_offset()].trim_end();
        let keyword = prefix
            .split(|character: char| !character.is_ascii_alphanumeric())
            .next_back()
            .unwrap_or_default();
        matches!(keyword, "return" | "break" | "next" | "yield" | "super")
    }

    fn negative_numeric_power(&self, node: &ParenthesesNode<'_>, inner: &str) -> bool {
        inner.trim_start().starts_with('-')
            && self
                .source()
                .get(node.closing_loc().end_offset()..)
                .is_some_and(|source| source.starts_with("**"))
    }

    fn keyword_argument_parentheses(&self, node: &ParenthesesNode<'_>) -> bool {
        let opening = node.opening_loc().start_offset();
        let prefix = &self.source()[..opening];
        let immediate = prefix
            .trim_end_matches(|character: char| character.is_whitespace())
            .split(|character: char| !character.is_ascii_alphanumeric() && !matches!(character, '_' | '?' | '!'))
            .next_back()
            .unwrap_or_default();
        let touches = !prefix.ends_with(char::is_whitespace);
        touches
            && matches!(
                immediate,
                "return" | "break" | "next" | "yield" | "super" | "defined?" | "while" | "until" | "rescue" | "when"
            )
    }

    fn like_method_argument_parentheses(&self, node: &ParenthesesNode<'_>) -> bool {
        let opening = node.opening_loc().start_offset();
        let closing = node.closing_loc().end_offset();
        let prefix = &self.source()[..opening];
        let whitespace_before = prefix
            .chars()
            .last()
            .is_some_and(char::is_whitespace);
        if !whitespace_before {
            return false;
        }
        let token = prefix
            .trim_end()
            .split(|character: char| !(character.is_ascii_alphanumeric() || "_?!".contains(character)))
            .next_back()
            .unwrap_or_default();
        if token.is_empty()
            || matches!(token, "return" | "break" | "next" | "yield" | "super" | "rescue" | "when")
        {
            return false;
        }
        let after = self.source().get(closing..).unwrap_or_default();
        semantic_parent(self.ancestors()).is_some_and(|parent| {
            parent.as_call_node().is_some_and(|call| {
                call.arguments()
                    .is_some_and(|arguments| arguments.arguments().len() == 1)
            })
        }) && (after.starts_with([',', '{'])
            || after.trim_start().starts_with('{')
            || after.is_empty())
    }

    fn first_arg_begins_with_hash_literal(&self, node: &ParenthesesNode<'_>, first: &Node<'_>) -> bool {
        return_unless!(method_chain_begins_with_hash_literal(first), false);
        let opening = node.opening_loc().start_offset();
        let whitespace_before = self.source()[..opening]
            .chars()
            .last()
            .is_some_and(char::is_whitespace);
        let suffix = self
            .source()
            .get(node.closing_loc().end_offset()..)
            .unwrap_or_default();
        if whitespace_before && suffix.starts_with(',') {
            return true;
        }
        let Some(parent) = semantic_parent(self.ancestors()).and_then(Node::as_call_node) else {
            return false;
        };
        parent.opening_loc().is_none()
            && parent
                .arguments()
                .and_then(|arguments| arguments.arguments().first())
                .is_some_and(|argument| same_location(&argument.location(), &node.location()))
    }

    fn ternary_parentheses_required(&self, node: &ParenthesesNode<'_>) -> bool {
        let Some(parent) = semantic_parent(self.ancestors()).and_then(Node::as_if_node) else {
            return false;
        };
        let ternary = parent.if_keyword_loc().is_none()
            && parent.then_keyword_loc().is_some()
            && parent.end_keyword_loc().is_none();
        if !ternary {
            return false;
        }
        let enabled = self
            .related_config_value("Style/TernaryParentheses", "Enabled")
            .is_none_or(|value| value != "false");
        let style = self
            .related_config_value("Style/TernaryParentheses", "EnforcedStyle")
            .unwrap_or_default();
        enabled
            && matches!(style, "require_parentheses" | "require_parentheses_when_complex")
            && parent
                .predicate()
                .as_parentheses_node()
                .is_some_and(|predicate| same_location(&predicate.location(), &node.location()))
    }

    fn find_offense_message(
        &self,
        node: &ParenthesesNode<'_>,
        expressions: &[Node<'_>],
    ) -> Option<&'static str> {
        let first = &expressions[0];
        let parent = semantic_parent(self.ancestors());
        let inner_source = self
            .source()
            .get(node.opening_loc().end_offset()..node.closing_loc().start_offset())?
            .trim();

        if keyword_node(first, inner_source) {
            return Some("a keyword");
        }
        if literal_node(first) {
            return Some("a literal");
        }
        if variable_node(first) {
            return Some("a variable");
        }
        if constant_node(first) {
            return Some("a constant");
        }
        if parent.is_some_and(|ancestor| ancestor.as_block_node().is_some())
            && (self.parentheses_are_only_statement(node) || first.as_range_node().is_some()) {
                return Some("block body");
            }
        if assignment_node(first)
            && parent.is_none_or(|ancestor| ancestor.as_parentheses_node().is_some())
        {
            return Some("an assignment");
        }
        if lambda_or_proc(first) {
            if !inner_source.trim_start().starts_with("->")
                && inner_source.contains(" do")
                && inner_source.contains("end")
            {
                return None;
            }
            return Some("an expression");
        }
        if parent.is_some_and(interpolation_parent) {
            return Some("an interpolated expression");
        }
        if argument_of_parenthesized_method_call(node, first, parent) {
            return Some("a method argument");
        }
        if first.as_rescue_modifier_node().is_some() {
            return Some("a one-line rescue");
        }
        if pattern_matching_node(first) {
            return Some("a one-line pattern matching");
        }
        if first.as_and_node().is_some() || first.as_or_node().is_some() {
            return self.logical_expression_message(first, parent);
        }
        if let Some(call) = first.as_call_node() {
            let name = call.name().as_slice();
            if matches!(name, b"!" | b"~" | b"+@" | b"-@") {
                return Some("a unary operation");
            }
            if comparison_method(name) {
                return parent.is_none().then_some("a comparison expression");
            }
            if self.do_end_block_in_method_chain(inner_source, parent) {
                return None;
            }
            if operator_method(name) {
                if name == b"/" {
                    return None;
                }
                return parent
                    .is_some_and(|ancestor| {
                        ancestor
                            .as_array_node()
                            .is_some_and(|array| array.elements().len() == 1)
                            || ancestor.as_return_node().is_some()
                            || ancestor.as_next_node().is_some()
                            || ancestor.as_break_node().is_some()
                            || ancestor.as_yield_node().is_some()
                    })
                    .then_some("a method call");
            }
            return Some("a method call");
        }
        if first.as_parentheses_node().is_some() {
            if first
                .as_parentheses_node()
                .and_then(|parentheses| {
                    parenthesized_expressions(&parentheses)
                        .into_iter()
                        .next()
                })
                .is_some_and(|inner| assignment_node(&inner))
            {
                return None;
            }
            return Some("a literal");
        }
        None
    }

    fn logical_expression_message(
        &self,
        node: &Node<'_>,
        parent: Option<&Node<'_>>,
    ) -> Option<&'static str> {
        let operator = node
            .as_and_node()
            .map(|logical| logical.operator_loc().as_slice())
            .or_else(|| node.as_or_node().map(|logical| logical.operator_loc().as_slice()))?;
        if self.source_file().at(&node.location()).contains('\n')
            && self.related_config_value(
                "Style/ParenthesesAroundCondition",
                "AllowInMultilineConditions",
            ) == Some("true")
            && self.related_config_value("Style/ParenthesesAroundCondition", "Enabled")
                != Some("false")
        {
            return None;
        }
        if matches!(operator, b"and" | b"or") && parent.is_some() {
            return None;
        }
        if parent.is_some_and(|ancestor| {
            ancestor.as_or_node().is_some()
                || ancestor.as_call_node().is_some()
                || ancestor.as_splat_node().is_some()
                || ancestor.as_assoc_splat_node().is_some()
        }) {
            return None;
        }
        if node.as_or_node().is_some()
            && parent.is_some_and(|ancestor| ancestor.as_and_node().is_some())
        {
            return None;
        }
        if parent.and_then(Node::as_if_node).is_some_and(|conditional| {
            conditional.if_keyword_loc().is_none()
                && conditional.then_keyword_loc().is_some()
                && conditional.end_keyword_loc().is_none()
        }) {
            return None;
        }
        Some("a logical expression")
    }

    fn do_end_block_in_method_chain(
        &self,
        inner_source: &str,
        parent: Option<&Node<'_>>,
    ) -> bool {
        !inner_source.trim_start().starts_with("->")
            && (parent.is_some_and(|ancestor| ancestor.as_call_node().is_some())
            || self
                .ancestors()
                .iter()
                .any(|ancestor| ancestor.as_call_node().is_some()))
            && inner_source.contains(" do")
            && inner_source.contains("end")
    }

    fn parentheses_are_only_statement(&self, node: &ParenthesesNode<'_>) -> bool {
        self.ancestors()
            .iter()
            .rev()
            .find_map(Node::as_statements_node)
            .is_none_or(|statements| {
                let body = statements.body();
                body.len() == 1
                    && body.first().is_some_and(|statement| {
                        same_location(&statement.location(), &node.location())
                    })
            })
    }

    fn offense(&mut self, node: &ParenthesesNode<'_>, description: &str) {
        let node_location = node.location();
        let location = node_location.start_offset()..node_location.end_offset();
        let opening = node.opening_loc();
        let closing = node.closing_loc();
        let message = format!("Don't use parentheses around {description}.");
        let inner = self
            .source()
            .get(opening.end_offset()..closing.start_offset())
            .unwrap_or_default();
        let mut replacement = inner.trim().to_string();
        if self
            .source()
            .get(closing.end_offset()..)
            .is_some_and(|source| source.starts_with('?'))
        {
            replacement.push(' ');
        }
        let suffix = self.source().get(closing.end_offset()..).unwrap_or_default();
        if inner
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .is_some_and(|line| line.contains('#'))
            && suffix.starts_with('.')
        {
            replacement.push('\n');
        }
        let heredoc_comma_end = if replacement.starts_with("<<") {
            suffix
                .find(',')
                .filter(|comma| !suffix[..*comma].contains('\n'))
                .map(|comma| closing.end_offset() + comma + 1)
        } else {
            None
        };
        if heredoc_comma_end.is_some() {
            if let Some(newline) = replacement.find('\n') {
                replacement.insert(newline, ',');
            }
        }
        add_offense!(self, location.clone(), message: message, |corrector| {
            corrector.replace(location, replacement);
            if let Some(comma_end) = heredoc_comma_end {
                corrector.remove(closing.end_offset()..comma_end);
            }
        });
    }
}

fn parenthesized_expressions<'pr>(node: &ParenthesesNode<'pr>) -> Vec<Node<'pr>> {
    node.body()
        .and_then(|body| body.as_statements_node())
        .map(|statements| statements.body().iter().collect())
        .unwrap_or_default()
}

fn literal_node(node: &Node<'_>) -> bool {
    node.as_string_node().is_some()
        || node.as_interpolated_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_interpolated_symbol_node().is_some()
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_rational_node().is_some()
        || node.as_imaginary_node().is_some()
        || node.as_array_node().is_some()
        || node.as_hash_node().is_some()
        || node.as_nil_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_regular_expression_node().is_some()
        || node.as_interpolated_regular_expression_node().is_some()
}

fn variable_node(node: &Node<'_>) -> bool {
    node.as_local_variable_read_node().is_some()
        || node.as_instance_variable_read_node().is_some()
        || node.as_class_variable_read_node().is_some()
        || node.as_global_variable_read_node().is_some()
        || node.as_back_reference_read_node().is_some()
        || node.as_numbered_reference_read_node().is_some()
}

fn constant_node(node: &Node<'_>) -> bool {
    node.as_constant_read_node().is_some() || node.as_constant_path_node().is_some()
}

fn assignment_node(node: &Node<'_>) -> bool {
    node.as_multi_write_node().is_some()
        || node.as_local_variable_write_node().is_some()
        || node.as_instance_variable_write_node().is_some()
        || node.as_class_variable_write_node().is_some()
        || node.as_global_variable_write_node().is_some()
        || node.as_constant_write_node().is_some()
        || node.as_constant_path_write_node().is_some()
        || node.as_local_variable_or_write_node().is_some()
        || node.as_instance_variable_or_write_node().is_some()
        || node.as_class_variable_or_write_node().is_some()
        || node.as_global_variable_or_write_node().is_some()
        || node.as_local_variable_and_write_node().is_some()
        || node.as_instance_variable_and_write_node().is_some()
        || node.as_class_variable_and_write_node().is_some()
        || node.as_global_variable_and_write_node().is_some()
        || node
            .as_call_node()
            .is_some_and(|call| {
                let name = call.name().as_slice();
                name.ends_with(b"=")
                    && !matches!(name, b"==" | b"===" | b"!=" | b"<=" | b">=" | b"=~" | b"!~")
            })
}

fn keyword_node(node: &Node<'_>, source: &str) -> bool {
    let keyword = node.as_return_node().is_some()
        || node.as_break_node().is_some()
        || node.as_next_node().is_some()
        || node.as_yield_node().is_some()
        || node.as_super_node().is_some()
        || node.as_forwarding_super_node().is_some()
        || node.as_defined_node().is_some()
        || node.as_self_node().is_some();
    let special = matches!(source, "__FILE__" | "__LINE__" | "__ENCODING__" | "redo" | "retry" | "self");
    if special {
        return true;
    }
    if !keyword {
        return false;
    }
    let name = source
        .split(|character: char| character.is_whitespace() || character == '(')
        .next()
        .unwrap_or_default();
    source == name || source[name.len()..].starts_with('(')
}

fn lambda_or_proc(node: &Node<'_>) -> bool {
    node.as_lambda_node().is_some()
        || node.as_call_node().is_some_and(|call| {
            matches!(call.name().as_slice(), b"lambda" | b"proc") && call.block().is_some()
        })
}

fn interpolation_parent(node: &Node<'_>) -> bool {
    node.as_embedded_statements_node().is_some()
        || node.as_interpolated_string_node().is_some()
        || node.as_interpolated_symbol_node().is_some()
}

fn call_chain_starts_with_integer(call: &ruby_prism::CallNode<'_>) -> bool {
    let Some(receiver) = call.receiver() else {
        return false;
    };
    receiver.as_integer_node().is_some()
        || receiver
            .as_call_node()
            .is_some_and(|receiver| call_chain_starts_with_integer(&receiver))
}

fn argument_of_parenthesized_method_call(
    node: &ParenthesesNode<'_>,
    expression: &Node<'_>,
    parent: Option<&Node<'_>>,
) -> bool {
    if expression.as_if_node().is_some()
        || expression.as_unless_node().is_some()
        || expression.as_while_node().is_some()
        || expression.as_until_node().is_some()
        || expression.as_rescue_modifier_node().is_some()
    {
        return false;
    }
    if expression.as_call_node().is_some_and(|call| {
        call.opening_loc().is_none()
            && call.arguments().is_some_and(|arguments| !arguments.arguments().is_empty())
            && (call.receiver().is_none() || call.call_operator_loc().is_some())
    }) || expression.as_rescue_modifier_node().is_some()
        || expression.as_match_predicate_node().is_some()
        || expression.as_match_required_node().is_some()
    {
        return false;
    }
    let Some(call) = parent.and_then(Node::as_call_node) else {
        return false;
    };
    call.opening_loc().is_some()
        && call
            .arguments()
            .is_some_and(|arguments| {
                arguments
                    .arguments()
                    .iter()
                    .any(|argument| same_location(&argument.location(), &node.location()))
            })
}

fn method_chain_begins_with_hash_literal(node: &Node<'_>) -> bool {
    if node.as_hash_node().is_some() {
        return true;
    }
    node.as_call_node()
        .and_then(|call| call.receiver())
        .is_some_and(|receiver| method_chain_begins_with_hash_literal(&receiver))
}

fn definition_parameter_node(node: &Node<'_>) -> bool {
    node.as_parameters_node().is_some()
        || node.as_block_parameters_node().is_some()
        || node.as_multi_target_node().is_some()
}

fn comparison_method(name: &[u8]) -> bool {
    matches!(name, b"==" | b"===" | b"!=" | b">" | b">=" | b"<" | b"<=" | b"<=>" | b"=~" | b"!~")
}

fn operator_method(name: &[u8]) -> bool {
    matches!(
        name,
        b"+" | b"-" | b"*" | b"/" | b"%" | b"**" | b"<<" | b">>" | b"&" | b"|" | b"^"
    )
}

fn pattern_matching_node(node: &Node<'_>) -> bool {
    node.as_match_predicate_node().is_some() || node.as_match_required_node().is_some()
}

fn semantic_parent<'a>(ancestors: &'a [Node<'a>]) -> Option<&'a Node<'a>> {
    ancestors.iter().rev().find(|node| {
        node.as_statements_node().is_none()
            && node.as_arguments_node().is_none()
            && node.as_program_node().is_none()
    })
}

fn same_location(left: &ruby_prism::Location<'_>, right: &ruby_prism::Location<'_>) -> bool {
    left.start_offset() == right.start_offset() && left.end_offset() == right.end_offset()
}
