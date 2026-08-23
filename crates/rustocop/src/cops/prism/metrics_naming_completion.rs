use super::source_syntax::top_level_elements;
use super::*;

define_cops! {
    ParameterLists => "Metrics/ParameterLists" => node(as_def_node, parameter_lists),
    CollectionLiteralLength => "Metrics/CollectionLiteralLength" => any_node(collection_literal_length),
    BinaryOperatorParameterName => "Naming/BinaryOperatorParameterName" => node(as_def_node, binary_operator_parameter_name),
    BlockParameterName => "Naming/BlockParameterName" => source(block_parameter_name),
    PredicatePrefix => "Naming/PredicatePrefix" => any_node(predicate_prefix),
}

fn predicate_prefix(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (name, location) = if let Some(definition) = node.as_def_node() {
        (
            String::from_utf8_lossy(definition.name().as_slice()).into_owned(),
            definition.name_loc(),
        )
    } else if let Some(call) = node.as_call_node() {
        if call.receiver().is_some() {
            return;
        }
        let method = String::from_utf8_lossy(call_name(&call));
        if !context
            .config_values("MethodDefinitionMacros")
            .iter()
            .any(|configured| configured == &method)
        {
            return;
        }
        let Some(argument) = first_argument(&call) else {
            return;
        };
        let Some(symbol) = argument.as_symbol_node() else {
            return;
        };
        (
            String::from_utf8_lossy(symbol.unescaped()).into_owned(),
            symbol.location(),
        )
    } else {
        return;
    };
    if name.ends_with('=')
        || context
            .config_values("AllowedMethods")
            .iter()
            .any(|allowed| allowed == &name)
    {
        return;
    }
    let Some(prefix) = context
        .config_values("NamePrefix")
        .iter()
        .find(|prefix| name.starts_with(prefix.as_str()))
    else {
        return;
    };
    let base = name
        .strip_prefix(prefix)
        .unwrap_or(&name)
        .trim_end_matches('?');
    if base
        .as_bytes()
        .first()
        .is_none_or(|byte| !byte.is_ascii_alphabetic() && *byte != b'_')
    {
        return;
    }
    if context.config_value("UseSorbetSigs") == Some("true")
        || context.config_bool("UseSorbetSigs", false)
    {
        let before = &context.source()[..location.start_offset()];
        let recent = before
            .rsplit_once("sig")
            .map_or("", |(_, signature)| signature);
        if !recent.contains("T::Boolean") {
            return;
        }
    }
    let forbidden = context
        .config_values("ForbiddenPrefixes")
        .iter()
        .any(|forbidden| forbidden == prefix);
    if !forbidden && name.ends_with('?') {
        return;
    }
    let replacement = if forbidden {
        format!("{base}?")
    } else {
        format!("{}?", name.trim_end_matches('?'))
    };
    context.report(format!("Rename `{name}` to `{replacement}`."), location);
}

fn parameter_lists(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.name().as_slice() == b"initialize"
        && context.ancestors().iter().any(|ancestor| {
            let source = context.source_file().at(&ancestor.location()).trim_start();
            ["Struct.new", "::Struct.new", "Data.define", "::Data.define"]
                .iter()
                .any(|prefix| source.starts_with(prefix))
        })
    {
        return;
    }
    let Some(parameters) = node.parameters() else {
        return;
    };
    let source = context.source_file().at(&parameters.location());
    let inner = source
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .unwrap_or(source);
    let base = parameters.location().start_offset() + usize::from(source.starts_with('('));
    let ranges = top_level_elements(context.source(), base, base + inner.len());
    let count_keywords = context.config_bool("CountKeywordArgs", true);
    let count = ranges
        .iter()
        .filter(|range| {
            let parameter = context.source()[(*range).clone()].trim();
            !parameter.starts_with('&') && (count_keywords || !parameter.contains(':'))
        })
        .count();
    let maximum = context.config_usize("Max", 5);
    if count > maximum {
        let raw_start = parameters.location().start_offset();
        let raw_end = parameters.location().end_offset();
        let start = raw_start.saturating_sub(usize::from(
            context.source().as_bytes().get(raw_start.saturating_sub(1)) == Some(&b'('),
        ));
        let end = raw_end + usize::from(context.source().as_bytes().get(raw_end) == Some(&b')'));
        context.report(
            format!("Avoid parameter lists longer than {maximum} parameters. [{count}/{maximum}]"),
            start..end,
        );
    }
    let optional = ranges
        .iter()
        .filter(|range| {
            let parameter = context.source()[(*range).clone()].trim();
            parameter.contains('=') && !parameter.starts_with("**")
        })
        .count();
    let optional_max = context.config_usize("MaxOptionalParameters", 3);
    if optional > optional_max {
        context.report(
            format!("Method has too many optional parameters. [{optional}/{optional_max}]"),
            node.location(),
        );
    }
}

fn collection_literal_length(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let threshold = context.config_usize("LengthThreshold", 250);
    if context.ancestors().iter().any(|ancestor| {
        ancestor.location().start_offset() == node.location().start_offset()
            && collection_element_count(ancestor).is_some_and(|count| count >= threshold)
    }) {
        return;
    }
    let count = if let Some(array) = node.as_array_node() {
        array.elements().len()
    } else if let Some(hash) = node.as_hash_node() {
        hash.elements().len()
    } else if let Some(hash) = node.as_keyword_hash_node() {
        hash.elements().len()
    } else if let Some(call) = node.as_call_node() {
        if call_name(&call) == b"[]" && root_constant(call.receiver(), b"Set") {
            argument_count(&call)
        } else {
            return;
        }
    } else if let Some(rescue) = node.as_rescue_node() {
        let exceptions = rescue.exceptions().iter().collect::<Vec<_>>();
        if exceptions.len() >= threshold {
            let Some((first, last)) = exceptions.first().zip(exceptions.last()) else {
                return;
            };
            context.report(
                "Avoid hard coding large quantities of data in code. Prefer reading the data from an external source.",
                first.location().start_offset()..last.location().end_offset(),
            );
        }
        return;
    } else {
        return;
    };
    if count >= threshold {
        context.report_node(
            node,
            "Avoid hard coding large quantities of data in code. Prefer reading the data from an external source.",
        );
    }
}

fn collection_element_count(node: &Node<'_>) -> Option<usize> {
    if let Some(array) = node.as_array_node() {
        Some(array.elements().len())
    } else if let Some(hash) = node.as_hash_node() {
        Some(hash.elements().len())
    } else if let Some(hash) = node.as_keyword_hash_node() {
        Some(hash.elements().len())
    } else if let Some(call) = node.as_call_node() {
        (call_name(&call) == b"[]" && root_constant(call.receiver(), b"Set"))
            .then(|| argument_count(&call))
    } else {
        None
    }
}

fn binary_operator_parameter_name(
    node: &ruby_prism::DefNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    const OPERATORS: &[&[u8]] = &[
        b"+", b"-", b"*", b"/", b"%", b"**", b"==", b"!=", b">", b"<", b">=", b"<=", b"<=>",
        b"eql?", b"equal?", b"|", b"&", b"^",
    ];
    if !OPERATORS.contains(&node.name().as_slice()) {
        return;
    }
    let Some(parameters) = node.parameters() else {
        return;
    };
    let parameter_source = context.source_file().at(&parameters.location());
    let old = parameter_source.trim_matches(['(', ')', ' ', '\n']);
    if old.is_empty() || old.contains(',') || matches!(old, "other" | "_other") {
        return;
    }
    let parameter_start =
        parameters.location().start_offset() + parameter_source.find(old).unwrap_or(0);
    let offense = parameter_start..parameter_start + old.len();
    let mut edits = vec![(offense.clone(), "other".to_string())];
    let location = node.location();
    edits.extend(
        identifier_ranges(
            context.source(),
            location.start_offset(),
            location.end_offset(),
            old,
        )
        .filter(|range| *range != offense)
        .map(|range| (range, "other".to_string())),
    );
    let operator = String::from_utf8_lossy(node.name().as_slice());
    context.replace_many(
        format!("When defining the `{operator}` operator, name its argument `other`."),
        offense,
        edits,
    );
}

fn identifier_ranges<'a>(
    source: &'a str,
    start: usize,
    end: usize,
    name: &'a str,
) -> impl Iterator<Item = std::ops::Range<usize>> + 'a {
    source[start..end]
        .match_indices(name)
        .filter_map(move |(relative, value)| {
            let range = start + relative..start + relative + value.len();
            let before = source[..range.start].bytes().next_back();
            let after = source[range.end..].bytes().next();
            (!before.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                && !after.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_'))
            .then_some(range)
        })
}

fn block_parameter_name(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let minimum = context.config_usize("MinNameLength", 2);
    let allow_numbers = context.config_bool("AllowNamesEndingInNumbers", false);
    let allowed = context.config_values("AllowedNames").to_vec();
    let forbidden = context.config_values("ForbiddenNames").to_vec();
    let mut search = 0;
    while let Some(first_relative) = source[search..].find('|') {
        let first = search + first_relative;
        let Some(second_relative) = source[first + 1..].find('|') else {
            break;
        };
        let second = first + 1 + second_relative;
        if source[first + 1..second].contains('\n') {
            search = second + 1;
            continue;
        }
        for (relative, raw) in split_parameters(&source[first + 1..second]) {
            let name = raw.trim_start_matches(['*', '&']);
            let bare = name.trim_start_matches('_');
            if allowed.iter().any(|allowed| allowed == bare) {
                continue;
            }
            let start = first + 1 + relative;
            let range = start..start + raw.len();
            let message = if forbidden.iter().any(|forbidden| forbidden == bare) {
                Some(format!(
                    "Do not use {bare} as a name for a block parameter."
                ))
            } else if bare.len() < minimum {
                Some(format!(
                    "Block parameter must be at least {minimum} characters long."
                ))
            } else if bare.bytes().any(|byte| byte.is_ascii_uppercase()) {
                Some("Only use lowercase characters for block parameter.".to_string())
            } else if !allow_numbers
                && bare
                    .bytes()
                    .next_back()
                    .is_some_and(|byte| byte.is_ascii_digit())
            {
                Some("Do not end block parameter with a number.".to_string())
            } else {
                None
            };
            if let Some(message) = message {
                context.report(message, range);
            }
        }
        search = second + 1;
    }
}

fn split_parameters(source: &str) -> Vec<(usize, &str)> {
    let mut start = 0;
    let mut result = Vec::new();
    for (at, character) in source.char_indices() {
        if character == ',' {
            let raw = source[start..at].trim();
            if !raw.is_empty() {
                result.push((start + source[start..at].find(raw).unwrap_or(0), raw));
            }
            start = at + 1;
        }
    }
    let raw = source[start..].trim();
    if !raw.is_empty() {
        result.push((start + source[start..].find(raw).unwrap_or(0), raw));
    }
    result
}
