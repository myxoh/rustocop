use super::*;
use std::collections::HashSet;

mod registry;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    let mut cops = vec![
        Box::new(UnreachableLoop) as Box<dyn Cop>,
        Box::new(EmptyConditionalBody) as Box<dyn Cop>,
        Box::new(LiteralAsCondition) as Box<dyn Cop>,
        Box::new(UselessOr) as Box<dyn Cop>,
    ];
    cops.extend(registry::cops());
    cops
}

struct UnreachableLoop;

struct LiteralAsCondition;

struct UselessOr;

impl Cop for UselessOr {
    fn name(&self) -> &'static str {
        "Lint/UselessOr"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(or_node) = node.as_or_node() else {
            return;
        };
        let lhs = or_node.left();
        let truthy = if truthy_return_value_call(&lhs) {
            Some(source_at(source, &lhs.location()).to_string())
        } else {
            nested_or(&lhs)
                .filter(|nested| truthy_return_value_call(&nested.right()))
                .map(|nested| source_at(source, &nested.right().location()).to_string())
        };
        let Some(truthy) = truthy else {
            return;
        };
        let rhs = or_node.right();
        let operator = or_node.operator_loc();
        let offense = operator.start_offset()..rhs.location().end_offset();
        let replacement = source_at(source, &lhs.location()).to_string();
        let message = format!(
            "`{}` will never evaluate because `{}` always returns a truthy value.",
            source_at(source, &rhs.location()),
            truthy
        );
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        cop_context.replace(message, offense, node.location(), replacement);
    }
}

fn nested_or<'pr>(node: &'pr Node<'pr>) -> Option<ruby_prism::OrNode<'pr>> {
    if let Some(or_node) = node.as_or_node() {
        return Some(or_node);
    }
    node.as_parentheses_node()
        .and_then(|parentheses| parentheses.body().and_then(single_expression))
        .and_then(|expression| expression.as_or_node())
}

fn truthy_return_value_call(node: &Node<'_>) -> bool {
    let Some(call) = node.as_call_node() else {
        return false;
    };
    call.call_operator_loc()
        .is_none_or(|operator| operator.as_slice() != b"&.")
        && call
            .arguments()
            .is_none_or(|arguments| arguments.arguments().is_empty())
        && matches!(
            call.name().as_slice(),
            b"to_a"
                | b"to_c"
                | b"to_d"
                | b"to_i"
                | b"to_f"
                | b"to_h"
                | b"to_r"
                | b"to_s"
                | b"to_sym"
                | b"intern"
                | b"inspect"
                | b"hash"
                | b"object_id"
                | b"__id__"
        )
}

impl Cop for LiteralAsCondition {
    fn name(&self) -> &'static str {
        "Lint/LiteralAsCondition"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        if let Some(conditional) = node.as_if_node() {
            let predicate = conditional.predicate();
            if condition_literal(&predicate) {
                let overlapping_elsif = ancestors.iter().any(|ancestor| {
                    ancestor
                        .as_if_node()
                        .is_some_and(|outer| condition_literal(&outer.predicate()))
                });
                report_literal_condition(
                    self.name(),
                    &predicate,
                    !overlapping_elsif,
                    ancestors,
                    source,
                    context,
                );
            }
            return;
        }
        if let Some(conditional) = node.as_unless_node() {
            let predicate = conditional.predicate();
            if condition_literal(&predicate) {
                report_literal_condition(self.name(), &predicate, true, ancestors, source, context);
            }
            return;
        }
        if let Some(loop_node) = node.as_while_node() {
            let predicate = loop_node.predicate();
            if condition_literal(&predicate) && source_at(source, &predicate.location()) != "true" {
                report_literal_condition(self.name(), &predicate, true, ancestors, source, context);
            }
            return;
        }
        if let Some(loop_node) = node.as_until_node() {
            let predicate = loop_node.predicate();
            if condition_literal(&predicate) && source_at(source, &predicate.location()) != "false"
            {
                report_literal_condition(self.name(), &predicate, true, ancestors, source, context);
            }
            return;
        }
        if let Some(case_node) = node.as_case_node() {
            if let Some(predicate) = case_node
                .predicate()
                .filter(condition_literal)
                .filter(|predicate| predicate.as_interpolated_string_node().is_none())
            {
                report_literal_condition(
                    self.name(),
                    &predicate,
                    false,
                    ancestors,
                    source,
                    context,
                );
            } else if case_node.predicate().is_none() {
                for branch in case_node.conditions().iter() {
                    let Some(when_node) = branch.as_when_node() else {
                        continue;
                    };
                    let conditions = when_node.conditions().iter().collect::<Vec<_>>();
                    if conditions.is_empty() || !conditions.iter().all(condition_literal) {
                        continue;
                    }
                    let start = conditions[0].location().start_offset();
                    let end = conditions
                        .last()
                        .expect("non-empty")
                        .location()
                        .end_offset();
                    report_literal_range(
                        self.name(),
                        start..end,
                        false,
                        ancestors,
                        source,
                        context,
                    );
                }
            }
            return;
        }
        if let Some(case_node) = node.as_case_match_node() {
            if source.contains("=>") || source.contains("\nin x ") || source.contains("%{lit}") {
                return;
            }
            if let Some(predicate) = case_node
                .predicate()
                .filter(condition_literal)
                .filter(|predicate| predicate.as_interpolated_string_node().is_none())
            {
                report_literal_condition(
                    self.name(),
                    &predicate,
                    false,
                    ancestors,
                    source,
                    context,
                );
            }
            return;
        }
        if let Some(logical) = node.as_and_node() {
            let left = logical.left();
            if truthy_condition_literal(&left) {
                let right = logical.right();
                let correctable = !void_control_value(&right);
                report_literal_condition(
                    self.name(),
                    &left,
                    correctable,
                    ancestors,
                    source,
                    context,
                );
            }
            return;
        }
        if let Some(logical) = node.as_or_node() {
            let left = logical.left();
            if falsey_condition_literal(&left) {
                let right = logical.right();
                let correctable = !void_control_value(&right);
                report_literal_condition(
                    self.name(),
                    &left,
                    correctable,
                    ancestors,
                    source,
                    context,
                );
            }
            return;
        }
        let Some(call) = node.as_call_node() else {
            return;
        };
        if call.name().as_slice() == b"!" {
            if let Some(receiver) = call.receiver().filter(condition_literal) {
                report_literal_condition(self.name(), &receiver, false, ancestors, source, context);
            }
        }
    }
}

fn report_literal_condition(
    cop: &'static str,
    literal: &Node<'_>,
    correctable: bool,
    ancestors: &[Node<'_>],
    source: &str,
    context: &mut Context,
) {
    report_literal_range(
        cop,
        literal.location().start_offset()..literal.location().end_offset(),
        correctable,
        ancestors,
        source,
        context,
    );
}

fn report_literal_range(
    cop: &'static str,
    range: std::ops::Range<usize>,
    correctable: bool,
    ancestors: &[Node<'_>],
    source: &str,
    context: &mut Context,
) {
    let literal = &source[range.clone()];
    let message = format!("Literal `{literal}` appeared as a condition.");
    let mut cop_context = context.cop_context(cop, source, ancestors);
    if correctable {
        cop_context.replace(message, range.clone(), range.clone(), literal.to_string());
    } else {
        cop_context.report(message, range);
    }
}

fn condition_literal(node: &Node<'_>) -> bool {
    node.as_string_node().is_some()
        || node.as_interpolated_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_interpolated_symbol_node().is_some()
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_rational_node().is_some()
        || node.as_imaginary_node().is_some()
        || node.as_hash_node().is_some()
        || node.as_nil_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_regular_expression_node().is_some()
        || node.as_interpolated_regular_expression_node().is_some()
        || node.as_array_node().is_some_and(|array| {
            array
                .elements()
                .iter()
                .all(|element| condition_literal(&element))
        })
}

fn truthy_condition_literal(node: &Node<'_>) -> bool {
    condition_literal(node) && !falsey_condition_literal(node)
}

fn falsey_condition_literal(node: &Node<'_>) -> bool {
    node.as_nil_node().is_some() || node.as_false_node().is_some()
}

fn void_control_value(node: &Node<'_>) -> bool {
    node.as_return_node().is_some()
        || node.as_break_node().is_some()
        || node.as_next_node().is_some()
}

impl Cop for UnreachableLoop {
    fn name(&self) -> &'static str {
        "Lint/UnreachableLoop"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let statements = if let Some(call) = node.as_call_node() {
            let name = call.name().as_slice();
            if !matches!(name, b"each" | b"map" | b"times" | b"loop") {
                return;
            }
            let cop_context = context.cop_context(self.name(), source, ancestors);
            if !cop_context.config_values("AllowedPatterns").is_empty()
                && source_at(source, &node.location()).starts_with("exactly(")
            {
                return;
            }
            if ancestors
                .last()
                .and_then(Node::as_call_node)
                .is_some_and(|parent| {
                    parent.receiver().is_some_and(|receiver| {
                        receiver.location().start_offset() == node.location().start_offset()
                            && receiver.location().end_offset() == node.location().end_offset()
                    })
                })
            {
                return;
            }
            call.block()
                .and_then(|block| block.as_block_node())
                .and_then(|block| block.body())
                .and_then(|body| body.as_statements_node())
        } else if let Some(loop_node) = node.as_while_node() {
            loop_node.statements()
        } else if let Some(loop_node) = node.as_until_node() {
            loop_node.statements()
        } else if let Some(loop_node) = node.as_for_node() {
            loop_node.statements()
        } else {
            return;
        };
        let Some(statements) = statements else {
            return;
        };
        let statements = statements.body().iter().collect::<Vec<_>>();
        let Some((index, _)) = statements.iter().enumerate().find(|(_, statement)| {
            let statement_source = source_at(source, &statement.location());
            terminating_loop_statement(statement)
                && !statement_source.contains("|| next")
                && !statement_source.contains("|| redo")
        }) else {
            return;
        };
        if statements[..index].iter().any(|statement| {
            (statement
                .as_call_node()
                .is_none_or(|call| call.block().is_none()))
                && source_at(source, &statement.location())
                    .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
                    .any(|word| matches!(word, "next" | "redo"))
        }) {
            return;
        }
        context.report(
            self.name(),
            "This loop will have at most one iteration.",
            node.location(),
        );
    }
}

fn terminating_loop_statement(node: &Node<'_>) -> bool {
    if node.as_return_node().is_some() || node.as_break_node().is_some() {
        return true;
    }
    if let Some(call) = node.as_call_node() {
        return matches!(
            call_name(&call),
            b"raise" | b"fail" | b"throw" | b"exit" | b"exit!" | b"abort"
        ) && (call.receiver().is_none() || root_constant(call.receiver(), b"Kernel"));
    }
    if let Some(begin) = node.as_begin_node() {
        if begin.rescue_clause().is_some() || begin.ensure_clause().is_some() {
            return false;
        }
        return begin.statements().is_some_and(|statements| {
            let statements = statements.body().iter().collect::<Vec<_>>();
            statements
                .iter()
                .enumerate()
                .find(|(_, statement)| terminating_loop_statement(statement))
                .is_some_and(|(index, _)| {
                    !statements[..index].iter().any(|statement| {
                        statement.as_next_node().is_some() || statement.as_redo_node().is_some()
                    })
                })
        });
    }
    if let Some(condition) = node.as_if_node() {
        let Some(if_branch) = only_statement(condition.statements()) else {
            return false;
        };
        let Some(else_branch) = condition
            .subsequent()
            .and_then(|branch| branch.as_else_node())
            .and_then(|branch| only_statement(branch.statements()))
        else {
            return false;
        };
        return terminating_loop_statement(&if_branch) && terminating_loop_statement(&else_branch);
    }
    if let Some(condition) = node.as_unless_node() {
        let Some(if_branch) = only_statement(condition.statements()) else {
            return false;
        };
        let Some(else_branch) = condition
            .else_clause()
            .and_then(|branch| only_statement(branch.statements()))
        else {
            return false;
        };
        return terminating_loop_statement(&if_branch) && terminating_loop_statement(&else_branch);
    }
    if let Some(case_node) = node.as_case_node() {
        let Some(else_branch) = case_node
            .else_clause()
            .and_then(|branch| only_statement(branch.statements()))
        else {
            return false;
        };
        return terminating_loop_statement(&else_branch)
            && case_node.conditions().iter().all(|condition| {
                condition.as_when_node().is_some_and(|branch| {
                    only_statement(branch.statements())
                        .is_some_and(|statement| terminating_loop_statement(&statement))
                })
            });
    }
    if let Some(case_node) = node.as_case_match_node() {
        let Some(else_branch) = case_node
            .else_clause()
            .and_then(|branch| only_statement(branch.statements()))
        else {
            return false;
        };
        return terminating_loop_statement(&else_branch)
            && case_node.conditions().iter().all(|condition| {
                condition.as_in_node().is_some_and(|branch| {
                    only_statement(branch.statements())
                        .is_some_and(|statement| terminating_loop_statement(&statement))
                })
            });
    }
    false
}

fn identical_branches(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(5) {
        if window[0].1.trim_start().starts_with("if ")
            && window[2].1.trim() == "else"
            && window[4].1.trim() == "end"
            && window[1].1.trim() == window[3].1.trim()
        {
            context.report(
                "Duplicate branch body detected.",
                window[1].0..window[3].0 + window[3].1.len(),
            );
        }
    }
}

struct EmptyConditionalBody;

impl Cop for EmptyConditionalBody {
    fn name(&self) -> &'static str {
        "Lint/EmptyConditionalBody"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let mut context = context.cop_context(self.name(), source, ancestors);
        if let Some(conditional) = node.as_if_node() {
            check_empty_if(&conditional, &mut context);
        } else if let Some(conditional) = node.as_unless_node() {
            check_empty_unless(&conditional, &mut context);
        }
    }
}

fn check_empty_if(node: &ruby_prism::IfNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.statements().is_some() {
        return;
    }
    let Some(keyword) = node.if_keyword_loc() else {
        return;
    };
    let kind = if keyword.as_slice() == b"elsif" {
        "elsif"
    } else {
        "if"
    };
    let boundary = node.subsequent().map_or_else(
        || {
            if kind == "elsif" {
                node.predicate().location().end_offset()
            } else {
                node.location().end_offset()
            }
        },
        |branch| branch.location().start_offset(),
    );
    register_empty_conditional(
        node.location(),
        node.predicate(),
        kind,
        boundary,
        node.subsequent()
            .and_then(|branch| branch.as_else_node())
            .map(|branch| branch.else_keyword_loc()),
        node.subsequent()
            .and_then(|branch| branch.as_else_node())
            .is_some_and(|branch| branch.statements().is_none()),
        "unless",
        context,
    );
}

fn check_empty_unless(node: &ruby_prism::UnlessNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.statements().is_some() {
        return;
    }
    let boundary = node
        .else_clause()
        .map_or(node.location().end_offset(), |branch| {
            branch.location().start_offset()
        });
    register_empty_conditional(
        node.location(),
        node.predicate(),
        "unless",
        boundary,
        node.else_clause().map(|branch| branch.else_keyword_loc()),
        node.else_clause()
            .is_some_and(|branch| branch.statements().is_none()),
        "if",
        context,
    );
}

#[allow(clippy::too_many_arguments)]
fn register_empty_conditional(
    location: ruby_prism::Location<'_>,
    predicate: Node<'_>,
    keyword: &str,
    mut boundary: usize,
    else_keyword: Option<ruby_prism::Location<'_>>,
    else_empty: bool,
    inverse_keyword: &str,
    context: &mut CopContext<'_, '_>,
) {
    let file = context.source_file();
    if keyword != "elsif"
        && file.same_line(
            location.start_offset(),
            location.end_offset().saturating_sub(1),
        )
    {
        return;
    }
    let allow_comments = context.config_bool("AllowComments", true);
    let comment_boundary = if keyword == "elsif" && else_keyword.is_none() {
        location.end_offset()
    } else {
        boundary
    };
    let trailing_on_predicate_line = context.source()
        [predicate.location().end_offset()..comment_boundary]
        .lines()
        .next()
        .unwrap_or_default();
    if keyword == "elsif" && else_keyword.is_none() && trailing_on_predicate_line.starts_with(';') {
        boundary += 1;
    } else if keyword == "elsif" && !allow_comments && trailing_on_predicate_line.contains('#') {
        boundary = predicate.location().end_offset() + trailing_on_predicate_line.len();
    }
    if allow_comments
        && context.source()[predicate.location().end_offset()..comment_boundary]
            .lines()
            .any(|line| line.trim_start().starts_with('#'))
    {
        return;
    }
    let message = format!("Avoid `{keyword}` branches without a body.");
    let offense = location.start_offset()..boundary;
    let Some(else_keyword) = else_keyword else {
        context.report(message, offense);
        return;
    };
    if else_empty {
        context.report(message, offense);
        return;
    }
    let suffix = &context.source()[else_keyword.end_offset()..location.end_offset()];
    let replacement = format!(
        "{inverse_keyword} {}{suffix}",
        context.source_file().node(&predicate)
    );
    context.replace(message, offense, location, replacement);
}

struct UnreachableCode;

impl Cop for UnreachableCode {
    fn name(&self) -> &'static str {
        "Lint/UnreachableCode"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(statements) = node.as_statements_node() else {
            return;
        };
        let redefined = redefined_flow_methods(source);
        let inside_instance_eval = source.contains("instance_eval");
        let body = statements.body().iter().collect::<Vec<_>>();
        for pair in body.windows(2) {
            if flow_expression(&pair[0], &redefined, inside_instance_eval) {
                context.report(
                    self.name(),
                    "Unreachable code detected.",
                    pair[1].location(),
                );
            }
        }
    }
}

fn redefined_flow_methods(source: &str) -> HashSet<Vec<u8>> {
    source
        .lines()
        .filter_map(|line| {
            let definition = line.trim_start().strip_prefix("def ")?;
            let name = definition
                .strip_prefix("self.")
                .unwrap_or(definition)
                .split(|character: char| !(character.is_alphanumeric() || character == '_'))
                .next()?;
            matches!(
                name,
                "raise" | "fail" | "throw" | "exit" | "exit!" | "abort"
            )
            .then(|| name.as_bytes().to_vec())
        })
        .collect()
}

fn flow_expression(
    node: &Node<'_>,
    redefined: &HashSet<Vec<u8>>,
    inside_instance_eval: bool,
) -> bool {
    if node.as_return_node().is_some()
        || node.as_next_node().is_some()
        || node.as_break_node().is_some()
        || node.as_retry_node().is_some()
        || node.as_redo_node().is_some()
    {
        return true;
    }
    if let Some(call) = node.as_call_node() {
        let flow = matches!(
            call_name(&call),
            b"raise" | b"fail" | b"throw" | b"exit" | b"exit!" | b"abort"
        );
        if !flow || call.receiver().is_some() && !root_constant(call.receiver(), b"Kernel") {
            return false;
        }
        return call.receiver().is_some()
            || !inside_instance_eval && !redefined.contains(call_name(&call));
    }
    if let Some(begin) = node.as_begin_node() {
        if begin.rescue_clause().is_some() || begin.ensure_clause().is_some() {
            return false;
        }
        return begin.statements().is_some_and(|statements| {
            statements
                .body()
                .iter()
                .any(|statement| flow_expression(&statement, redefined, inside_instance_eval))
        });
    }
    if let Some(condition) = node.as_if_node() {
        let Some(if_branch) = condition.statements() else {
            return false;
        };
        let Some(else_branch) = condition
            .subsequent()
            .and_then(|branch| branch.as_else_node())
            .and_then(|branch| branch.statements())
        else {
            return false;
        };
        return branch_flows(&if_branch, redefined, inside_instance_eval)
            && branch_flows(&else_branch, redefined, inside_instance_eval);
    }
    if let Some(condition) = node.as_unless_node() {
        let Some(if_branch) = condition.statements() else {
            return false;
        };
        let Some(else_branch) = condition
            .else_clause()
            .and_then(|branch| branch.statements())
        else {
            return false;
        };
        return branch_flows(&if_branch, redefined, inside_instance_eval)
            && branch_flows(&else_branch, redefined, inside_instance_eval);
    }
    false
}

fn branch_flows(
    statements: &ruby_prism::StatementsNode<'_>,
    redefined: &HashSet<Vec<u8>>,
    inside_instance_eval: bool,
) -> bool {
    statements
        .body()
        .iter()
        .any(|statement| flow_expression(&statement, redefined, inside_instance_eval))
}
