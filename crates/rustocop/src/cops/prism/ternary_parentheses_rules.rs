use super::*;

define_cops! {
    TernaryParentheses => "Style/TernaryParentheses" => compatibility_prism_node(as_if_node, ternary_parentheses),
}

fn ternary_parentheses(node: &ruby_prism::IfNode<'_>, context: &mut CopContext<'_, '_>) {
    if !is_ternary(node) {
        return;
    }
    let condition = node.predicate();
    let parenthesized = condition.as_parentheses_node();
    if parenthesized.as_ref().is_some_and(|parentheses| {
        context
            .source_file()
            .at(&parentheses.location())
            .lines()
            .last()
            == Some(")")
    }) {
        return;
    }
    let inner = parenthesized
        .as_ref()
        .and_then(parenthesized_expressions)
        .unwrap_or_default();
    if inner.len() == 1 && inner[0].as_match_predicate_node().is_some() {
        return;
    }

    let safe_assignment = parenthesized.is_some() && inner.len() == 1 && assignment_node(&inner[0]);
    let allow_safe_assignment = context.config_bool("AllowSafeAssignment", true);
    let style = context
        .policy()
        .enforced_style("require_no_parentheses")
        .to_string();
    let complex = if parenthesized.is_some() {
        inner.iter().any(complex_condition)
    } else {
        complex_condition(&condition)
    };
    let offense = if safe_assignment {
        !allow_safe_assignment
    } else {
        match style.as_str() {
            "require_parentheses" => parenthesized.is_none(),
            "require_parentheses_when_complex" => complex == parenthesized.is_none(),
            _ => parenthesized.is_some(),
        }
    };
    if !offense {
        return;
    }

    let message = match style.as_str() {
        "require_parentheses" => "Use parentheses for ternary conditions.",
        "require_parentheses_when_complex" if parenthesized.is_some() => {
            "Only use parentheses for ternary expressions with complex conditions."
        }
        "require_parentheses_when_complex" => {
            "Use parentheses for ternary expressions with complex conditions."
        }
        _ => "Omit parentheses for ternary conditions.",
    };
    let offense_location = node.location();

    if safe_assignment || parenthesized.as_ref().is_some_and(|parentheses| {
        parenthesized_expressions(parentheses)
            .is_some_and(|expressions| expressions.iter().any(below_ternary_precedence))
    }) {
        context.report(message, &offense_location);
        return;
    }

    if let Some(parentheses) = parenthesized {
        let replacement = unparenthesized_condition(&parentheses, &inner, node, context);
        context.replace(message, &offense_location, parentheses.location(), replacement);
    } else {
        let source = context.source_file().at(&condition.location());
        context.replace(
            message,
            &offense_location,
            condition.location(),
            format!("({source})"),
        );
    }
}

fn is_ternary(node: &ruby_prism::IfNode<'_>) -> bool {
    node.if_keyword_loc().is_none()
        && node.then_keyword_loc().is_some()
        && node.end_keyword_loc().is_none()
}

fn parenthesized_expressions<'pr>(
    parentheses: &ruby_prism::ParenthesesNode<'pr>,
) -> Option<Vec<Node<'pr>>> {
    let statements = parentheses.body()?.as_statements_node()?;
    Some(statements.body().iter().collect())
}

fn assignment_node(node: &Node<'_>) -> bool {
    node.as_multi_write_node().is_some()
        || node.as_local_variable_write_node().is_some()
        || node.as_instance_variable_write_node().is_some()
        || node.as_class_variable_write_node().is_some()
        || node.as_global_variable_write_node().is_some()
        || node.as_constant_write_node().is_some()
        || node.as_constant_path_write_node().is_some()
        || node.as_call_node().is_some_and(|call| setter_name(call_name(&call)))
}

fn setter_name(name: &[u8]) -> bool {
    name.ends_with(b"=")
        && !matches!(name, b"==" | b"!=" | b"<=" | b">=" | b"===" | b"=~" | b"!~" | b"<=>")
}

fn complex_condition(node: &Node<'_>) -> bool {
    if let Some(parentheses) = node.as_parentheses_node() {
        return parenthesized_expressions(&parentheses)
            .is_some_and(|expressions| expressions.iter().any(complex_condition));
    }
    if node.as_local_variable_read_node().is_some()
        || node.as_instance_variable_read_node().is_some()
        || node.as_class_variable_read_node().is_some()
        || node.as_global_variable_read_node().is_some()
        || node.as_back_reference_read_node().is_some()
        || node.as_numbered_reference_read_node().is_some()
        || node.as_constant_read_node().is_some()
        || node.as_constant_path_node().is_some()
        || node.as_defined_node().is_some()
        || node.as_yield_node().is_some()
    {
        return false;
    }
    if let Some(call) = node.as_call_node() {
        return operator_name(call_name(&call)) && call_name(&call) != b"[]";
    }
    true
}

fn operator_name(name: &[u8]) -> bool {
    matches!(
        name,
        b"+" | b"-"
            | b"*"
            | b"/"
            | b"%"
            | b"**"
            | b"&"
            | b"|"
            | b"^"
            | b"<<"
            | b">>"
            | b"=="
            | b"==="
            | b"!="
            | b"<"
            | b">"
            | b"<="
            | b">="
            | b"<=>"
            | b"=~"
            | b"!~"
            | b"!"
            | b"~"
            | b"+@"
            | b"-@"
    )
}

fn below_ternary_precedence(node: &Node<'_>) -> bool {
    if let Some(or_node) = node.as_or_node() {
        return or_node.operator_loc().as_slice() == b"or";
    }
    if let Some(and_node) = node.as_and_node() {
        return and_node.operator_loc().as_slice() == b"and";
    }
    node.as_call_node().is_some_and(|call| {
        call_name(&call) == b"!"
            && call
                .message_loc()
                .is_some_and(|location| location.as_slice() == b"not")
    })
}

fn unparenthesized_condition(
    parentheses: &ruby_prism::ParenthesesNode<'_>,
    expressions: &[Node<'_>],
    ternary: &ruby_prism::IfNode<'_>,
    context: &CopContext<'_, '_>,
) -> String {
    let opening = parentheses.opening_loc();
    let closing = parentheses.closing_loc();
    let container = opening.end_offset()..closing.start_offset();
    let mut edits = Vec::new();
    if let Some(expression) = expressions.last() {
        if let Some(defined) = expression.as_defined_node() {
            if defined.lparen_loc().is_none() {
                let value = defined.value();
                edits.push(SourceEdit::replace(
                    defined.keyword_loc().end_offset()..value.location().start_offset(),
                    "(",
                ));
                edits.push(SourceEdit::replace(
                    value.location().end_offset()..value.location().end_offset(),
                    ")",
                ));
            }
        } else if let Some(call) = expression.as_call_node() {
            let method_name = call_name(&call);
            let needs_argument_parentheses = call.call_operator_loc().is_some()
                || method_name.first().is_some_and(u8::is_ascii_alphabetic);
            if call.opening_loc().is_none() && needs_argument_parentheses {
                if let Some(arguments) = call.arguments() {
                    if let (Some(first), Some(last), Some(message)) = (
                        arguments.arguments().first(),
                        arguments.arguments().last(),
                        call.message_loc(),
                    ) {
                        edits.push(SourceEdit::replace(
                            message.end_offset()..first.location().start_offset(),
                            "(",
                        ));
                        edits.push(SourceEdit::replace(
                            last.location().end_offset()..last.location().end_offset(),
                            ")",
                        ));
                    }
                }
            }
        }
    }
    let mut rendered = context
        .source_file()
        .rewrite(container.clone(), edits)
        .unwrap_or_else(|| context.source()[container].to_string());
    if ternary
        .then_keyword_loc()
        .is_some_and(|question| question.start_offset() == closing.end_offset())
    {
        rendered.push(' ');
    }
    rendered
}
