use super::*;

define_cops! {
    NestedModifier => "Style/NestedModifier" => compatibility_prism_any_node(nested_modifier),
}

const MESSAGE: &str = "Avoid using nested modifiers.";

fn nested_modifier(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if context.ancestors().iter().rev().any(|ancestor| {
        ModifierConditional::from_node(ancestor)
            .is_some_and(|modifier| same_location(&modifier.body, node))
    }) {
        return;
    }
    let Some(outer) = ModifierConditional::from_node(node) else {
        return;
    };
    let Some(inner) = ModifierConditional::from_node(&outer.body) else {
        return;
    };
    if matches!(outer.kind, ModifierKind::While | ModifierKind::Until)
        || matches!(inner.kind, ModifierKind::While | ModifierKind::Until)
    {
        context.report(MESSAGE, &inner.keyword);
        return;
    }

    let file = context.source_file();
    let statement = file.node(&inner.body);
    let outer_predicate = predicate_source(&outer.predicate, file, outer.kind == ModifierKind::If);
    let replacement = match (inner.kind, outer.kind) {
        (ModifierKind::If, ModifierKind::If) => format!(
            "{statement} if {outer_predicate} && {}",
            predicate_source(&inner.predicate, file, true)
        ),
        (ModifierKind::Unless, ModifierKind::Unless) => format!(
            "{statement} unless {outer_predicate} || {}",
            predicate_source(&inner.predicate, file, false)
        ),
        (ModifierKind::If, ModifierKind::Unless) => format!(
            "{statement} unless {outer_predicate} || {}",
            negated_source(&inner.predicate, file)
        ),
        (ModifierKind::Unless, ModifierKind::If) => format!(
            "{statement} if {outer_predicate} && {}",
            negated_source(&inner.predicate, file)
        ),
        _ => return,
    };
    context.replace(MESSAGE, &inner.keyword, &outer.location, replacement);
}

fn predicate_source(node: &Node<'_>, file: SourceFile<'_>, conjunction: bool) -> String {
    let source = render_command_call(node, file).unwrap_or_else(|| file.node(node).to_string());
    if conjunction && (node.as_and_node().is_some() || node.as_or_node().is_some()) {
        format!("({source})")
    } else {
        source
    }
}

fn negated_source(node: &Node<'_>, file: SourceFile<'_>) -> String {
    if let Some(call) = render_command_call(node, file) {
        return format!("!{call}");
    }
    let source = file.node(node);
    if simple_negation_target(node) {
        format!("!{source}")
    } else {
        format!("!({source})")
    }
}

fn simple_negation_target(node: &Node<'_>) -> bool {
    node.as_local_variable_read_node().is_some()
        || node.as_instance_variable_read_node().is_some()
        || node.as_class_variable_read_node().is_some()
        || node.as_global_variable_read_node().is_some()
        || node.as_constant_read_node().is_some()
        || node.as_constant_path_node().is_some()
        || node.as_self_node().is_some()
        || node.as_call_node().is_some_and(|call| {
            argument_count(&call) == 0
                && call.block().is_none()
                && method_identifier(call_name(&call))
        })
}

fn render_command_call(node: &Node<'_>, file: SourceFile<'_>) -> Option<String> {
    let call = node.as_call_node()?;
    if call.opening_loc().is_some()
        || argument_count(&call) == 0
        || call.block().is_some()
        || !method_identifier(call_name(&call))
    {
        return None;
    }
    let receiver = call.receiver().map(|receiver| file.node(&receiver));
    let operator = call
        .call_operator_loc()
        .map(|operator| String::from_utf8_lossy(operator.as_slice()).into_owned())
        .unwrap_or_default();
    let selector = String::from_utf8_lossy(call_name(&call));
    let arguments = joined_arguments(&call, file, ", ");
    Some(match receiver {
        Some(receiver) => format!("{receiver}{operator}{selector}({arguments})"),
        None => format!("{selector}({arguments})"),
    })
}

fn method_identifier(name: &[u8]) -> bool {
    name.first()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
        && name
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'?' | b'!' | b'='))
}
