use super::*;
use crate::rubocop::ast::node::core::NodeRef as RubocopNodeRef;
use crate::rubocop::ast::prism::convert as convert_rubocop_ast;

define_cops! {
    ArrayCoercion => "Style/ArrayCoercion" => any_node(array_coercion),
    MultipleComparison => "Style/MultipleComparison" => node(as_or_node, multiple_comparison),
    ExplicitBlockArgument => "Style/ExplicitBlockArgument" => source(explicit_block_argument),
}

fn array_coercion(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(array) = node.as_array_node() {
        if array.opening_loc().is_none_or(|opening| opening.as_slice() != b"[")
            || array.elements().len() != 1
        {
            return;
        }
        let Some(argument) = array
            .elements()
            .iter()
            .next()
            .and_then(|element| element.as_splat_node())
            .and_then(|splat| splat.expression())
        else {
            return;
        };
        let argument_source = context.source_file().node(&argument);
        context.replace(
            format!("Use `Array({argument_source})` instead of `[*{argument_source}]`."),
            array.location(),
            array.location(),
            format!("Array({argument_source})"),
        );
        return;
    }

    let Some(unless_node) = node.as_unless_node() else { return };
    return_if!(unless_node.end_keyword_loc().is_some() || unless_node.else_clause().is_some());
    let Some(predicate) = unless_node.predicate().as_call_node() else { return };
    return_unless!(predicate.name().as_slice() == b"is_a?");
    let Some(checked) = predicate.receiver().and_then(|receiver| receiver.as_local_variable_read_node()) else { return };
    let Some(arguments) = predicate.arguments() else { return };
    let mut arguments = arguments.arguments().iter();
    let Some(array_constant) = arguments.next() else { return };
    return_unless!(arguments.next().is_none() && node_is_root_constant(&array_constant, b"Array"));
    let Some(assignment) = unless_node
        .statements()
        .filter(|statements| statements.body().len() == 1)
        .and_then(|statements| statements.body().first())
        .and_then(|body| body.as_local_variable_write_node())
    else { return };
    let Some(wrapped) = assignment.value().as_array_node() else { return };
    return_unless!(wrapped.opening_loc().is_some() && wrapped.elements().len() == 1);
    let Some(wrapped_variable) = wrapped
        .elements()
        .iter()
        .next()
        .and_then(|element| element.as_local_variable_read_node())
    else { return };
    let name = checked.name().as_slice();
    return_unless!(assignment.name().as_slice() == name && wrapped_variable.name().as_slice() == name);
    let name = String::from_utf8_lossy(name);
    context.replace(
        format!("Use `Array({name})` instead of explicit `Array` check."),
        unless_node.location(),
        unless_node.location(),
        format!("{name} = Array({name})"),
    );
}

fn multiple_comparison(node: &ruby_prism::OrNode<'_>, context: &mut CopContext<'_, '_>) {
    if context.parent().is_some_and(|parent| parent.as_or_node().is_some()) {
        return;
    }
    let threshold = context.config_usize("ComparisonsThreshold", 2);
    let allow_methods = context.config_bool("AllowMethodComparison", true);
    let mut comparisons = Vec::new();
    if !collect_comparisons(
        &node.as_node(),
        allow_methods,
        context.source(),
        &mut comparisons,
    ) {
        return;
    }
    let Some(variable) = comparisons.first().map(|comparison| comparison.0.clone()) else {
        return;
    };
    let retained = comparisons
        .iter()
        .position(|(candidate, _, _, _)| candidate != &variable)
        .unwrap_or(comparisons.len());
    comparisons.truncate(retained);
    if comparisons.len() < threshold {
        return;
    }
    let start = comparisons[0].2;
    let end = comparisons.last().unwrap().3;
    let values = comparisons
        .iter()
        .map(|(_, value, _, _)| value.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    context.replace(
        "Avoid comparing a variable with multiple items in a conditional, use `Array#include?` instead.",
        start..end,
        start..end,
        format!("[{values}].include?({variable})"),
    );
}

fn collect_comparisons(
    node: &Node<'_>,
    allow_methods: bool,
    source: &str,
    comparisons: &mut Vec<(String, String, usize, usize)>,
) -> bool {
    if let Some(or_node) = node.as_or_node() {
        return collect_comparisons(&or_node.left(), allow_methods, source, comparisons)
            && collect_comparisons(&or_node.right(), allow_methods, source, comparisons);
    }
    if !is_equality_comparison(node) {
        return false;
    }
    if node.as_call_node().is_some_and(|call| {
        call.receiver()
            .is_some_and(|receiver| receiver.as_local_variable_read_node().is_some())
            && only_argument(&call)
                .is_some_and(|argument| argument.as_local_variable_read_node().is_some())
    }) {
        return true;
    }
    let Some((variable, value)) = comparison_parts(node, allow_methods) else {
        let call = node.as_call_node().expect("equality comparison is a call");
        let receiver = call.receiver().expect("equality comparison has a receiver");
        let argument = only_argument(&call).expect("equality comparison has one argument");
        return receiver.as_local_variable_read_node().is_some()
            || receiver.as_call_node().is_some()
            || argument.as_local_variable_read_node().is_some()
            || argument.as_call_node().is_some();
    };
    let variable_location = variable.location();
    let value_location = value.location();
    comparisons.push((
        source[variable_location.start_offset()..variable_location.end_offset()].to_string(),
        source[value_location.start_offset()..value_location.end_offset()].to_string(),
        node.location().start_offset(),
        node.location().end_offset(),
    ));
    true
}

fn is_equality_comparison(node: &Node<'_>) -> bool {
    node.as_call_node().is_some_and(|call| {
        call.name().as_slice() == b"=="
            && call.receiver().is_some()
            && call
                .arguments()
                .is_some_and(|arguments| arguments.arguments().len() == 1)
    })
}

fn comparison_parts<'pr>(
    node: &Node<'pr>,
    allow_methods: bool,
) -> Option<(Node<'pr>, Node<'pr>)> {
    let call = node.as_call_node()?;
    let receiver = call.receiver()?;
    let value = call.arguments()?.arguments().iter().next()?;
    if receiver.as_local_variable_read_node().is_some()
        && value.as_local_variable_read_node().is_some()
    {
        return None;
    }
    if comparison_variable(&receiver, allow_methods) {
        if allow_methods && value.as_call_node().is_some() {
            return None;
        }
        Some((receiver, value))
    } else if comparison_variable(&value, allow_methods) {
        if allow_methods && receiver.as_call_node().is_some() {
            return None;
        }
        Some((value, receiver))
    } else {
        None
    }
}

fn comparison_variable(node: &Node<'_>, allow_methods: bool) -> bool {
    node.as_local_variable_read_node().is_some()
        || allow_methods && node.as_call_node().is_some()
}

fn explicit_block_argument(context: &mut CopContext<'_, '_>) {
    let source = context.source().to_owned();
    let parsed = ruby_prism::parse(source.as_bytes());
    let (ast, root) = convert_rubocop_ast(&source, &parsed.node());
    let Some(root) = root.map(|root| ast.node(root)) else {
        return;
    };
    let mut edited_definitions = std::collections::HashSet::new();
    for block in root.each_node(&["block", "numblock", "itblock"]) {
        let (send, arguments, yield_node, offense_range) =
            if block.node_child(0).is_some_and(|node| node.kind() == "args") {
                let Some(send) = block
                    .parent()
                    .filter(|node| matches!(node.kind(), "super" | "zsuper"))
                else {
                    continue;
                };
                let Some(arguments) = block.node_child(0) else {
                    continue;
                };
                let Some(yield_node) = block.node_child(1).filter(|node| node.kind() == "yield") else {
                    continue;
                };
                let (Some(send_range), Some(block_range)) = (send.source_range(), block.source_range()) else {
                    continue;
                };
                (send, arguments, yield_node, send_range.start..block_range.end)
            } else {
                let Some(send) = block.node_child(0) else {
                    continue;
                };
                let Some(arguments) = block.node_child(1).filter(|node| node.kind() == "args") else {
                    continue;
                };
                let Some(yield_node) = block.node_child(2).filter(|node| node.kind() == "yield") else {
                    continue;
                };
                let Some(range) = block.source_range() else {
                    continue;
                };
                (send, arguments, yield_node, range)
            };
        let block_arguments = arguments.child_nodes();
        let yield_arguments = yield_node.child_nodes();
        if block_arguments.len() != yield_arguments.len()
            || block_arguments
                .iter()
                .zip(&yield_arguments)
                .any(|(block_argument, yield_argument)| {
                    block_argument.symbol_child(0).is_none()
                        || block_argument.symbol_child(0) != yield_argument.symbol_child(0)
                })
        {
            continue;
        }
        let Some(definition) = block
            .ancestors()
            .into_iter()
            .find(|node| matches!(node.kind(), "def" | "defs"))
        else {
            continue;
        };
        let definition_arguments = definition
            .child_nodes()
            .into_iter()
            .find(|node| node.kind() == "args");
        let existing_block_argument = definition_arguments
            .into_iter()
            .flat_map(RubocopNodeRef::child_nodes)
            .find(|node| node.kind() == "blockarg");
        let block_name = existing_block_argument
            .and_then(|node| node.symbol_child(0))
            .unwrap_or(if existing_block_argument.is_some() { "" } else { "block" });
        let block_range = explicit_character_range_to_byte(&source, offense_range);
        let Some(mut send_range) = send.source_range() else {
            continue;
        };
        if matches!(send.kind(), "super" | "zsuper") {
            if let Some(block_source) = block.source_range() {
                send_range.end = block_source.start;
            }
        }
        let send_range = explicit_character_range_to_byte(&source, send_range);
        let call = source[send_range.clone()].trim_end();
        let replacement = explicit_forwarding_call(call, send, definition, block_name);
        let mut edits = vec![(block_range.clone(), replacement)];
        if existing_block_argument.is_none() && edited_definitions.insert(definition.id()) {
            if let Some(edit) = explicit_definition_block_edit(&source, definition, block_name) {
                edits.push(edit);
            }
        }
        context.replace_many(
            "Consider using explicit block argument in the surrounding method's signature over `yield`.",
            block_range,
            edits,
        );
    }
}

fn explicit_forwarding_call(
    call: &str,
    send: RubocopNodeRef<'_>,
    definition: RubocopNodeRef<'_>,
    block_name: &str,
) -> String {
    if send.kind() == "zsuper" {
        let arguments = definition
            .child_nodes()
            .into_iter()
            .find(|node| node.kind() == "args")
            .into_iter()
            .flat_map(RubocopNodeRef::child_nodes)
            .filter(|argument| argument.kind() != "blockarg")
            .filter_map(|argument| {
                if matches!(argument.kind(), "optarg" | "kwoptarg") {
                    argument.symbol_child(0).map(str::to_owned)
                } else {
                    argument.source().map(str::to_owned)
                }
            })
            .collect::<Vec<_>>();
        let prefix = if arguments.is_empty() {
            String::new()
        } else {
            format!("{}, ", arguments.join(", "))
        };
        return format!("super({prefix}&{block_name})");
    }
    if let Some(close) = call.rfind(')').filter(|_| send.loc("end").is_some()) {
        let trimmed = call[..close].trim_end();
        let separator = if trimmed.ends_with('(') {
            ""
        } else if trimmed.ends_with(',') {
            " "
        } else {
            ", "
        };
        return format!(
            "{}{separator}&{block_name}{}{}",
            trimmed,
            &call[trimmed.len()..close],
            &call[close..]
        );
    }
    let has_arguments = send
        .loc("selector")
        .zip(send.source_range())
        .is_some_and(|((selector, _), range)| selector.end < range.end)
        || matches!(send.kind(), "super") && call.trim() != "super";
    if has_arguments {
        format!("{call}, &{block_name}")
    } else {
        format!("{call}(&{block_name})")
    }
}

fn explicit_definition_block_edit(
    source: &str,
    definition: RubocopNodeRef<'_>,
    block_name: &str,
) -> Option<(std::ops::Range<usize>, String)> {
    let arguments = definition
        .child_nodes()
        .into_iter()
        .find(|node| node.kind() == "args")?;
    if let Some((closing, _)) = arguments.loc("end") {
        let at = explicit_character_to_byte(source, closing.start);
        let empty = arguments.child_nodes().is_empty();
        return Some((at..at, format!("{}&{block_name}", if empty { "" } else { ", " })));
    }
    let (name, _) = definition.loc("name")?;
    let at = explicit_character_to_byte(source, name.end);
    Some((at..at, format!("(&{block_name})")))
}

fn explicit_character_range_to_byte(source: &str, range: std::ops::Range<usize>) -> std::ops::Range<usize> {
    explicit_character_to_byte(source, range.start)..explicit_character_to_byte(source, range.end)
}

fn explicit_character_to_byte(source: &str, offset: usize) -> usize {
    source.char_indices().nth(offset).map_or(source.len(), |(byte, _)| byte)
}
