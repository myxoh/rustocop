use super::*;

pub(super) fn shared_mutable_default(
    node: &ruby_prism::CallNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    const MESSAGE: &str = "Do not create a Hash with a mutable default value as the default value can accidentally be changed.";
    return_unless!(node.name().as_slice() == b"new");
    return_unless!(node
        .receiver()
        .is_some_and(|receiver| node_is_root_constant(&receiver, b"Hash")));
    let arguments = node
        .arguments()
        .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
        .unwrap_or_default();
    let Some(default) = arguments.first() else { return };

    if arguments.len() == 1 && capacity_keyword_argument(default) {
        return;
    }
    let mutable_literal = default.as_array_node().is_some()
        || default.as_hash_node().is_some()
        || default.as_keyword_hash_node().is_some();
    let mutable_constructor = default.as_call_node().is_some_and(|call| {
        call.name().as_slice() == b"new"
            && argument_count(&call) == 0
            && call.receiver().is_some_and(|receiver| {
                node_is_root_constant(&receiver, b"Array")
                    || node_is_root_constant(&receiver, b"Hash")
            })
    });
    let mutable_with_capacity = arguments.len() > 1 && default.as_hash_node().is_some();
    if mutable_with_capacity || arguments.len() == 1 && (mutable_literal || mutable_constructor) {
        context.report(MESSAGE, node.location());
    }
}

fn capacity_keyword_argument(node: &ruby_prism::Node<'_>) -> bool {
    let Some(hash) = node.as_keyword_hash_node() else {
        return false;
    };
    let elements = hash.elements().iter().collect::<Vec<_>>();
    let [element] = elements.as_slice() else {
        return false;
    };
    element
        .as_assoc_node()
        .and_then(|association| association.key().as_symbol_node())
        .is_some_and(|symbol| symbol.unescaped() == b"capacity")
}

pub(super) fn optional_arguments(
    node: &ruby_prism::DefNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    // RuboCop implements only `on_def`; singleton definitions (`defs`) are
    // intentionally outside this cop's callback surface.
    if node.receiver().is_some() {
        return;
    }
    let Some(parameters) = node.parameters() else {
        return;
    };
    if parameters.posts().is_empty() {
        return;
    }
    for optional in parameters.optionals().iter() {
        context.report(
            "Optional arguments should appear at the end of the argument list.",
            optional.location(),
        );
    }
}

pub(super) fn optional_boolean_parameter(
    node: &ruby_prism::DefNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    use crate::rubocop::cop::mixin::allowed_methods::AllowedMethods;

    let method_name = String::from_utf8_lossy(node.name().as_slice());
    let allowed_methods = AllowedMethods::new(
        context.config_values("AllowedMethods").to_vec(),
        Vec::new(),
        Vec::new(),
    );
    if allowed_methods.allowed_method(&method_name) {
        return;
    }
    let Some(parameters) = node.parameters() else {
        return;
    };
    for optional in parameters.optionals().iter() {
        let Some(optional) = optional.as_optional_parameter_node() else {
            continue;
        };
        let value = optional.value();
        let value = if value.as_true_node().is_some() {
            "true"
        } else if value.as_false_node().is_some() {
            "false"
        } else {
            continue;
        };
        let text = context.source_file().at(&optional.location());
        let name = String::from_utf8_lossy(optional.name().as_slice());
        context.report(
            format!("Prefer keyword arguments for arguments with a boolean default value; use `{name}: {value}` instead of `{text}`."),
            optional.location(),
        );
    }
}
