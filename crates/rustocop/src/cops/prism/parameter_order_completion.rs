use super::source_syntax::top_level_elements;
use super::*;

define_cops! {
    KeywordParametersOrder => "Style/KeywordParametersOrder" => compatibility_prism_any_node(keyword_parameters_order),
    ItAssignment => "Style/ItAssignment" => compatibility_prism_any_node(it_assignment),
}

fn it_assignment(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let candidate = if let Some(write) = node.as_local_variable_write_node() {
        Some((write.name(), write.name_loc()))
    } else if let Some(target) = node.as_local_variable_target_node() {
        Some((target.name(), target.location()))
    } else if let Some(write) = node.as_local_variable_and_write_node() {
        Some((write.name(), write.name_loc()))
    } else if let Some(write) = node.as_local_variable_or_write_node() {
        Some((write.name(), write.name_loc()))
    } else { node.as_local_variable_operator_write_node().map(|write| (write.name(), write.name_loc())) };
    if let Some((name, location)) = candidate {
        report_it_name(name.as_slice(), location, context);
    }

    let parameters = if let Some(definition) = node.as_def_node() {
        definition.parameters()
    } else if let Some(block) = node.as_block_node() {
        block
            .parameters()
            .and_then(|parameters| parameters.as_block_parameters_node())
            .and_then(|parameters| parameters.parameters())
    } else if let Some(lambda) = node.as_lambda_node() {
        lambda
            .parameters()
            .and_then(|parameters| parameters.as_block_parameters_node())
            .and_then(|parameters| parameters.parameters())
    } else {
        None
    };
    if let Some(parameters) = parameters {
        report_it_parameters(&parameters, context);
    }
}

fn report_it_name(
    name: &[u8],
    location: ruby_prism::Location<'_>,
    context: &mut CopContext<'_, '_>,
) {
    if name != b"it" {
        return;
    }
    let mut end = location.end_offset();
    if context.source().as_bytes().get(end.saturating_sub(1)) == Some(&b':') {
        end -= 1;
    }
    context.report(
        "`it` is the default block parameter; consider another name.",
        location.start_offset()..end,
    );
}

fn report_it_parameters(
    parameters: &ruby_prism::ParametersNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    for parameter in parameters
        .requireds()
        .iter()
        .chain(parameters.posts().iter())
    {
        if let Some(parameter) = parameter.as_required_parameter_node() {
            report_it_name(parameter.name().as_slice(), parameter.location(), context);
        }
    }
    for parameter in parameters.optionals().iter() {
        if let Some(parameter) = parameter.as_optional_parameter_node() {
            report_it_name(parameter.name().as_slice(), parameter.name_loc(), context);
        }
    }
    if let Some(parameter) = parameters
        .rest()
        .and_then(|parameter| parameter.as_rest_parameter_node())
    {
        if let (Some(name), Some(location)) = (parameter.name(), parameter.name_loc()) {
            report_it_name(name.as_slice(), location, context);
        }
    }
    for parameter in parameters.keywords().iter() {
        if let Some(parameter) = parameter.as_required_keyword_parameter_node() {
            report_it_name(parameter.name().as_slice(), parameter.name_loc(), context);
        } else if let Some(parameter) = parameter.as_optional_keyword_parameter_node() {
            report_it_name(parameter.name().as_slice(), parameter.name_loc(), context);
        }
    }
    if let Some(parameter) = parameters
        .keyword_rest()
        .and_then(|parameter| parameter.as_keyword_rest_parameter_node())
    {
        if let (Some(name), Some(location)) = (parameter.name(), parameter.name_loc()) {
            report_it_name(name.as_slice(), location, context);
        }
    }
    if let Some(parameter) = parameters.block() {
        if let (Some(name), Some(location)) = (parameter.name(), parameter.name_loc()) {
            report_it_name(name.as_slice(), location, context);
        }
    }
}

fn keyword_parameters_order(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let location = if let Some(definition) = node.as_def_node() {
        definition
            .parameters()
            .map(|parameters| parameters.location())
    } else if let Some(block) = node.as_block_node() {
        block
            .parameters()
            .and_then(|parameters| parameters.as_block_parameters_node())
            .map(|parameters| parameters.location())
    } else {
        None
    };
    let Some(location) = location else {
        return;
    };
    let raw = context.source_file().at(&location);
    let (prefix, suffix) = if raw.starts_with('(') && raw.ends_with(')') {
        ("(", ")")
    } else if raw.starts_with('|') && raw.ends_with('|') {
        ("|", "|")
    } else {
        ("", "")
    };
    let inner_start = location.start_offset() + prefix.len();
    let inner_end = location.end_offset() - suffix.len();
    let ranges = top_level_elements(context.source(), inner_start, inner_end);
    let parameters = ranges
        .iter()
        .map(|range| context.source()[range.clone()].trim())
        .collect::<Vec<_>>();
    let required_after = |index: usize| {
        parameters[index + 1..]
            .iter()
            .any(|parameter| required_keyword(parameter))
    };
    let offending = parameters
        .iter()
        .enumerate()
        .filter(|(index, parameter)| optional_keyword(parameter) && required_after(*index))
        .map(|(index, _)| ranges[index].clone())
        .collect::<Vec<_>>();
    if offending.is_empty() {
        return;
    }
    let mut ordered = Vec::<&str>::new();
    ordered.extend(
        parameters
            .iter()
            .filter(|parameter| !keyword_parameter(parameter))
            .copied(),
    );
    ordered.extend(
        parameters
            .iter()
            .filter(|parameter| required_keyword(parameter))
            .copied(),
    );
    ordered.extend(
        parameters
            .iter()
            .filter(|parameter| optional_keyword(parameter))
            .copied(),
    );
    ordered.extend(
        parameters
            .iter()
            .filter(|parameter| parameter.starts_with("**") || parameter.starts_with('&'))
            .copied(),
    );
    let replacement = format!("{prefix}{}{suffix}", ordered.join(", "));
    for (index, offense) in offending.into_iter().enumerate() {
        if raw.contains('#') {
            context.report(
                "Place optional keyword parameters at the end of the parameters list.",
                offense,
            );
        } else if index == 0 {
            context.replace(
                "Place optional keyword parameters at the end of the parameters list.",
                offense,
                &location,
                replacement.clone(),
            );
        } else {
            context.replace_indirectly(
                "Place optional keyword parameters at the end of the parameters list.",
                offense.clone(),
                offense.clone(),
                &context.source()[offense],
            );
        }
    }
}

fn keyword_parameter(parameter: &str) -> bool {
    required_keyword(parameter)
        || optional_keyword(parameter)
        || parameter.starts_with("**")
        || parameter.starts_with('&')
}

fn required_keyword(parameter: &str) -> bool {
    parameter.ends_with(':') && !parameter.starts_with("**")
}

fn optional_keyword(parameter: &str) -> bool {
    parameter
        .split_once(':')
        .is_some_and(|(name, value)| valid_keyword_name(name) && !value.trim().is_empty())
}

fn valid_keyword_name(name: &str) -> bool {
    let mut characters = name.trim().chars();
    characters
        .next()
        .is_some_and(|character| character == '_' || character.is_alphabetic())
        && characters.all(|character| character == '_' || character.is_alphanumeric())
}
