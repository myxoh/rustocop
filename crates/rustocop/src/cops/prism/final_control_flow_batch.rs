use super::*;
use std::collections::HashSet;

mod registry;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    let mut cops = vec![
        Box::new(UnreachableLoop) as Box<dyn Cop>,
        Box::new(EmptyConditionalBody) as Box<dyn Cop>,
    ];
    cops.extend(registry::cops());
    cops
}

struct UnreachableLoop;

impl Cop for UnreachableLoop {
    fn name(&self) -> &'static str {
        "Lint/UnreachableLoop"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let statements = if let Some(call) = node.as_call_node() {
            if call.name().as_slice() != b"each" {
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
        let Some((index, _)) = statements
            .iter()
            .enumerate()
            .find(|(_, statement)| terminating_loop_statement(statement))
        else {
            return;
        };
        if statements[..index]
            .iter()
            .any(|statement| contains_continue_keyword(source_at(source, &statement.location())))
        {
            return;
        }
        context.report(self.name(), "This loop will have at most one iteration.", node.location());
    }
}

fn terminating_loop_statement(node: &Node<'_>) -> bool {
    if node.as_return_node().is_some() || node.as_break_node().is_some() {
        return true;
    }
    if let Some(call) = node.as_call_node() {
        return matches!(call_name(&call), b"raise" | b"fail" | b"throw" | b"exit" | b"exit!" | b"abort")
            && (call.receiver().is_none() || root_constant(call.receiver(), b"Kernel"));
    }
    if let Some(begin) = node.as_begin_node() {
        if begin.rescue_clause().is_some() || begin.ensure_clause().is_some() {
            return false;
        }
        return begin.statements().is_some_and(|statements| {
            let statements = statements.body().iter().collect::<Vec<_>>();
            statements.iter().enumerate().find(|(_, statement)| terminating_loop_statement(statement)).is_some_and(|(index, _)| {
                !statements[..index].iter().any(|statement| statement.as_next_node().is_some() || statement.as_redo_node().is_some())
            })
        });
    }
    if let Some(condition) = node.as_if_node() {
        let Some(if_branch) = only_statement(condition.statements()) else { return false };
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
        let Some(if_branch) = only_statement(condition.statements()) else { return false };
        let Some(else_branch) = condition
            .else_clause()
            .and_then(|branch| only_statement(branch.statements()))
        else {
            return false;
        };
        return terminating_loop_statement(&if_branch) && terminating_loop_statement(&else_branch);
    }
    false
}

fn contains_continue_keyword(source: &str) -> bool {
    source
        .split(|character: char| !character.is_alphanumeric() && character != '_')
        .any(|part| matches!(part, "next" | "redo"))
}

fn identical_branches(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(5) {
        if window[0].1.trim_start().starts_with("if ")
            && window[2].1.trim() == "else"
            && window[4].1.trim() == "end"
            && window[1].1.trim() == window[3].1.trim()
        {
            context.report("Duplicate branch body detected.", window[1].0..window[3].0 + window[3].1.len());
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
        "if",
        context,
    );
}

#[allow(clippy::too_many_arguments)]
fn register_empty_conditional(
    location: ruby_prism::Location<'_>,
    predicate: Node<'_>,
    keyword: &str,
    boundary: usize,
    else_keyword: Option<ruby_prism::Location<'_>>,
    inverse_keyword: &str,
    context: &mut CopContext<'_, '_>,
) {
    let file = context.source_file();
    if file.same_line(
        location.start_offset(),
        location.end_offset().saturating_sub(1),
    ) {
        return;
    }
    if context.config_bool("AllowComments", true)
        && context.source()[location.start_offset()..location.end_offset()]
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
    let suffix = &context.source()[else_keyword.end_offset()..location.end_offset()];
    if suffix.trim() == "end" {
        context.report(message, offense);
        return;
    }
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
            matches!(name, "raise" | "fail" | "throw" | "exit" | "exit!" | "abort")
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
        let Some(else_branch) = condition.else_clause().and_then(|branch| branch.statements()) else {
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

fn literal_condition(context: &mut CopContext<'_, '_>) {
    const LITERALS: &[&str] = &[
        "[1, 2, [3, 4]]", ":\"#{a}\"", "[1]", "2.0", "false", "nil", ":sym", "123",
        "42", "1", "{}",
    ];
    for (offset, line) in context.source_file().lines() {
        let mut covered = Vec::<std::ops::Range<usize>>::new();
        for literal in LITERALS {
            for (at, _) in line.match_indices(literal) {
                let range = at..at + literal.len();
                if covered.iter().any(|used| used.start <= at && range.end <= used.end) {
                    continue;
                }
                let before = line[..at].trim_end();
                let after = line[range.end..].trim_start();
                let keyword = ["if", "elsif", "unless", "while", "until", "case", "when"]
                    .iter()
                    .any(|word| before == *word || before.ends_with(&format!(" {word}")));
                let unary = before.ends_with('!')
                    || before.ends_with("not(")
                    || before.ends_with("not");
                let left_operand = after.starts_with('?')
                    || after.starts_with("&&")
                    || after.starts_with("||");
                if !keyword && !unary && !left_operand {
                    continue;
                }
                let message = format!("Literal `{literal}` appeared as a condition.");
                if before == "elsif" || before.ends_with(" elsif") {
                    context.report(message, offset + range.start..offset + range.end);
                } else {
                    context.replace(
                        message,
                        offset + range.start..offset + range.end,
                        offset + range.start..offset + range.end,
                        *literal,
                    );
                }
                covered.push(range);
            }
        }
    }
}
