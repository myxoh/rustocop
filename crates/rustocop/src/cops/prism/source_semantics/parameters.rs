use super::*;

pub(super) fn shared_mutable_default(context: &mut CopContext<'_, '_>) {
    const MESSAGE: &str = "Do not create a Hash with a mutable default value as the default value can accidentally be changed.";
    let source = context.source();
    for call in call_ranges(source, "Hash.new(") {
        let arguments = source[call.start + 9..call.end - 1].trim();
        let default = arguments.split(',').next().unwrap_or_default().trim();
        if matches!(default, "[]" | "{}" | "Array.new" | "Hash.new")
            && !arguments.contains(".freeze")
        {
            context.report(MESSAGE, call);
        }
    }
    for (start, line) in source_lines(source) {
        let trimmed = line.trim();
        if trimmed.starts_with("Hash.new Array.new") {
            let leading = line.len() - line.trim_start().len();
            context.report(MESSAGE, start + leading..start + line.len());
        }
    }
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
