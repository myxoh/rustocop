use super::*;

mod registry;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    let mut cops = vec![
        Box::new(UnreachableLoop) as Box<dyn Cop>,
        super::catalog_cop::custom("Lint/EmptyConditionalBody", empty_conditional),
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

fn empty_conditional(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if ["if ", "unless ", "while ", "until "]
            .iter()
            .any(|keyword| window[0].1.trim_start().starts_with(keyword))
            && window[1].1.trim() == "end"
        {
            context.report(
                "Avoid empty conditional bodies.",
                window[0].0..window[1].0 + window[1].1.len(),
            );
        }
    }
}

fn unreachable_code(context: &mut CopContext<'_, '_>) {
    let redefines_raise = context.source().lines().any(|line| {
        let line = line.trim_start();
        line.starts_with("def raise")
            || line.starts_with("def self.raise")
            || line.starts_with("def fail")
            || line.starts_with("def self.fail")
    });
    let dynamic_receiver = context.source().contains("instance_eval");
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        let terminator = window[0].1.trim();
        if matches!(terminator, "raise" | "fail") && (redefines_raise || dynamic_receiver) {
            continue;
        }
        if matches!(terminator, "return" | "break" | "next" | "raise" | "fail")
            && !window[1].1.trim().is_empty()
            && !matches!(window[1].1.trim(), "end" | "else" | "ensure" | "rescue")
            && window[0].1.len() - window[0].1.trim_start().len()
                == window[1].1.len() - window[1].1.trim_start().len()
        {
            context.report(
                "Unreachable code detected.",
                window[1].0..window[1].0 + window[1].1.len(),
            );
        }
    }
}

fn literal_condition(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        for condition in [
            "if true",
            "if false",
            "if nil",
            "unless true",
            "unless false",
        ] {
            if let Some(at) = line.find(condition) {
                context.report(
                    "Literal used as a condition.",
                    offset + at..offset + at + condition.len(),
                );
            }
        }
    }
}
