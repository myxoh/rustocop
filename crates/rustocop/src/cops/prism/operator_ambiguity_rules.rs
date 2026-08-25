use super::*;

define_cops! {
    AmbiguousOperator => "Lint/AmbiguousOperator" => any_node(ambiguous_operator),
    AmbiguousOperatorPrecedence => "Lint/AmbiguousOperatorPrecedence" => any_node(ambiguous_operator_precedence),
    ParenthesesAsGroupedExpression => "Lint/ParenthesesAsGroupedExpression" => call(parentheses_as_grouped_expression),
}

fn parentheses_as_grouped_expression(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if argument_count(node) != 1
        || grouped_expression_operator_or_setter(call_name(node))
    {
        return;
    }
    let (Some(selector), Some(argument)) = (node.message_loc(), only_argument(node)) else {
        return;
    };
    let (opening, closing) =
        if let (Some(opening), Some(closing)) = (node.opening_loc(), node.closing_loc()) {
            (opening, closing)
        } else if let Some(parentheses) = argument.as_parentheses_node() {
            (parentheses.opening_loc(), parentheses.closing_loc())
        } else {
            return;
        };
    if selector.end_offset() >= opening.start_offset() {
        return;
    }
    let gap = &context.source()[selector.end_offset()..opening.start_offset()];
    if gap.is_empty() || !gap.chars().all(char::is_whitespace) {
        return;
    }
    if grouped_expression_chained_call(node, context)
        || context.ancestors().iter().rev().any(|ancestor| {
            ancestor.location().start_offset() == node.location().start_offset()
                && (ancestor.as_and_node().is_some()
                    || ancestor.as_or_node().is_some()
                    || ancestor.as_if_node().is_some()
                    || ancestor.as_range_node().is_some()
                    || ancestor.as_assoc_node().is_some())
        })
    {
        return;
    }
    context.remove(
        format!(
            "`{}` interpreted as grouped expression.",
            &context.source()[opening.start_offset()..closing.end_offset()]
        ),
        selector.end_offset()..opening.start_offset(),
        selector.end_offset()..opening.start_offset(),
    );
}

fn grouped_expression_operator_or_setter(name: &[u8]) -> bool {
    matches!(
        name,
        b"+" | b"-"
            | b"*"
            | b"/"
            | b"%"
            | b"**"
            | b"=="
            | b"!="
            | b"==="
            | b"=~"
            | b"!~"
            | b"<"
            | b">"
            | b"<="
            | b">="
            | b"<=>"
            | b"<<"
            | b">>"
            | b"&"
            | b"|"
            | b"^"
            | b"[]"
            | b"[]="
            | b"!"
            | b"~"
            | b"+@"
            | b"-@"
    ) || name.ends_with(b"=")
}

fn grouped_expression_chained_call(
    node: &CallNode<'_>,
    context: &CopContext<'_, '_>,
) -> bool {
    let location = node.location();
    context.ancestors().iter().rev().any(|ancestor| {
        ancestor.as_call_node().is_some_and(|call| {
            call.receiver().is_some_and(|receiver| {
                receiver.location().start_offset() == location.start_offset()
                    && receiver.location().end_offset() == location.end_offset()
            })
        })
    })
}

fn ambiguous_operator_precedence(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let Some(current) = precedence(node) else {
        return;
    };
    let Some(parent_node) = context.parent() else {
        return;
    };
    if parent_node.as_parentheses_node().is_some() {
        return;
    }
    let parent = precedence(parent_node).or_else(|| {
        if current != 7 {
            return None;
        }
        if parent_node.as_and_node().is_some_and(|node| node.operator_loc().as_slice() == b"and") {
            Some(9)
        } else if parent_node.as_or_node().is_some_and(|node| node.operator_loc().as_slice() == b"or") {
            Some(10)
        } else {
            None
        }
    });
    let Some(parent) = parent else { return };
    if current >= parent {
        return;
    }
    let location = node.location();
    context.replace_many(
        "Wrap expressions with varying precedence with parentheses to avoid ambiguity.",
        &location,
        vec![
            (
                location.start_offset()..location.start_offset(),
                "(".to_string(),
            ),
            (
                location.end_offset()..location.end_offset(),
                ")".to_string(),
            ),
        ],
    );
}

fn precedence(node: &Node<'_>) -> Option<u8> {
    if let Some(and) = node.as_and_node() {
        return (and.operator_loc().as_slice() == b"&&").then_some(7);
    }
    if let Some(or) = node.as_or_node() {
        return (or.operator_loc().as_slice() == b"||").then_some(8);
    }
    let call = node.as_call_node()?;
    if argument_count(&call) != 1 {
        return None;
    }
    match call_name(&call) {
        b"**" => Some(1),
        b"*" | b"/" | b"%" => Some(2),
        b"+" | b"-" => Some(3),
        b"<<" | b">>" => Some(4),
        b"&" => Some(5),
        b"|" | b"^" => Some(6),
        _ => None,
    }
}

fn ambiguous_operator(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(call) = node.as_call_node() {
        ambiguous_call_operator(&call, context);
        return;
    }
    let Some(super_node) = node.as_super_node() else {
        return;
    };
    if super_node.lparen_loc().is_some() {
        return;
    }
    let Some(argument) = super_node
        .arguments()
        .and_then(|arguments| arguments.arguments().first())
    else {
        return;
    };
    let Some(splat) = argument.as_splat_node() else {
        return;
    };
    let operator = splat.operator_loc();
    let end = super_node.location().end_offset();
    context.replace_many(
        "Ambiguous splat operator. Parenthesize the method arguments if it's surely a splat operator, or add a whitespace to the right of the `*` if it should be a multiplication.",
        &operator,
        vec![
            (super_node.keyword_loc().end_offset()..operator.start_offset(), "(".to_string()),
            (end..end, ")".to_string()),
        ],
    );
}

fn ambiguous_call_operator(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.opening_loc().is_some() || grouped_expression_operator_or_setter(call_name(node))
    {
        return;
    }
    let Some(argument) = first_argument(node).or_else(|| {
        node.block()
            .filter(|block| block.as_block_argument_node().is_some())
    }) else {
        return;
    };
    if argument.as_lambda_node().is_some() {
        return;
    }
    let argument_location = argument.location();
    let argument_source = context.source_file().node(&argument);
    let (operator, message) = if argument_source.starts_with("**")
        && !argument_source[2..].starts_with(char::is_whitespace)
    {
        ("**", "Ambiguous keyword splat operator. Parenthesize the method arguments if it's surely a keyword splat operator, or add a whitespace to the right of the `**` if it should be an exponent.")
    } else if argument_source.starts_with('+')
        && !argument_source[1..].starts_with(char::is_whitespace)
    {
        ("+", "Ambiguous positive number operator. Parenthesize the method arguments if it's surely a positive number operator, or add a whitespace to the right of the `+` if it should be an addition.")
    } else if argument_source.starts_with('-')
        && !argument_source.starts_with("->")
        && !argument_source[1..].starts_with(char::is_whitespace)
    {
        ("-", "Ambiguous negative number operator. Parenthesize the method arguments if it's surely a negative number operator, or add a whitespace to the right of the `-` if it should be a subtraction.")
    } else if argument_source.starts_with('*')
        && !argument_source[1..].starts_with(char::is_whitespace)
    {
        ("*", "Ambiguous splat operator. Parenthesize the method arguments if it's surely a splat operator, or add a whitespace to the right of the `*` if it should be a multiplication.")
    } else if argument_source.starts_with('&')
        && !argument_source[1..].starts_with(char::is_whitespace)
    {
        ("&", "Ambiguous block operator. Parenthesize the method arguments if it's surely a block operator, or add a whitespace to the right of the `&` if it should be a binary AND.")
    } else {
        return;
    };
    let Some(selector) = node.message_loc() else {
        return;
    };
    if !context.source()[selector.end_offset()..argument_location.start_offset()]
        .chars()
        .all(char::is_whitespace)
    {
        return;
    }
    context.replace_many(
        message,
        argument_location.start_offset()..argument_location.start_offset() + operator.len(),
        vec![
            (
                selector.end_offset()..argument_location.start_offset(),
                "(".to_string(),
            ),
            (
                node.location().end_offset()..node.location().end_offset(),
                ")".to_string(),
            ),
        ],
    );
}
