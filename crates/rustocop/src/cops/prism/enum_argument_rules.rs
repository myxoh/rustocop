use super::*;

define_cops! {
    ToEnumArguments => "Lint/ToEnumArguments" => call(to_enum_arguments),
}

fn to_enum_arguments(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) != b"to_enum"
        || node
            .receiver()
            .is_some_and(|receiver| receiver.as_self_node().is_none())
    {
        return;
    }
    let Some(definition) = context.ancestors().iter().rev().find_map(Node::as_def_node) else {
        return;
    };
    let Some(arguments) = node.arguments() else {
        return;
    };
    let ranges = source_syntax::top_level_elements(
        context.source(),
        arguments.location().start_offset(),
        arguments.location().end_offset(),
    );
    let Some(target) = ranges
        .first()
        .map(|range| context.source()[range.clone()].trim())
    else {
        return;
    };
    let method = String::from_utf8_lossy(definition.name().as_slice());
    if !matches!(target, "__method__" | "__callee__")
        && static_symbol_name(target) != Some(method.as_ref())
    {
        return;
    }
    let expected = expected_forwarded_arguments(&definition, context.source_file());
    let actual = ranges
        .iter()
        .skip(1)
        .map(|range| normalize(&context.source()[range.clone()]))
        .collect::<Vec<_>>();
    if actual != expected {
        context.report_call(node, "Ensure you correctly provided all the arguments.");
    }
}

fn expected_forwarded_arguments(
    definition: &ruby_prism::DefNode<'_>,
    file: SourceFile<'_>,
) -> Vec<String> {
    let Some(parameters) = definition.parameters() else {
        return Vec::new();
    };
    let location = parameters.location();
    let source = file.at(&location);
    let inner = source
        .strip_prefix('(')
        .and_then(|source| source.strip_suffix(')'))
        .unwrap_or(source);
    source_syntax::split_arguments(inner, 0, inner.len())
        .into_iter()
        .filter_map(|range| forwarded_parameter(inner[range].trim()))
        .collect()
}

fn forwarded_parameter(parameter: &str) -> Option<String> {
    if parameter.starts_with('&') {
        return None;
    }
    if matches!(parameter, "..." | "*" | "**") {
        return Some(parameter.to_string());
    }
    if parameter.starts_with('*') {
        return Some(normalize(parameter));
    }
    if let Some((name, _default)) = parameter.split_once('=') {
        return Some(normalize(name));
    }
    if let Some((name, _value)) = parameter.split_once(':') {
        let name = name.trim();
        return Some(format!("{name}:{name}"));
    }
    Some(normalize(parameter))
}

fn static_symbol_name(source: &str) -> Option<&str> {
    source.strip_prefix(':').filter(|name| {
        !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    })
}

fn normalize(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}
