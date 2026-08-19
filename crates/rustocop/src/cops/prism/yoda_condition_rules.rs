use super::*;

define_cops! {
    YodaCondition => "Style/YodaCondition" => call(yoda_condition),
}

fn yoda_condition(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let operator = call_name(node);
    if !matches!(operator, b"==" | b"!=" | b"<" | b"<=" | b">" | b">=") {
        return;
    }
    let style = context
        .policy()
        .enforced_style("forbid_for_all_comparison_operators");
    if style.ends_with("equality_operators_only") && !matches!(operator, b"==" | b"!=") {
        return;
    }
    let (Some(left), Some(right)) = (node.receiver(), only_argument(node)) else {
        return;
    };
    if file_program_name_comparison(&left, operator, &right, context)
        || interpolated_literal(&left)
    {
        return;
    }

    let left_constant = constant_portion(&left);
    let right_constant = constant_portion(&right);
    if left_constant == right_constant {
        return;
    }
    let require_yoda = style.starts_with("require_");
    if (require_yoda && left_constant) || (!require_yoda && right_constant) {
        return;
    }

    let file = context.source_file();
    let original = file.node(&node.as_node());
    let reverse = match operator {
        b"<" => ">",
        b"<=" => ">=",
        b">" => "<",
        b">=" => "<=",
        _ => std::str::from_utf8(operator).unwrap_or_default(),
    };
    let correction = format!("{} {reverse} {}", file.node(&right), file.node(&left));
    context.replace_call(
        node,
        format!("Reverse the order of the operands `{original}`."),
        correction,
    );
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

fn interpolated_literal(node: &Node<'_>) -> bool {
    node.as_interpolated_string_node().is_some()
        || node.as_interpolated_regular_expression_node().is_some()
}

fn file_program_name_comparison(
    left: &Node<'_>,
    operator: &[u8],
    right: &Node<'_>,
    context: &CopContext<'_, '_>,
) -> bool {
    matches!(operator, b"==" | b"!=")
        && context.source_file().node(left) == "__FILE__"
        && right.as_global_variable_read_node().is_some_and(|global| {
            matches!(global.name().as_slice(), b"$0" | b"$PROGRAM_NAME")
        })
}
