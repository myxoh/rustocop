use super::source_syntax::top_level_elements;
use super::*;

define_cops! {
    ParameterLists => "Metrics/ParameterLists" => any_node(parameter_lists),
    CollectionLiteralLength => "Metrics/CollectionLiteralLength" => any_node(collection_literal_length),
    BlockParameterName => "Naming/BlockParameterName" => any_node(block_parameter_name),
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

fn parameter_lists(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(definition) = node.as_def_node() {
        parameter_lists_definition(&definition, context);
    } else if let Some(block) = node.as_block_node() {
        parameter_lists_block(&block, context);
    }
}

fn parameter_lists_definition(
    node: &ruby_prism::DefNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    if node.name().as_slice() == b"initialize"
        && context.ancestors().iter().any(|ancestor| {
            let source = context.source_file().at(&ancestor.location()).trim_start();
            ["Struct.new", "::Struct.new", "Data.define", "::Data.define"]
                .iter()
                .any(|prefix| source.starts_with(prefix))
        })
        && context
            .ancestors()
            .iter()
            .rev()
            .find_map(Node::as_block_node)
            .and_then(|block| block.body())
            .and_then(|body| body.as_statements_node())
            .is_some_and(|statements| statements.body().len() == 1)
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
        let start = node
            .lparen_loc()
            .map_or(parameters.location().start_offset(), |left| {
                left.start_offset()
            });
        let end = node
            .rparen_loc()
            .map_or(parameters.location().end_offset(), |right| {
                right.end_offset()
            });
        let message =
            format!("Avoid parameter lists longer than {maximum} parameters. [{count}/{maximum}]");
        if context.source()[start..end].contains("# rubocop:disable Metrics/ParameterLists") {
            context.add_offense(start..end, message, |_| {});
        } else {
            context.report(message, start..end);
        }
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

fn parameter_lists_block(node: &ruby_prism::BlockNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(parameters) = node
        .parameters()
        .and_then(|parameters| parameters.as_block_parameters_node())
    else {
        return;
    };
    if context.parent().and_then(Node::as_call_node).is_some_and(|call| {
        matches!(call_name(&call), b"lambda" | b"proc")
            || call_name(&call) == b"new"
                && call
                    .receiver()
                    .is_some_and(|receiver| root_constant(Some(receiver), b"Proc"))
    }) {
        return;
    }
    let location = parameters.location();
    let source = context.source_file().at(&location);
    let inner = source
        .strip_prefix('|')
        .and_then(|value| value.strip_suffix('|'))
        .unwrap_or(source);
    let base = location.start_offset() + usize::from(source.starts_with('|'));
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
        context.report(
            format!("Avoid parameter lists longer than {maximum} parameters. [{count}/{maximum}]"),
            location,
        );
    }
}

fn collection_literal_length(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let threshold = context.config_usize("LengthThreshold", 250);
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

fn binary_operator_parameter_name(
    node: &ruby_prism::DefNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    if node.receiver().is_some() || !checked_binary_operator(node.name().as_slice()) {
        return;
    }
    let Some(parameters) = node.parameters() else {
        return;
    };
    if parameters.requireds().len() != 1
        || !parameters.optionals().is_empty()
        || parameters.rest().is_some()
        || !parameters.posts().is_empty()
        || !parameters.keywords().is_empty()
        || parameters.keyword_rest().is_some()
        || parameters.block().is_some()
    {
        return;
    }
    let Some(parameter) = parameters.requireds().iter().next()
        .and_then(|parameter| parameter.as_required_parameter_node()) else { return };
    let old = String::from_utf8_lossy(parameter.name().as_slice());
    if matches!(old.as_ref(), "other" | "_other") {
        return;
    }
    let offense = parameter.location().start_offset()..parameter.location().end_offset();
    let mut edits = vec![(offense.clone(), "other".to_string())];
    let location = node.location();
    edits.extend(
        identifier_ranges(
            context.source(),
            location.start_offset(),
            location.end_offset(),
            &old,
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

fn checked_binary_operator(name: &[u8]) -> bool {
    if matches!(name, b"eql?" | b"equal?") {
        return true;
    }
    if matches!(name, b"+@" | b"-@" | b"[]" | b"[]=" | b"<<" | b"===" | b"`" | b"=~") {
        return false;
    }
    std::str::from_utf8(name)
        .ok()
        .and_then(|name| name.chars().next())
        .is_some_and(|character| !character.is_alphanumeric() && character != '_')
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

fn block_parameter_name(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let parameters = if let Some(block) = node.as_block_node() {
        block
            .parameters()
            .and_then(|parameters| parameters.as_block_parameters_node())
            .and_then(|parameters| parameters.parameters())
    } else if let Some(lambda) = node.as_lambda_node() {
        lambda
            .parameters()
            .and_then(|parameters| parameters.as_parameters_node())
    } else {
        return;
    };
    let Some(parameters) = parameters else { return };
    let minimum = context.config_usize("MinNameLength", 1);
    let allow_numbers = context.config_bool("AllowNamesEndingInNumbers", false);
    let allowed = context.config_values("AllowedNames").to_vec();
    let forbidden = context.config_values("ForbiddenNames").to_vec();
    for parameter in block_name_parameters(&parameters) {
        if parameter.name == "_" {
            continue;
        }
        let bare = parameter.name.trim_start_matches('_');
        if allowed.iter().any(|allowed| allowed == bare) {
            continue;
        }
        if forbidden.iter().any(|forbidden| forbidden == bare) {
            context.report(
                format!("Do not use {bare} as a name for a block parameter."),
                parameter.range.clone(),
            );
        }
        if bare.chars().any(char::is_uppercase) {
            context.report(
                "Only use lowercase characters for block parameter.",
                parameter.range.clone(),
            );
        }
        if bare.chars().count() < minimum {
            context.report(
                format!("Block parameter must be at least {minimum} characters long."),
                parameter.range.clone(),
            );
        }
        if !allow_numbers && bare.ends_with(|character: char| character.is_ascii_digit()) {
            context.report(
                "Do not end block parameter with a number.",
                parameter.range,
            );
        }
    }
}

struct BlockNameParameter {
    name: String,
    range: std::ops::Range<usize>,
}

fn block_name_parameters(parameters: &ruby_prism::ParametersNode<'_>) -> Vec<BlockNameParameter> {
    let mut result = Vec::new();
    for parameter in parameters
        .requireds()
        .iter()
        .chain(parameters.posts().iter())
    {
        if let Some(parameter) = parameter.as_required_parameter_node() {
            push_block_name_parameter(
                &mut result,
                parameter.name().as_slice(),
                parameter.location().start_offset(),
                0,
            );
        }
    }
    for parameter in parameters.optionals().iter() {
        if let Some(parameter) = parameter.as_optional_parameter_node() {
            push_block_name_parameter(
                &mut result,
                parameter.name().as_slice(),
                parameter.name_loc().start_offset(),
                0,
            );
        }
    }
    for parameter in parameters.keywords().iter() {
        let name_and_location = parameter
            .as_required_keyword_parameter_node()
            .map(|parameter| (parameter.name(), parameter.name_loc()))
            .or_else(|| {
                parameter
                    .as_optional_keyword_parameter_node()
                    .map(|parameter| (parameter.name(), parameter.name_loc()))
            });
        if let Some((name, location)) = name_and_location {
            push_block_name_parameter(&mut result, name.as_slice(), location.start_offset(), 0);
        }
    }
    if let Some(parameter) = parameters
        .rest()
        .and_then(|parameter| parameter.as_rest_parameter_node())
    {
        if let Some(name) = parameter.name() {
            push_block_name_parameter(
                &mut result,
                name.as_slice(),
                parameter.location().start_offset(),
                1,
            );
        }
    }
    if let Some(parameter) = parameters
        .keyword_rest()
        .and_then(|parameter| parameter.as_keyword_rest_parameter_node())
    {
        if let Some(name) = parameter.name() {
            push_block_name_parameter(
                &mut result,
                name.as_slice(),
                parameter.location().start_offset(),
                2,
            );
        }
    }
    if let Some(parameter) = parameters.block() {
        if let Some(name) = parameter.name() {
            push_block_name_parameter(
                &mut result,
                name.as_slice(),
                parameter.location().start_offset(),
                0,
            );
        }
    }
    result
}

fn push_block_name_parameter(
    parameters: &mut Vec<BlockNameParameter>,
    name: &[u8],
    start: usize,
    prefix_length: usize,
) {
    let name = String::from_utf8_lossy(name).into_owned();
    parameters.push(BlockNameParameter {
        range: start..start + name.len() + prefix_length,
        name,
    });
}
