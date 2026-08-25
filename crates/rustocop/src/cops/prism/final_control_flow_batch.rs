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
        let edit_end = ancestors
            .iter()
            .filter_map(Node::as_or_node)
            .filter(|outer| {
                outer.location().start_offset() == node.location().start_offset()
                    && node.location().end_offset() <= outer.left().location().end_offset()
            })
            .map(|outer| outer.location().end_offset())
            .max()
            .unwrap_or_else(|| node.location().end_offset());
        let message = format!(
            "`{}` will never evaluate because `{}` always returns a truthy value.",
            source_at(source, &rhs.location()),
            truthy
        );
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        cop_context.replace(
            message,
            offense,
            node.location().start_offset()..edit_end,
            replacement,
        );
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

    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
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
                if overlapping_elsif {
                    report_literal_condition(
                        self.name(),
                        &predicate,
                        false,
                        ancestors,
                        source,
                        context,
                    );
                } else {
                    let mut replacement = if falsey_condition_literal(&predicate) {
                        conditional
                            .subsequent()
                            .map(|subsequent| conditional_subsequent_source(&subsequent, source))
                            .unwrap_or_default()
                    } else {
                        let body = conditional
                            .statements()
                            .map(|statements| {
                                conditional_branch_source(
                                    &statements,
                                    conditional.if_keyword_loc(),
                                    source,
                                )
                            })
                            .unwrap_or_default();
                        body
                    };
                    if conditional
                        .if_keyword_loc()
                        .is_some_and(|keyword| keyword.as_slice() == b"elsif")
                    {
                        let indent = " ".repeat(
                            source[..node.location().start_offset()]
                                .rsplit('\n')
                                .next()
                                .map_or(0, str::len),
                        );
                        replacement = format!(
                            "else\n{indent}  {}\n{indent}end",
                            replacement.replace('\n', &format!("\n{indent}  "))
                        );
                    }
                    report_literal_edit(
                        self.name(),
                        &predicate,
                        node.location().start_offset()..node.location().end_offset(),
                        replacement,
                        ancestors,
                        source,
                        context,
                    );
                }
            }
            return;
        }
        if let Some(conditional) = node.as_unless_node() {
            let predicate = conditional.predicate();
            if condition_literal(&predicate) {
                let replacement = if falsey_condition_literal(&predicate) {
                    conditional
                        .statements()
                        .map(|statements| {
                            conditional_branch_source(
                                &statements,
                                Some(conditional.keyword_loc()),
                                source,
                            )
                        })
                        .unwrap_or_default()
                } else {
                    conditional
                        .else_clause()
                        .and_then(|otherwise| otherwise.statements())
                        .map(|statements| conditional_statements_source(&statements, source))
                        .unwrap_or_default()
                };
                report_literal_edit(
                    self.name(),
                    &predicate,
                    node.location().start_offset()..node.location().end_offset(),
                    replacement,
                    ancestors,
                    source,
                    context,
                );
            }
            return;
        }
        if let Some(loop_node) = node.as_while_node() {
            let predicate = loop_node.predicate();
            if condition_literal(&predicate) && source_at(source, &predicate.location()) != "true" {
                if falsey_condition_literal(&predicate) {
                    let loop_source = source_at(source, &node.location());
                    let replacement = if loop_source.trim_start().starts_with("begin") {
                        unwrap_post_loop_body(loop_source)
                    } else {
                        String::new()
                    };
                    report_literal_edit(
                        self.name(),
                        &predicate,
                        node.location().start_offset()..node.location().end_offset(),
                        replacement,
                        ancestors,
                        source,
                        context,
                    );
                } else {
                    report_literal_edit(
                        self.name(),
                        &predicate,
                        predicate.location().start_offset()..predicate.location().end_offset(),
                        "true".to_string(),
                        ancestors,
                        source,
                        context,
                    );
                }
            }
            return;
        }
        if let Some(loop_node) = node.as_until_node() {
            let predicate = loop_node.predicate();
            if condition_literal(&predicate) && source_at(source, &predicate.location()) != "false"
            {
                if falsey_condition_literal(&predicate) {
                    report_literal_edit(
                        self.name(),
                        &predicate,
                        predicate.location().start_offset()..predicate.location().end_offset(),
                        "false".to_string(),
                        ancestors,
                        source,
                        context,
                    );
                } else {
                    let loop_source = source_at(source, &node.location());
                    let replacement = if loop_source.trim_start().starts_with("begin") {
                        unwrap_post_loop_body(loop_source)
                    } else {
                        String::new()
                    };
                    report_literal_edit(
                        self.name(),
                        &predicate,
                        node.location().start_offset()..node.location().end_offset(),
                        replacement,
                        ancestors,
                        source,
                        context,
                    );
                }
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
                if correctable {
                    let mut edit = node.location().start_offset()..node.location().end_offset();
                    let mut replacement = source_at(source, &right.location()).to_string();
                    if truthy_condition_literal(&right) {
                        if let Some(parent) = ancestors.iter().rev().find_map(Node::as_if_node) {
                            if parent.predicate().location().start_offset()
                                == node.location().start_offset()
                                && parent.predicate().location().end_offset()
                                    == node.location().end_offset()
                            {
                                edit = parent.location().start_offset()
                                    ..parent.location().end_offset();
                                replacement = parent
                                    .statements()
                                    .map(|statements| {
                                        conditional_branch_source(
                                            &statements,
                                            parent.if_keyword_loc(),
                                            source,
                                        )
                                    })
                                    .unwrap_or_default();
                            }
                        }
                    }
                    report_literal_edit(
                        self.name(),
                        &left,
                        edit,
                        replacement,
                        ancestors,
                        source,
                        context,
                    );
                } else {
                    report_literal_condition(self.name(), &left, false, ancestors, source, context);
                }
            }
            return;
        }
        if let Some(logical) = node.as_or_node() {
            let left = logical.left();
            if falsey_condition_literal(&left) {
                let right = logical.right();
                let correctable = !void_control_value(&right);
                if correctable {
                    report_literal_edit(
                        self.name(),
                        &left,
                        node.location().start_offset()..node.location().end_offset(),
                        source_at(source, &right.location()).to_string(),
                        ancestors,
                        source,
                        context,
                    );
                } else {
                    report_literal_condition(self.name(), &left, false, ancestors, source, context);
                }
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

fn conditional_branch_source(
    statements: &ruby_prism::StatementsNode<'_>,
    keyword: Option<ruby_prism::Location<'_>>,
    source: &str,
) -> String {
    if keyword.is_none_or(|keyword| keyword.start_offset() > statements.location().start_offset()) {
        source[statements.location().start_offset()..statements.location().end_offset()].to_string()
    } else {
        conditional_statements_source(statements, source)
    }
}

fn conditional_statements_source(
    statements: &ruby_prism::StatementsNode<'_>,
    source: &str,
) -> String {
    let location = statements.location();
    let start_line = source[..location.start_offset()]
        .rfind('\n')
        .map_or(0, |at| at + 1);
    let end_line = source[location.end_offset()..]
        .find('\n')
        .map_or(location.end_offset(), |at| location.end_offset() + at);
    let indentation = location.start_offset() - start_line;
    source[start_line..end_line]
        .lines()
        .map(|line| line.get(indentation..).unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn conditional_subsequent_source(node: &Node<'_>, source: &str) -> String {
    if let Some(otherwise) = node.as_else_node() {
        return otherwise
            .statements()
            .map(|statements| conditional_statements_source(&statements, source))
            .unwrap_or_default();
    }
    let mut replacement = source_at(source, &node.location()).to_string();
    if replacement.starts_with("elsif") {
        replacement.replace_range(..5, "if");
    }
    replacement
}

fn unwrap_post_loop_body(source: &str) -> String {
    let Some(header_end) = source.find('\n') else {
        return String::new();
    };
    let Some(footer) = source.rfind("\nend ") else {
        return String::new();
    };
    source[header_end + 1..footer]
        .lines()
        .map(|line| line.strip_prefix("  ").unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn report_literal_edit(
    cop: &'static str,
    literal: &Node<'_>,
    edit: std::ops::Range<usize>,
    replacement: String,
    ancestors: &[Node<'_>],
    source: &str,
    context: &mut Context,
) {
    let range = literal.location().start_offset()..literal.location().end_offset();
    let message = format!(
        "Literal `{}` appeared as a condition.",
        &source[range.clone()]
    );
    context
        .cop_context(cop, source, ancestors)
        .replace(message, range, edit, replacement);
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
        || node.as_x_string_node().is_some()
        || node.as_interpolated_x_string_node().is_some()
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

struct DuplicateBranchCop;

struct AstDuplicateBranch {
    key: String,
    literal: String,
    offense: std::ops::Range<usize>,
    else_branch: bool,
}

impl Cop for DuplicateBranchCop {
    fn name(&self) -> &'static str {
        "Lint/DuplicateBranch"
    }

    #[allow(clippy::too_many_lines)]
    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let branches = if let Some(if_node) = node.as_if_node() {
            if if_node
                .if_keyword_loc()
                .is_some_and(|keyword| keyword.as_slice() == b"elsif")
                || ancestors.iter().any(|ancestor| {
                    ancestor.as_else_node().is_some_and(|else_node| {
                        else_node.statements().is_some_and(|statements| {
                            statements.location().start_offset() == node.location().start_offset()
                                && statements.location().end_offset()
                                    == node.location().end_offset()
                            })
                    })
                    || ancestor.as_if_node().is_some_and(|parent| {
                        parent
                            .subsequent()
                            .and_then(|branch| branch.as_else_node())
                            .and_then(|else_node| else_node.statements())
                            .is_some_and(|statements| {
                                statements.location().start_offset()
                                    == node.location().start_offset()
                                    && statements.location().end_offset()
                                        == node.location().end_offset()
                            })
                    })
                    || ancestor.as_unless_node().is_some_and(|parent| {
                        parent
                            .else_clause()
                            .and_then(|else_node| else_node.statements())
                            .is_some_and(|statements| {
                                statements.location().start_offset()
                                    == node.location().start_offset()
                                    && statements.location().end_offset()
                                        == node.location().end_offset()
                            })
                    })
                })
            {
                return;
            }
            duplicate_if_branches(&if_node, source)
        } else if let Some(unless_node) = node.as_unless_node() {
            let mut branches = Vec::new();
            push_ast_branch(
                &mut branches,
                unless_node.statements(),
                unless_node.location().start_offset()..unless_node.location().end_offset(),
                false,
                source,
            );
            if let Some(else_node) = unless_node.else_clause() {
                push_ast_branch(
                    &mut branches,
                    else_node.statements(),
                    else_node.else_keyword_loc().start_offset()
                        ..else_node.else_keyword_loc().end_offset(),
                    true,
                    source,
                );
            }
            branches
        } else if let Some(case_node) = node.as_case_node() {
            let mut branches = case_node
                .conditions()
                .iter()
                .filter_map(|condition| condition.as_when_node())
                .filter_map(|branch| {
                    ast_branch(
                        branch.statements(),
                        branch.location().start_offset()..branch.location().end_offset(),
                        false,
                        source,
                    )
                })
                .collect::<Vec<_>>();
            if let Some(else_node) = case_node.else_clause() {
                push_ast_branch(
                    &mut branches,
                    else_node.statements(),
                    else_node.else_keyword_loc().start_offset()
                        ..else_node.else_keyword_loc().end_offset(),
                    true,
                    source,
                );
            }
            branches
        } else if let Some(case_node) = node.as_case_match_node() {
            let mut branches = case_node
                .conditions()
                .iter()
                .filter_map(|condition| condition.as_in_node())
                .filter_map(|branch| {
                    ast_branch(
                        branch.statements(),
                        branch.location().start_offset()..branch.location().end_offset(),
                        false,
                        source,
                    )
                })
                .collect::<Vec<_>>();
            if let Some(else_node) = case_node.else_clause() {
                push_ast_branch(
                    &mut branches,
                    else_node.statements(),
                    else_node.else_keyword_loc().start_offset()
                        ..else_node.else_keyword_loc().end_offset(),
                    true,
                    source,
                );
            }
            branches
        } else if let Some(begin_node) = node.as_begin_node() {
            let Some(rescue_node) = begin_node.rescue_clause() else {
                return;
            };
            let mut branches = Vec::new();
            let mut current = Some(rescue_node);
            while let Some(rescue) = current {
                let end = rescue
                    .statements()
                    .map_or_else(|| rescue.location().end_offset(), |body| body.location().end_offset());
                push_ast_branch(
                    &mut branches,
                    rescue.statements(),
                    rescue.keyword_loc().start_offset()..end,
                    false,
                    source,
                );
                current = rescue.subsequent();
            }
            if let Some(else_node) = begin_node.else_clause() {
                push_ast_branch(
                    &mut branches,
                    else_node.statements(),
                    else_node.else_keyword_loc().start_offset()
                        ..else_node.else_keyword_loc().end_offset(),
                    true,
                    source,
                );
            }
            branches
        } else {
            return;
        };

        if branches.len() < 2 {
            return;
        }
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        let ignore_literals = cop_context.config_bool("IgnoreLiteralBranches", false);
        let ignore_constants = cop_context.config_bool("IgnoreConstantBranches", false);
        let ignore_duplicate_else = cop_context.config_bool("IgnoreDuplicateElseBranch", false);
        let branch_count = branches.len();
        let mut seen = HashSet::new();
        for (index, branch) in branches.into_iter().enumerate() {
            let duplicate = !seen.insert(branch.key);
            if !duplicate
                || ignore_literals && duplicate_literal(&branch.literal, ignore_constants)
                || ignore_constants && duplicate_constant_branch(&branch.literal)
                || ignore_duplicate_else
                    && branch.else_branch
                    && branch_count > 2
                    && index + 1 == branch_count
            {
                continue;
            }
            cop_context.report("Duplicate branch body detected.", branch.offense);
        }
    }
}

fn duplicate_if_branches(node: &ruby_prism::IfNode<'_>, source: &str) -> Vec<AstDuplicateBranch> {
    let mut branches = Vec::new();
    push_ast_branch(
        &mut branches,
        node.statements(),
        node.location().start_offset()..node.location().end_offset(),
        false,
        source,
    );
    let mut subsequent = node.subsequent();
    while let Some(branch) = subsequent {
        if let Some(elsif) = branch.as_if_node() {
            let start = elsif
                .if_keyword_loc()
                .map_or_else(|| elsif.location().start_offset(), |keyword| keyword.start_offset());
            let end = conditional_branch_end(&elsif, source);
            push_ast_branch(&mut branches, elsif.statements(), start..end, false, source);
            subsequent = elsif.subsequent();
        } else if let Some(else_node) = branch.as_else_node() {
            let only = else_node
                .statements()
                .and_then(|statements| {
                    let mut nodes = statements.body().iter().collect::<Vec<_>>();
                    (nodes.len() == 1).then(|| nodes.pop().expect("one statement"))
                });
            let elsif = only.as_ref().and_then(|child| child.as_if_node()).filter(|child| {
                child
                    .if_keyword_loc()
                    .is_some_and(|keyword| keyword.as_slice() == b"elsif")
            });
            if let Some(elsif) = elsif {
                let start = elsif
                    .if_keyword_loc()
                    .map_or_else(|| elsif.location().start_offset(), |keyword| keyword.start_offset());
                let end = conditional_branch_end(&elsif, source);
                push_ast_branch(&mut branches, elsif.statements(), start..end, false, source);
                subsequent = elsif.subsequent();
            } else {
                let offense = if node.if_keyword_loc().is_none() {
                    only.map_or_else(
                        || {
                            else_node.else_keyword_loc().start_offset()
                                ..else_node.else_keyword_loc().end_offset()
                        },
                        |body| body.location().start_offset()..body.location().end_offset(),
                    )
                } else {
                    else_node.else_keyword_loc().start_offset()
                        ..else_node.else_keyword_loc().end_offset()
                };
                push_ast_branch(
                    &mut branches,
                    else_node.statements(),
                    offense,
                    true,
                    source,
                );
                break;
            }
        } else {
            break;
        }
    }
    branches
}

fn conditional_branch_end(node: &ruby_prism::IfNode<'_>, source: &str) -> usize {
    let mut end = node
        .end_keyword_loc()
        .map_or_else(|| node.location().end_offset(), |keyword| keyword.start_offset());
    while end > node.location().start_offset()
        && source.as_bytes().get(end - 1).is_some_and(u8::is_ascii_whitespace)
    {
        end -= 1;
    }
    end
}

fn push_ast_branch(
    branches: &mut Vec<AstDuplicateBranch>,
    statements: Option<ruby_prism::StatementsNode<'_>>,
    offense: std::ops::Range<usize>,
    else_branch: bool,
    source: &str,
) {
    if let Some(branch) = ast_branch(statements, offense, else_branch, source) {
        branches.push(branch);
    }
}

fn ast_branch(
    statements: Option<ruby_prism::StatementsNode<'_>>,
    offense: std::ops::Range<usize>,
    else_branch: bool,
    source: &str,
) -> Option<AstDuplicateBranch> {
    let statements = statements?;
    let nodes = statements.body().iter().collect::<Vec<_>>();
    if nodes.is_empty() {
        return None;
    }
    let literal = if nodes.len() == 1 {
        source_at(source, &nodes[0].location()).to_string()
    } else {
        source_at(source, &statements.location()).to_string()
    };
    let key = nodes
        .iter()
        .map(|node| duplicate_branch_node_source(node, source))
        .collect::<Vec<_>>()
        .join("\n");
    Some(AstDuplicateBranch {
        key,
        literal,
        offense,
        else_branch,
    })
}

fn duplicate_branch_node_source(node: &Node<'_>, source: &str) -> String {
    let raw = source_at(source, &node.location());
    let mut normalized = raw
        .lines()
        .map(duplicate_branch_line)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let mut search_from = node.location().end_offset();
    for marker in raw.split("<<").skip(1).filter_map(|tail| {
        let tail = tail.trim_start_matches(['-', '~']);
        let marker = tail
            .trim_start()
            .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
            .next()?;
        (!marker.is_empty()).then_some(marker)
    }) {
        let Some(relative_end) = source[search_from..]
            .lines()
            .scan(search_from, |offset, line| {
                let start = *offset;
                *offset += line.len() + 1;
                Some((start, line))
            })
            .find_map(|(offset, line)| (line.trim() == marker).then_some(offset + line.len()))
        else {
            continue;
        };
        normalized.push('\n');
        normalized.push_str(&source[search_from..relative_end]);
        search_from = relative_end;
    }
    normalized
}

fn duplicate_branch_line(line: &str) -> String {
    let line = line.trim();
    let mut quote = None;
    let mut escaped = false;
    for (index, byte) in line.bytes().enumerate() {
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' && quote.is_some() {
            escaped = true;
            continue;
        }
        if matches!(byte, b'\'' | b'"' | b'`') {
            if quote == Some(byte) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(byte);
            }
            continue;
        }
        if byte == b'#'
            && quote.is_none()
            && (index == 0 || line.as_bytes()[index - 1].is_ascii_whitespace())
            && line.as_bytes().get(index + 1) != Some(&b'{')
        {
            return line[..index].trim_end().to_string();
        }
    }
    line.to_string()
}

fn duplicate_literal(source: &str, ignore_constants: bool) -> bool {
    let source = source.trim();
    if source.is_empty() || source.contains("#{") || source.starts_with('`') {
        return false;
    }
    if matches!(source, "true" | "false" | "nil" | "[]" | "{}") {
        return true;
    }
    if source.starts_with('/') && source.rfind('/').is_some_and(|at| at > 0)
        || source.starts_with(':') && !source.starts_with(":\"") && !source.starts_with(":'")
        || (source.starts_with('"') && source.ends_with('"'))
        || (source.starts_with('\'') && source.ends_with('\''))
    {
        return true;
    }
    let number = source.trim_end_matches(['r', 'i']);
    if number.parse::<f64>().is_ok() {
        return true;
    }
    if let Some((left, right)) = source.split_once("...").or_else(|| source.split_once("..")) {
        return left.trim().parse::<f64>().is_ok() && right.trim().parse::<f64>().is_ok();
    }
    if source.starts_with('[') && source.ends_with(']') {
        return source[1..source.len() - 1]
            .split(',')
            .all(|item| duplicate_literal_atom(item.trim(), ignore_constants));
    }
    if source.starts_with('{') && source.ends_with('}') {
        return source[1..source.len() - 1].split(',').all(|pair| {
            pair.split_once(':')
                .is_some_and(|(_, value)| duplicate_literal_atom(value.trim(), ignore_constants))
        });
    }
    ignore_constants && constant_literal(source)
}

fn duplicate_literal_atom(source: &str, ignore_constants: bool) -> bool {
    duplicate_literal(source, ignore_constants) || ignore_constants && constant_literal(source)
}

fn constant_literal(source: &str) -> bool {
    !source.is_empty()
        && source.split("::").all(|part| {
            part.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        })
}

fn duplicate_constant_branch(source: &str) -> bool {
    let source = source.trim();
    if constant_literal(source) {
        return true;
    }
    if source.starts_with('[') && source.ends_with(']') {
        return source[1..source.len() - 1]
            .split(',')
            .all(|item| duplicate_constant_branch(item.trim()));
    }
    if source.starts_with('{') && source.ends_with('}') {
        return source[1..source.len() - 1].split(',').all(|pair| {
            pair.split_once(':')
                .is_some_and(|(_, value)| duplicate_constant_branch(value.trim()))
        });
    }
    false
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
        ancestors: &[Node<'pr>],
        _source: &str,
        context: &mut Context,
    ) {
        let Some(statements) = node.as_statements_node() else {
            return;
        };
        let body = statements.body().iter().collect::<Vec<_>>();
        let mut redefined = redefined_flow_methods(&body);
        if let Some(definition) = ancestors.iter().rev().find_map(Node::as_def_node) {
            let name = definition.name().as_slice();
            if matches!(
                name,
                b"raise" | b"fail" | b"throw" | b"exit" | b"exit!" | b"abort"
            ) {
                redefined.insert(name.to_vec());
            }
        }
        let inside_instance_eval = ancestors.iter().any(|ancestor| {
            ancestor
                .as_call_node()
                .is_some_and(|call| call_name(&call) == b"instance_eval")
        });
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

fn redefined_flow_methods(body: &[Node<'_>]) -> HashSet<Vec<u8>> {
    body.iter()
        .filter_map(|node| {
            let definition = node.as_def_node()?;
            let name = definition.name().as_slice();
            matches!(
                name,
                b"raise" | b"fail" | b"throw" | b"exit" | b"exit!" | b"abort"
            )
            .then(|| name.to_vec())
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
        let receiver_is_kernel = root_constant(call.receiver(), b"Kernel");
        if !flow || call.receiver().is_some() && !receiver_is_kernel {
            return false;
        }
        return receiver_is_kernel
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
        let Some(subsequent) = condition.subsequent() else {
            return false;
        };
        return branch_flows(&if_branch, redefined, inside_instance_eval)
            && flow_expression(&subsequent, redefined, inside_instance_eval);
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
    if let Some(branch) = node.as_else_node() {
        return branch
            .statements()
            .is_some_and(|statements| branch_flows(&statements, redefined, inside_instance_eval));
    }
    if let Some(case_node) = node.as_case_node() {
        let Some(else_branch) = case_node.else_clause() else {
            return false;
        };
        return case_node.conditions().iter().all(|condition| {
            condition.as_when_node().is_some_and(|branch| {
                branch.statements().is_some_and(|statements| {
                    branch_flows(&statements, redefined, inside_instance_eval)
                })
            })
        }) && flow_expression(&else_branch.as_node(), redefined, inside_instance_eval);
    }
    if let Some(case_node) = node.as_case_match_node() {
        let Some(else_branch) = case_node.else_clause() else {
            return false;
        };
        return case_node.conditions().iter().all(|condition| {
            condition.as_in_node().is_some_and(|branch| {
                branch.statements().is_some_and(|statements| {
                    branch_flows(&statements, redefined, inside_instance_eval)
                })
            })
        }) && flow_expression(&else_branch.as_node(), redefined, inside_instance_eval);
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
