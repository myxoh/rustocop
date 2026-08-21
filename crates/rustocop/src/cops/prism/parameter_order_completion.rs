use super::source_syntax::top_level_elements;
use super::*;

define_cops! {
    KeywordParametersOrder => "Style/KeywordParametersOrder" => any_node(keyword_parameters_order),
    ItAssignment => "Style/ItAssignment" => source(it_assignment),
}

fn it_assignment(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let signature = line.trim_start().starts_with("def ");
        for (at, _) in line.match_indices("it") {
            let before = line[..at].bytes().next_back();
            let after = line[at + 2..].bytes().next();
            if before.is_some_and(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'@' | b'$' | b'.')
            }) || after.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            {
                continue;
            }
            let tail = line[at + 2..].trim_start();
            if signature && line[..at].trim() == "def" {
                continue;
            }
            let assignment = tail.starts_with('=') && !tail.starts_with("==");
            let parameter = signature
                && (tail.is_empty()
                    || tail.starts_with([')', ',', ':', '='])
                    || tail.starts_with(" ="));
            if assignment || parameter {
                context.report(
                    "`it` is the default block parameter; consider another name.",
                    offset + at..offset + at + 2,
                );
            }
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
