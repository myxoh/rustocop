use ruby_prism::InterpolatedStringNode;

use super::*;

define_cops! {
    ImplicitStringConcatenation => "Lint/ImplicitStringConcatenation" => node(as_interpolated_string_node, implicit_string_concatenation),
    RedundantInterpolation => "Style/RedundantInterpolation" => node(as_interpolated_string_node, redundant_interpolation),
    RedundantInterpolationUnfreeze => "Style/RedundantInterpolationUnfreeze" => call(redundant_interpolation_unfreeze),
    StringLiteralsInInterpolation => "Style/StringLiteralsInInterpolation" => node(as_string_node, string_literals_in_interpolation),
}

fn implicit_string_concatenation(
    node: &InterpolatedStringNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    if node.opening_loc().is_some() {
        return;
    }
    let parts = node.parts().iter().collect::<Vec<_>>();
    for pair in parts.windows(2) {
        let (left, right) = (&pair[0], &pair[1]);
        if !literal_part(left) || !literal_part(right) {
            continue;
        }
        let left_end = left.location().end_offset();
        let right_start = right.location().start_offset();
        if context.source()[left_end..right_start].contains('\n')
            || !ends_with_literal_delimiter(left, context.source_file())
        {
            continue;
        }

        let mut message = format!(
            "Combine {} and {} into a single string literal, rather than using implicit string concatenation.",
            display_string(left, context.source_file()),
            display_string(right, context.source_file())
        );
        if context
            .parent()
            .is_some_and(|parent| parent.as_array_node().is_some())
        {
            message.push_str(
                " Or, if they were intended to be separate array elements, separate them with a comma.",
            );
        } else if context
            .parent()
            .is_some_and(|parent| parent.as_call_node().is_some())
        {
            message.push_str(
                " Or, if they were intended to be separate method arguments, separate them with a comma.",
            );
        }

        let offense = left.location().start_offset()..right.location().end_offset();
        if empty_string(left) {
            context.remove(message, offense, left.location());
        } else if empty_string(right) {
            context.remove(message, offense, right.location());
        } else {
            context.replace(message, offense, left_end..right_start, " + ");
        }
    }
}

fn literal_part(node: &Node<'_>) -> bool {
    node.as_string_node().is_some()
        || node
            .as_interpolated_string_node()
            .is_some_and(|string| string.opening_loc().is_some())
}

fn ends_with_literal_delimiter(node: &Node<'_>, file: SourceFile<'_>) -> bool {
    let source = file.node(node).as_bytes();
    source.first().is_some_and(|first| {
        matches!(first, b'\'' | b'"') && source.last().is_some_and(|last| last == first)
    })
}

fn empty_string(node: &Node<'_>) -> bool {
    node.as_string_node()
        .is_some_and(|string| string.unescaped().is_empty())
}

fn display_string(node: &Node<'_>, file: SourceFile<'_>) -> String {
    let source = file.node(node);
    if !source.contains('\n') {
        return source.to_string();
    }
    let mut value = Vec::new();
    append_string_content(node, &mut value);
    ruby_inspect_string(&String::from_utf8_lossy(&value))
}

fn append_string_content(node: &Node<'_>, output: &mut Vec<u8>) {
    if let Some(string) = node.as_string_node() {
        output.extend_from_slice(string.unescaped());
    } else if let Some(string) = node.as_interpolated_string_node() {
        for part in string.parts().iter() {
            append_string_content(&part, output);
        }
    }
}

fn ruby_inspect_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{escaped}\"")
}

fn string_literals_in_interpolation(
    node: &ruby_prism::StringNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    if !context
        .ancestors()
        .iter()
        .any(|ancestor| ancestor.as_embedded_statements_node().is_some())
    {
        return;
    }
    let (Some(opening), Some(closing)) = (node.opening_loc(), node.closing_loc()) else {
        return;
    };
    let style = context.policy().enforced_style("single_quotes");
    let (current, preferred, description) = if style == "single_quotes" {
        (b'"', '\'', "single-quoted")
    } else if style == "double_quotes" {
        (b'\'', '"', "double-quoted")
    } else {
        return;
    };
    if opening.as_slice() != [current] || closing.as_slice() != [current] {
        return;
    }
    context.replace_many(
        format!("Prefer {description} strings inside interpolations."),
        node.location(),
        vec![
            (
                opening.start_offset()..opening.end_offset(),
                preferred.to_string(),
            ),
            (
                closing.start_offset()..closing.end_offset(),
                preferred.to_string(),
            ),
        ],
    );
}

fn redundant_interpolation_unfreeze(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(3, 0) {
        return;
    }
    const MESSAGE: &str = "Don't unfreeze interpolated strings as they are already unfrozen.";
    if call_name(node) == b"new" && root_constant(node.receiver(), b"String") {
        let Some(argument) = only_argument(node) else {
            return;
        };
        if argument.as_interpolated_string_node().is_none() {
            return;
        }
        let Some(selector) = node.message_loc() else {
            return;
        };
        let offense = node.location().start_offset()..selector.end_offset();
        context.replace(
            MESSAGE,
            offense,
            node.location(),
            context.source_file().node(&argument),
        );
        return;
    }
    if !matches!(call_name(node), b"+@" | b"dup") || argument_count(node) != 0 {
        return;
    }
    let Some(receiver) = node.receiver() else {
        return;
    };
    let Some(interpolated) = receiver.as_interpolated_string_node() else {
        return;
    };
    if !interpolated.parts().iter().any(|part| {
        part.as_embedded_variable_node().is_some() || part.as_embedded_statements_node().is_some()
    }) {
        return;
    }
    if interpolated.opening_loc().is_some_and(|opening| {
        let opening = context.source_file().at(&opening);
        opening.starts_with("<<") && (opening.contains('\'') || opening.contains('"'))
    }) {
        return;
    }
    let Some(selector) = node.message_loc() else {
        return;
    };
    let edit = if selector.start_offset() < receiver.location().start_offset() {
        selector.start_offset()..selector.end_offset()
    } else {
        receiver.location().end_offset()..node.location().end_offset()
    };
    context.remove(MESSAGE, &selector, edit);
}

fn redundant_interpolation(node: &InterpolatedStringNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.parts().len() != 1
        || context.ancestors().iter().rev().any(|ancestor| {
            ancestor.as_array_node().is_some_and(|array| {
                context
                    .source_file()
                    .node(&array.as_node())
                    .starts_with("%W")
            })
        })
        || context.parent().is_some_and(|parent| {
            parent.as_interpolated_string_node().is_some() || parent.as_string_node().is_some()
        })
    {
        return;
    }
    let Some(part) = node.parts().first() else {
        return;
    };
    let (expression, statement_count) = if let Some(embedded) = part.as_embedded_variable_node() {
        (embedded.variable(), 1)
    } else if let Some(embedded) = part.as_embedded_statements_node() {
        let Some(statements) = embedded.statements() else {
            return;
        };
        let count = statements.body().len();
        if context
            .source_file()
            .node(&statements.as_node())
            .contains(" => ")
        {
            return;
        }
        if statements.body().is_empty() {
            return;
        }
        (statements.as_node(), count)
    } else {
        return;
    };
    let mut rendered = if statement_count == 1 {
        if let Some(statements) = expression.as_statements_node() {
            let Some(value) = statements.body().first() else {
                return;
            };
            render_interpolated_value(&value, context.source_file())
        } else {
            render_interpolated_value(&expression, context.source_file())
        }
    } else {
        context.source_file().node(&expression).to_string()
    };
    if statement_count > 1 || needs_parentheses(&expression) {
        rendered = format!("({rendered})");
    }
    rendered.push_str(".to_s");
    context.replace(
        "Prefer `to_s` over string interpolation.",
        node.location(),
        node.location(),
        rendered,
    );
}

fn render_interpolated_value(node: &Node<'_>, file: SourceFile<'_>) -> String {
    let Some(call) = node.as_call_node() else {
        return file.node(node).to_string();
    };
    if call.opening_loc().is_some() || argument_count(&call) == 0 || !identifier(call_name(&call)) {
        return file.node(node).to_string();
    }
    let receiver = call
        .receiver()
        .map(|receiver| {
            let operator = call
                .call_operator_loc()
                .map_or(".", |operator| file.at(&operator));
            format!("{}{operator}", file.node(&receiver))
        })
        .unwrap_or_default();
    let arguments = call
        .arguments()
        .expect("argument count checked")
        .arguments()
        .iter()
        .map(|argument| file.node(&argument))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{receiver}{}({arguments})",
        String::from_utf8_lossy(call_name(&call))
    )
}

fn needs_parentheses(node: &Node<'_>) -> bool {
    node.as_statements_node().is_some_and(|statements| {
        statements.body().len() != 1
            || statements
                .body()
                .first()
                .is_some_and(|value| needs_parentheses(&value))
    }) || node.as_and_node().is_some()
        || node.as_or_node().is_some()
        || node.as_match_predicate_node().is_some()
        || node.as_call_node().is_some_and(|call| {
            matches!(call_name(&call), b"+" | b"-" | b"*" | b"/" | b"%" | b"**")
        })
}

fn identifier(name: &[u8]) -> bool {
    name.first()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
        && name
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
}
