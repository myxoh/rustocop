use super::*;

define_cops! {
    AmbiguousOperator => "Lint/AmbiguousOperator" => call(ambiguous_operator),
    AmbiguousOperatorPrecedence => "Lint/AmbiguousOperatorPrecedence" => any_node(ambiguous_operator_precedence),
    ParenthesesAsGroupedExpression => "Lint/ParenthesesAsGroupedExpression" => call(parentheses_as_grouped_expression),
}

fn parentheses_as_grouped_expression(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if argument_count(node) != 1
        || matches!(
            call_name(node),
            b"+" | b"-" | b"*" | b"/" | b"%" | b"**" | b"&" | b"|" | b"^" | b"<<" | b">>"
        )
        || call_name(node).ends_with(b"=")
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
    if context.ancestors().iter().rev().any(|ancestor| {
        ancestor.as_call_node().is_some()
            || ancestor.as_and_node().is_some()
            || ancestor.as_or_node().is_some()
            || ancestor.as_if_node().is_some()
            || ancestor.as_range_node().is_some()
            || ancestor.as_assoc_node().is_some()
    }) {
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

fn ambiguous_operator_precedence(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let Some(current) = precedence(node) else {
        return;
    };
    let mut parent = None;
    for ancestor in context.ancestors().iter().rev() {
        if ancestor.as_parentheses_node().is_some() {
            return;
        }
        if let Some(precedence) = precedence(ancestor) {
            parent = Some(precedence);
            break;
        }
    }
    let Some(parent) = parent else { return };
    if current == parent {
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

fn ambiguous_operator(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.opening_loc().is_some() || matches!(call_name(node), b"+" | b"-" | b"*" | b"**" | b"&")
    {
        return;
    }
    let Some(argument) = first_argument(node).or_else(|| {
        node.block()
            .filter(|block| block.as_block_argument_node().is_some())
    }) else {
        return;
    };
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
