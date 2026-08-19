use super::*;

define_cops! {
    YodaCondition => "Style/YodaCondition" => call(on_send),
}

fn on_send(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let operator = call_name(node);
    if !yoda_compatible_condition(operator) {
        return;
    }
    let style = context
        .policy()
        .enforced_style("forbid_for_all_comparison_operators");
    if equality_only(style) && non_equality_operator(operator) {
        return;
    }
    let (Some(left), Some(right)) = (node.receiver(), only_argument(node)) else {
        return;
    };
    if file_constant_equal_program_name(&left, operator, &right, context)
        || valid_yoda(&left, &right, style)
    {
        return;
    }

    let offense = actual_code_range(node);
    let message = message(node, context);
    let correction = corrected_code(&left, operator, &right, context);
    context.add_offense(offense.clone(), message, |corrector| {
        corrector.replace(offense, correction);
    });
}

fn yoda_compatible_condition(operator: &[u8]) -> bool {
    comparison_operator(operator) && !noncommutative_operator(operator)
}

fn comparison_operator(operator: &[u8]) -> bool {
    matches!(
        operator,
        b"==" | b"!=" | b"<" | b"<=" | b">" | b">=" | b"===" | b"=~" | b"!~"
    )
}

fn noncommutative_operator(operator: &[u8]) -> bool {
    matches!(operator, b"===" | b"=~" | b"!~")
}

fn equality_only(style: &str) -> bool {
    style.ends_with("equality_operators_only")
}

fn non_equality_operator(operator: &[u8]) -> bool {
    !matches!(operator, b"==" | b"!=")
}

fn valid_yoda(left: &Node<'_>, right: &Node<'_>, style: &str) -> bool {
    let left_constant = constant_portion(left);
    let right_constant = constant_portion(right);
    if left_constant == right_constant || interpolation(left) {
        return true;
    }
    if enforce_yoda(style) {
        left_constant
    } else {
        right_constant
    }
}

fn enforce_yoda(style: &str) -> bool {
    style.starts_with("require_")
}

fn message(node: &CallNode<'_>, context: &CopContext<'_, '_>) -> String {
    format!(
        "Reverse the order of the operands `{}`.",
        context.source_file().node(&node.as_node())
    )
}

fn corrected_code(
    left: &Node<'_>,
    operator: &[u8],
    right: &Node<'_>,
    context: &CopContext<'_, '_>,
) -> String {
    let file = context.source_file();
    format!(
        "{} {} {}",
        file.node(right),
        reverse_comparison(operator),
        file.node(left)
    )
}

fn reverse_comparison(operator: &[u8]) -> &str {
    match operator {
        b"<" => ">",
        b"<=" => ">=",
        b">" => "<",
        b">=" => "<=",
        _ => std::str::from_utf8(operator).unwrap_or_default(),
    }
}

fn actual_code_range(node: &CallNode<'_>) -> std::ops::Range<usize> {
    let location = node.location();
    location.start_offset()..location.end_offset()
}

fn constant_portion(node: &Node<'_>) -> bool {
    if scalar_literal(node)
        || node.as_constant_read_node().is_some()
        || node.as_constant_path_node().is_some()
    {
        return true;
    }
    if let Some(array) = node.as_array_node() {
        return array.elements().iter().all(|element| constant_portion(&element));
    }
    if let Some(hash) = node.as_hash_node() {
        return hash.elements().iter().all(|element| {
            element.as_assoc_node().is_some_and(|pair| {
                constant_portion(&pair.key()) && constant_portion(&pair.value())
            })
        });
    }
    if let Some(range) = node.as_range_node() {
        return range.left().is_none_or(|left| constant_portion(&left))
            && range.right().is_none_or(|right| constant_portion(&right));
    }
    false
}

fn scalar_literal(node: &Node<'_>) -> bool {
    node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_rational_node().is_some()
        || node.as_imaginary_node().is_some()
        || node.as_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_nil_node().is_some()
        || node.as_regular_expression_node().is_some()
}

fn interpolation(node: &Node<'_>) -> bool {
    node.as_interpolated_string_node().is_some()
        || node.as_interpolated_regular_expression_node().is_some()
}

fn file_constant_equal_program_name(
    left: &Node<'_>,
    operator: &[u8],
    right: &Node<'_>,
    context: &CopContext<'_, '_>,
) -> bool {
    matches!(operator, b"==" | b"!=")
        && source_file_path_constant(left, context)
        && program_name(right)
}

fn source_file_path_constant(node: &Node<'_>, context: &CopContext<'_, '_>) -> bool {
    context.source_file().node(node) == "__FILE__"
}

fn program_name(node: &Node<'_>) -> bool {
    node.as_global_variable_read_node().is_some_and(|global| {
        matches!(global.name().as_slice(), b"$0" | b"$PROGRAM_NAME")
    })
}
