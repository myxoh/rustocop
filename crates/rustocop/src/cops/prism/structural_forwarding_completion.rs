use super::*;

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
    if comparisons.len() < threshold {
        return;
    }
    let variable = &comparisons[0].0;
    if comparisons
        .iter()
        .any(|(candidate, _, _, _)| candidate != variable)
    {
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
    if let Some((variable, value)) = comparison_parts(node, allow_methods) {
        let variable_location = variable.location();
        let value_location = value.location();
        comparisons.push((
            source[variable_location.start_offset()..variable_location.end_offset()].to_string(),
            source[value_location.start_offset()..value_location.end_offset()].to_string(),
            node.location().start_offset(),
            node.location().end_offset(),
        ));
    }
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
    let source = context.source();
    for (def_offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("def ") {
            continue;
        }
        let Some(relative_end) = source[def_offset..].find("\nend") else {
            continue;
        };
        let method_end = def_offset + relative_end + 4;
        let body = &source[def_offset..method_end];
        if body
            .lines()
            .skip(1)
            .any(|nested| nested.trim_start().starts_with("def "))
        {
            continue;
        }
        if explicit_inline_blocks(context, def_offset, line, method_end) {
            continue;
        }
        if let Some(block_relative) = body.find(" { |") {
            let block_start = def_offset + block_relative;
            let Some(pipe_relative) = source[block_start + 4..].find('|') else {
                continue;
            };
            let pipe = block_start + 4 + pipe_relative;
            let args = source[block_start + 4..pipe].trim();
            let Some(close_relative) = source[pipe + 1..method_end].find('}') else {
                continue;
            };
            let close = pipe + 1 + close_relative;
            if source[pipe + 1..close].trim() != format!("yield {args}") {
                continue;
            }
            let call_start = source[..block_start]
                .rfind('\n')
                .map_or(def_offset, |at| at + 1);
            let indent = &source[call_start
                ..call_start + source[call_start..].len()
                    - source[call_start..].trim_start().len()];
            let call = source[call_start..block_start].trim();
            let replacement_call = if let Some(call) = call.strip_suffix(')') {
                format!("{}{call}, &block)", indent)
            } else {
                format!("{indent}{call}(&block)")
            };
            let signature_insert = if let Some(close) = line.rfind(')') {
                def_offset + close
            } else {
                def_offset + line.len()
            };
            let signature_text = if line.contains('(') {
                ", &block"
            } else {
                "(&block)"
            };
            context.replace_many(
                "Consider using explicit block argument in the surrounding method's signature over `yield`.",
                call_start + indent.len()..close + 1,
                vec![
                    (call_start..close + 1, replacement_call),
                    (signature_insert..signature_insert, signature_text.to_string()),
                ],
            );
            continue;
        }
        let Some(block_start_relative) = body.find(" do |") else {
            continue;
        };
        let block_start = def_offset + block_start_relative;
        let Some(args_end_relative) = source[block_start + 5..].find('|') else {
            continue;
        };
        let args_end = block_start + 5 + args_end_relative;
        let args = source[block_start + 5..args_end].trim();
        let Some(block_end_relative) = source[args_end + 1..method_end].find("\n  end") else {
            continue;
        };
        let block_end = args_end + 1 + block_end_relative;
        let block_body = source[args_end + 1..block_end].trim();
        if block_body != format!("yield {args}") {
            continue;
        }
        let call_start = source[..block_start]
            .rfind('\n')
            .map_or(def_offset, |at| at + 1);
        let indent_len = source[call_start..].len() - source[call_start..].trim_start().len();
        let indent = &source[call_start..call_start + indent_len];
        let call = source[call_start..block_start].trim();
        let replacement_call = format!("{indent}{call}(&block)");
        let signature_insert = if let Some(close) = line.rfind(')') {
            def_offset + close
        } else {
            def_offset + line.len()
        };
        let signature_text = if line.contains('(') {
            ", &block"
        } else {
            "(&block)"
        };
        context.replace_many(
            "Consider using explicit block argument in the surrounding method's signature over `yield`.",
            call_start + indent_len..block_end + 6,
            vec![
                (call_start..block_end + 6, replacement_call),
                (signature_insert..signature_insert, signature_text.to_string()),
            ],
        );
    }
}

fn explicit_inline_blocks(
    context: &mut CopContext<'_, '_>,
    def_offset: usize,
    signature: &str,
    method_end: usize,
) -> bool {
    let existing_block = signature
        .split('&')
        .nth(1)
        .map(|tail| {
            tail.bytes()
                .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                .map(char::from)
                .collect::<String>()
        })
        .filter(|name| !name.is_empty());
    let block_name = existing_block.as_deref().unwrap_or("block");
    let mut candidates = Vec::<(std::ops::Range<usize>, String)>::new();
    for (line_offset, line) in context.source_file().lines() {
        if line_offset <= def_offset || line_offset >= method_end {
            continue;
        }
        let Some(open) = line.find(" {") else {
            continue;
        };
        let Some(close) = line.rfind('}') else {
            continue;
        };
        if close <= open + 2 {
            continue;
        }
        let block = line[open + 2..close].trim();
        let forwards = if let Some(parameters) = block.strip_prefix('|') {
            let Some((parameters, body)) = parameters.split_once('|') else {
                continue;
            };
            body.trim() == format!("yield {}", parameters.trim())
        } else {
            block == "yield"
        };
        if !forwards {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let call = line[indent..open].trim_end();
        let replacement = if call == "super" && signature.contains('(') {
            let parameters = signature
                .split_once('(')
                .and_then(|(_, rest)| rest.rsplit_once(')'))
                .map(|(parameters, _)| {
                    parameters
                        .split(',')
                        .map(str::trim)
                        .filter(|parameter| !parameter.is_empty())
                        .map(|parameter| {
                            parameter
                                .split_once('=')
                                .map_or(parameter, |(name, _)| name.trim())
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            let separator = if parameters.is_empty() { "" } else { ", " };
            format!("super({parameters}{separator}&{block_name})")
        } else if let Some(call) = call.strip_suffix(')') {
            let inner = call.trim_end_matches(',');
            let separator = if inner.ends_with('(') { "" } else { ", " };
            format!("{inner}{separator}&{block_name})")
        } else {
            format!("{}(&{block_name})", call.trim_end_matches(','))
        };
        candidates.push((line_offset + indent..line_offset + close + 1, replacement));
    }
    if candidates.is_empty() {
        return false;
    }
    let signature_insert = if let Some(close) = signature.rfind(')') {
        def_offset + close
    } else {
        def_offset + signature.len()
    };
    let signature_text = if signature.trim_end().ends_with("()") {
        "&block"
    } else if signature.contains('(') {
        ", &block"
    } else {
        "(&block)"
    };
    let mut edits = vec![candidates[0].clone()];
    if existing_block.is_none() {
        edits.push((
            signature_insert..signature_insert,
            signature_text.to_string(),
        ));
    }
    let message = "Consider using explicit block argument in the surrounding method's signature over `yield`.";
    context.replace_many(message, candidates[0].0.clone(), edits);
    for (range, replacement) in candidates.iter().skip(1) {
        context.replace(message, range.clone(), range.clone(), replacement);
    }
    true
}
