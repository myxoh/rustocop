use super::*;

define_cops! {
    FloatDivision => "Style/FloatDivision" => compatibility_prism_call(float_division),
}

const MESSAGE: &str = "Do not apply inconsequential numeric operations to variables.";

fn useless_numeric_operation(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(call) = node.as_call_node() {
        check_numeric_call(&call, context);
    } else if let Some(write) = node.as_local_variable_operator_write_node() {
        let operator = write.binary_operator().as_slice();
        if !inconsequential(operator, &write.value()) {
            return;
        }
        let name = String::from_utf8_lossy(write.name().as_slice());
        context.replace_node(node, MESSAGE, format!("{name} = {name}"));
    }
}

fn check_numeric_call(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let (Some(receiver), Some(argument)) = (node.receiver(), only_argument(node)) else {
        return;
    };
    if !variable_read(&receiver) || !inconsequential(call_name(node), &argument) {
        return;
    }
    let replacement = context.source_file().node(&receiver).to_string();
    context.replace_call(node, MESSAGE, replacement);
}

fn inconsequential(operator: &[u8], value: &Node<'_>) -> bool {
    match operator {
        b"+" | b"-" => integer_value(value) == Some(0),
        b"*" | b"/" | b"**" => integer_value(value) == Some(1),
        _ => false,
    }
}

fn integer_value(node: &Node<'_>) -> Option<i32> {
    let integer = node.as_integer_node()?;
    TryInto::<i32>::try_into(integer.value()).ok()
}

fn variable_read(node: &Node<'_>) -> bool {
    node.as_call_node().is_some_and(|call| {
        call.receiver().is_none() && argument_count(&call) == 0 && call.block().is_none()
    })
}

fn float_division(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) != b"/" {
        return;
    }
    let (Some(left), Some(right)) = (node.receiver(), only_argument(node)) else {
        return;
    };
    let left_coercion = float_coercion(&left);
    let right_coercion = float_coercion(&right);
    let file = context.source_file();
    let style = context.policy().enforced_style("left_coerce");
    let (message, replacement) = match style {
        "left_coerce" if right_coercion.is_some() => {
            let left = left_coercion.as_ref().map_or_else(
                || format!("{}.to_f", file.node(&left)),
                |_| file.node(&left).to_string(),
            );
            let right = file.node(&right_coercion.unwrap());
            (
                "Prefer using `.to_f` on the left side.",
                format!("{left} / {right}"),
            )
        }
        "right_coerce" if left_coercion.is_some() => {
            let left = file.node(&left_coercion.unwrap());
            let right = right_coercion.as_ref().map_or_else(
                || format!("{}.to_f", file.node(&right)),
                |_| file.node(&right).to_string(),
            );
            (
                "Prefer using `.to_f` on the right side.",
                format!("{left} / {right}"),
            )
        }
        "single_coerce" if left_coercion.is_some() && right_coercion.is_some() => (
            "Prefer using `.to_f` on one side only.",
            format!(
                "{} / {}",
                file.node(&left),
                file.node(&right_coercion.unwrap())
            ),
        ),
        "fdiv" if left_coercion.is_some() || right_coercion.is_some() => {
            let left = left_coercion.as_ref().unwrap_or(&left);
            let right = right_coercion.as_ref().unwrap_or(&right);
            let right = strip_outer_parentheses(file.node(right));
            (
                "Prefer using `fdiv` for float divisions.",
                format!("{}.fdiv({right})", file.node(left)),
            )
        }
        _ => return,
    };
    context.replace_call(node, message, replacement);
}

fn float_coercion<'pr>(node: &Node<'pr>) -> Option<Node<'pr>> {
    let call = node.as_call_node()?;
    if call_name(&call) != b"to_f" || argument_count(&call) != 0 {
        return None;
    }
    let receiver = call.receiver()?;
    if receiver.as_numbered_reference_read_node().is_some()
        || receiver.as_call_node().is_some_and(|call| {
            call_name(&call) == b"last_match" && root_constant(call.receiver(), b"Regexp")
        })
    {
        return None;
    }
    Some(receiver)
}

fn strip_outer_parentheses(source: &str) -> &str {
    source
        .strip_prefix('(')
        .and_then(|source| source.strip_suffix(')'))
        .unwrap_or(source)
}
