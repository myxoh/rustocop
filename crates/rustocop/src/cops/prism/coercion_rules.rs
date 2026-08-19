use super::*;

define_cops! {
    RedundantStringCoercion => "Lint/RedundantStringCoercion" => call(redundant_string_coercion),
}

fn redundant_string_coercion(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) != b"to_s" || argument_count(node) != 0 {
        return;
    }
    let interpolation = interpolation_expression(node, context);
    let output_method = enclosing_output_method(node, context);
    let usage = if interpolation {
        "interpolation"
    } else if let Some(method) = output_method {
        method
    } else {
        return;
    };
    let Some(selector) = node.message_loc() else {
        return;
    };
    let usage = usage_label(usage);
    if let Some(receiver) = node.receiver() {
        context.remove(
            format!("Redundant use of `Object#to_s` in {usage}."),
            &selector,
            receiver.location().end_offset()..node.location().end_offset(),
        );
    } else {
        context.replace(
            format!("Use `self` instead of `Object#to_s` in {usage}."),
            &selector,
            node.location(),
            "self",
        );
    }
}

fn usage_label(usage: &str) -> String {
    if usage == "interpolation" {
        usage.to_string()
    } else {
        format!("`{usage}`")
    }
}

fn interpolation_expression(node: &CallNode<'_>, context: &CopContext<'_, '_>) -> bool {
    context.ancestors().iter().rev().any(|ancestor| {
        ancestor
            .as_embedded_statements_node()
            .and_then(|embedded| embedded.statements())
            .and_then(|statements| statements.body().last())
            .is_some_and(|expression| {
                expression.location().start_offset() == node.location().start_offset()
                    && expression.location().end_offset() == node.location().end_offset()
            })
    })
}

fn enclosing_output_method<'a>(
    node: &CallNode<'_>,
    context: &'a CopContext<'_, '_>,
) -> Option<&'a str> {
    context.ancestors().iter().rev().find_map(|ancestor| {
        let call = ancestor.as_call_node()?;
        if call.receiver().is_some() || !matches!(call_name(&call), b"print" | b"puts" | b"warn") {
            return None;
        }
        let direct_argument = call.arguments().is_some_and(|arguments| {
            arguments.arguments().iter().any(|argument| {
                argument.location().start_offset() == node.location().start_offset()
                    && argument.location().end_offset() == node.location().end_offset()
            })
        });
        direct_argument.then(|| match call_name(&call) {
            b"print" => "print",
            b"puts" => "puts",
            _ => "warn",
        })
    })
}
