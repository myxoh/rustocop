use ruby_prism::{BlockNode, CallNode, Node};

use super::*;

define_cops! {
    HashTransformKeys => "Style/HashTransformKeys" => rubocop_callbacks(HashTransformKeysRule, [on_block, on_send]),
    HashTransformValues => "Style/HashTransformValues" => rubocop_callbacks(HashTransformValuesRule, [on_block, on_send]),
}

#[derive(Clone, Copy)]
enum TransformKind {
    Keys,
    Values,
}

impl TransformKind {
    fn method(self) -> &'static str {
        match self {
            Self::Keys => "transform_keys",
            Self::Values => "transform_values",
        }
    }

    fn supported(self, context: &CopContext<'_, '_>) -> bool {
        match self {
            Self::Keys => context.target_ruby_version().at_least(2, 5),
            Self::Values => context.target_ruby_version().at_least(2, 4),
        }
    }
}

impl HashTransformKeysRule<'_, '_, '_> {
    fn on_block(&mut self, block: &BlockNode<'_>) {
        check_direct_block(self, block, TransformKind::Keys);
    }

    fn on_send(&mut self, node: &CallNode<'_>) {
        check_wrapped_map(self, node, TransformKind::Keys);
    }
}

impl HashTransformValuesRule<'_, '_, '_> {
    fn on_block(&mut self, block: &BlockNode<'_>) {
        check_direct_block(self, block, TransformKind::Values);
    }

    fn on_send(&mut self, node: &CallNode<'_>) {
        check_wrapped_map(self, node, TransformKind::Values);
    }
}

fn check_direct_block(
    context: &mut CopContext<'_, '_>,
    block: &BlockNode<'_>,
    kind: TransformKind,
) {
    return_if!(!kind.supported(context));
    let Some(call) = context.parent().and_then(Node::as_call_node) else { return };
    let method = call.name().as_slice();
    if method == b"each_with_object" {
        check_each_with_object(context, &call, block, kind);
    } else if method == b"to_h" && context.target_ruby_version().at_least(2, 6) {
        check_to_h_block(context, &call, block, kind);
    }
}

fn check_each_with_object(
    context: &mut CopContext<'_, '_>,
    call: &CallNode<'_>,
    block: &BlockNode<'_>,
    kind: TransformKind,
) {
    return_if!(!kind.supported(context));
    let Some(receiver) = call.receiver() else { return };
    return_if!(!known_hash_receiver(&receiver));
    let Some(argument) = only_argument(call) else { return };
    return_unless!(argument.as_hash_node().is_some_and(|hash| hash.elements().is_empty()));
    let Some(names) = block_parameter_names(block, context.source_file()) else { return };
    let [key, value, memo] = names.as_slice() else { return };
    let Some(assignment) = block.body().and_then(single_expression).and_then(|body| body.as_call_node()) else { return };
    return_unless!(assignment.name().as_slice() == b"[]=");
    let Some(assignment_receiver) = assignment.receiver() else { return };
    return_unless!(context.source_file().node(&assignment_receiver) == memo);
    let arguments = call_arguments(&assignment);
    let [new_key, new_value] = arguments.as_slice() else { return };
    let (transformed, unchanged, transformed_name, unchanged_name) = match kind {
        TransformKind::Keys => (new_key, new_value, key, value),
        TransformKind::Values => (new_value, new_key, value, key),
    };
    let Some(transformation) = valid_transformation(
        transformed,
        unchanged,
        transformed_name,
        unchanged_name,
        context.source_file(),
    ) else { return };
    return_if!(contains_word(&transformation, memo));
    let Some(selector) = call.message_loc() else { return };
    let call_end = call.closing_loc().map_or(call.location().end_offset(), |location| location.end_offset());
    let parameters = block.parameters().expect("captured block parameters");
    let offense = call.location().start_offset()..block.closing_loc().end_offset();
    let edits = vec![
        (selector.start_offset()..call_end, kind.method().to_string()),
        (
            parameters.location().start_offset()..parameters.location().end_offset(),
            format!("|{transformed_name}|"),
        ),
        (
            assignment.location().start_offset()..assignment.location().end_offset(),
            transformation,
        ),
    ];
    register_transform(context, offense, edits, kind, "each_with_object");
}

fn check_to_h_block(
    context: &mut CopContext<'_, '_>,
    call: &CallNode<'_>,
    block: &BlockNode<'_>,
    kind: TransformKind,
) {
    return_if!(!kind.supported(context));
    let Some(receiver) = call.receiver() else { return };
    return_if!(!known_hash_receiver(&receiver) || argument_count(call) != 0);
    let Some((transformed_name, transformation, body_range, parameter_range)) =
        block_array_transformation(block, kind, context.source_file())
    else { return };
    let Some(selector) = call.message_loc() else { return };
    let offense = call.location().start_offset()..block.closing_loc().end_offset();
    let edits = vec![
        (
            selector.start_offset()..selector.end_offset(),
            kind.method().to_string(),
        ),
        (parameter_range, format!("|{transformed_name}|")),
        (body_range, transformation),
    ];
    register_transform(context, offense, edits, kind, "to_h {...}");
}

fn check_wrapped_map(
    context: &mut CopContext<'_, '_>,
    node: &CallNode<'_>,
    kind: TransformKind,
) {
    return_if!(!kind.supported(context));
    let (map, block, wrapper, description, offense_end) = if node.name().as_slice() == b"to_h" {
        let Some(map) = node.receiver().and_then(|receiver| receiver.as_call_node()) else { return };
        return_unless!(matches!(map.name().as_slice(), b"map" | b"collect"));
        let Some(block) = map.block().and_then(|block| block.as_block_node()) else { return };
        let outer_block = node.block().and_then(|block| block.as_block_node());
        let call_end = outer_block.as_ref().map_or(node.location().end_offset(), |block| {
            call_end_before_block(block, context.source())
        });
        (
            map,
            block,
            Wrapper::ToH {
                end: call_end,
                preserve: outer_block.is_some(),
            },
            "map {...}.to_h",
            call_end,
        )
    } else if node.name().as_slice() == b"[]"
        && node.receiver().is_some_and(|receiver| context.source_file().node(&receiver).trim_start_matches("::") == "Hash")
    {
        let Some(map) = only_argument(node).and_then(|argument| argument.as_call_node()) else { return };
        return_unless!(matches!(map.name().as_slice(), b"map" | b"collect"));
        let Some(block) = map.block().and_then(|block| block.as_block_node()) else { return };
        (
            map,
            block,
            Wrapper::HashBrackets {
                start: node.location().start_offset(),
                end: node.location().end_offset(),
            },
            "Hash[_.map {...}]",
            node.location().end_offset(),
        )
    } else {
        return;
    };
    let Some(receiver) = map.receiver() else { return };
    return_if!(!known_hash_receiver(&receiver));
    let Some((transformed_name, transformation, body_range, parameter_range)) =
        block_array_transformation(&block, kind, context.source_file())
    else { return };
    let Some(selector) = map.message_loc() else { return };
    let mut edits = vec![
        (
            selector.start_offset()..selector.end_offset(),
            kind.method().to_string(),
        ),
        (parameter_range, format!("|{transformed_name}|")),
        (body_range, transformation),
    ];
    match wrapper {
        Wrapper::ToH { end, preserve } => {
            if !preserve {
                edits.push((
                    block.closing_loc().end_offset()..end,
                    String::new(),
                ));
            }
        }
        Wrapper::HashBrackets { start, end } => {
            edits.push((
                start..map.location().start_offset(),
                String::new(),
            ));
            edits.push((
                block.closing_loc().end_offset()..end,
                String::new(),
            ));
        }
    }
    register_transform(
        context,
        node.location().start_offset()..offense_end,
        edits,
        kind,
        description,
    );
}

enum Wrapper {
    ToH { end: usize, preserve: bool },
    HashBrackets { start: usize, end: usize },
}

fn block_array_transformation(
    block: &BlockNode<'_>,
    kind: TransformKind,
    file: SourceFile<'_>,
) -> Option<(String, String, std::ops::Range<usize>, std::ops::Range<usize>)> {
    let names = block_parameter_names(block, file)?;
    let [key, value] = names.as_slice() else { return None };
    let body = block.body().and_then(single_expression)?;
    let array = body.as_array_node()?;
    let values = array.elements().iter().collect::<Vec<_>>();
    let [new_key, new_value] = values.as_slice() else { return None };
    let (transformed, unchanged, transformed_name, unchanged_name) = match kind {
        TransformKind::Keys => (new_key, new_value, key, value),
        TransformKind::Values => (new_value, new_key, value, key),
    };
    let mut transformation = valid_transformation(
        transformed,
        unchanged,
        transformed_name,
        unchanged_name,
        file,
    )?;
    if transformed.as_keyword_hash_node().is_some()
        && !transformation.trim_start().starts_with('{')
    {
        transformation = format!("{{ {transformation} }}");
    }
    let parameters = block.parameters()?;
    Some((
        transformed_name.clone(),
        transformation,
        array.location().start_offset()..array.location().end_offset(),
        parameters.location().start_offset()..parameters.location().end_offset(),
    ))
}

fn valid_transformation(
    transformed: &Node<'_>,
    unchanged: &Node<'_>,
    transformed_name: &str,
    unchanged_name: &str,
    file: SourceFile<'_>,
) -> Option<String> {
    let transformed_source = file.node(transformed);
    if file.node(unchanged) != unchanged_name
        || transformed_source == transformed_name
        || !contains_word(transformed_source, transformed_name)
        || contains_word(transformed_source, unchanged_name)
    {
        return None;
    }
    Some(transformed_source.to_string())
}

fn known_hash_receiver(node: &Node<'_>) -> bool {
    if node.as_hash_node().is_some() {
        return true;
    }
    node.as_call_node().is_some_and(|call| {
        matches!(
            call.name().as_slice(),
            b"to_h"
                | b"to_hash"
                | b"merge"
                | b"merge!"
                | b"update"
                | b"invert"
                | b"except"
                | b"tally"
                | b"group_by"
                | b"transform_keys"
                | b"transform_keys!"
                | b"transform_values"
                | b"transform_values!"
                | b"each_with_object"
        )
    })
}

fn call_arguments<'pr>(node: &CallNode<'pr>) -> Vec<Node<'pr>> {
    node.arguments()
        .map(|arguments| arguments.arguments().iter().collect())
        .unwrap_or_default()
}

fn block_parameter_names(block: &BlockNode<'_>, file: SourceFile<'_>) -> Option<Vec<String>> {
    let parameters = block.parameters()?;
    let source = file
        .slice(parameters.location().start_offset()..parameters.location().end_offset())?
        .trim()
        .trim_matches('|')
        .trim();
    if let Some(rest) = source.strip_prefix('(') {
        let close = rest.find(')')?;
        let mut names = rest[..close]
            .split(',')
            .map(|name| name.trim().to_string())
            .collect::<Vec<_>>();
        names.extend(
            rest[close + 1..]
                .trim_start_matches(',')
                .split(',')
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_string),
        );
        return Some(names);
    }
    Some(source.split(',').map(|name| name.trim().to_string()).collect())
}

fn contains_word(source: &str, name: &str) -> bool {
    source
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .any(|word| word == name)
}

fn call_end_before_block(block: &BlockNode<'_>, source: &str) -> usize {
    let mut end = block.location().start_offset();
    while end > 0 && matches!(source.as_bytes()[end - 1], b' ' | b'\t') {
        end -= 1;
    }
    end
}

fn register_transform(
    context: &mut CopContext<'_, '_>,
    offense: std::ops::Range<usize>,
    edits: Vec<(std::ops::Range<usize>, String)>,
    kind: TransformKind,
    description: &str,
) {
    let message = format!("Prefer `{}` over `{description}`.", kind.method());
    add_offense!(context, offense, message: message, |corrector| {
        for (range, replacement) in edits {
            corrector.replace(range, replacement);
        }
    });
}
