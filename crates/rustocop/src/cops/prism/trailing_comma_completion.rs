use super::*;

declare_cops!(TrailingCommaInArrayLiteral, TrailingCommaInHashLiteral);
define_any_node_cop!(TrailingCommaInArrayLiteral => "Style/TrailingCommaInArrayLiteral" => array_trailing_comma);
define_any_node_cop!(TrailingCommaInHashLiteral => "Style/TrailingCommaInHashLiteral" => hash_trailing_comma);

fn array_trailing_comma(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let Some(array) = node.as_array_node() else {
        return;
    };
    check_trailing_comma(
        &array.location(),
        array.elements().last().map(|element| {
            (
                element.location().start_offset(),
                element.location().end_offset(),
            )
        }),
        &array
            .elements()
            .iter()
            .map(|element| element.location().start_offset())
            .collect::<Vec<_>>(),
        "array",
        "Put a comma after the last item of a multiline array.",
        "Avoid comma after the last item of an array.",
        context,
    );
}

fn hash_trailing_comma(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let Some(hash) = node.as_hash_node() else {
        return;
    };
    check_trailing_comma(
        &hash.location(),
        hash.elements().last().map(|element| {
            (
                element.location().start_offset(),
                element.location().end_offset(),
            )
        }),
        &hash
            .elements()
            .iter()
            .map(|element| element.location().start_offset())
            .collect::<Vec<_>>(),
        "hash",
        "Put a comma after the last item of a multiline hash.",
        "Avoid comma after the last item of a hash.",
        context,
    );
}

fn check_trailing_comma(
    location: &ruby_prism::Location<'_>,
    last_element: Option<(usize, usize)>,
    element_starts: &[usize],
    kind: &str,
    missing_message: &str,
    extra_message: &str,
    context: &mut CopContext<'_, '_>,
) {
    let source = context.source_file().at(location);
    if source.len() < 2 {
        return;
    }
    if ["%w", "%W", "%i", "%I"]
        .iter()
        .any(|prefix| source.starts_with(prefix))
    {
        return;
    }
    let close = location.end_offset().saturating_sub(1);
    let Some((last_start, last_end)) = last_element else {
        return;
    };
    if last_end > close {
        return;
    }
    let tail = &context.source()[last_end..close];
    let leading = tail
        .bytes()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count();
    let comma_at = last_end + leading;
    let mut comma = context.source().as_bytes().get(comma_at) == Some(&b',');
    if comma
        && context.source()[location.start_offset()..last_end].contains("<<-")
        && context.source_file().line(comma_at).trim() == ","
    {
        comma = false;
    }
    let multiline = source.contains('\n');
    let style = context
        .config_value("EnforcedStyleForMultiline")
        .unwrap_or("no_comma");
    let item_lines = element_starts
        .iter()
        .map(|start| context.source_file().line_start(*start))
        .collect::<std::collections::HashSet<_>>();
    let required = match style {
        "comma" => tail.contains('\n') && multiline && item_lines.len() == element_starts.len(),
        "consistent_comma" => {
            item_lines.len() > 1
                || element_starts.first().is_some_and(|first| {
                    context.source_file().line_start(*first)
                        != context.source_file().line_start(location.start_offset())
                })
        }
        "diff_comma" => {
            let after_item = if comma {
                &tail[leading + 1..]
            } else {
                &tail[leading..]
            };
            let rest_of_line = after_item.split('\n').next().unwrap_or(after_item).trim();
            after_item.contains('\n') && (rest_of_line.is_empty() || rest_of_line.starts_with('#'))
        }
        _ => false,
    };
    if comma && !required {
        let at = comma_at;
        let message = if style == "comma" {
            format!("Avoid comma after the last item of {article}{kind}, unless each item is on its own line.", article = if kind == "array" { "an " } else { "a " })
        } else if style == "diff_comma" {
            format!("Avoid comma after the last item of {article}{kind}, unless that item immediately precedes a newline.", article = if kind == "array" { "an " } else { "a " })
        } else if style == "consistent_comma" {
            format!("Avoid comma after the last item of {article}{kind}, unless items are split onto multiple lines.", article = if kind == "array" { "an " } else { "a " })
        } else {
            extra_message.to_string()
        };
        context.remove(message, at..at + 1, at..at + 1);
    } else if !comma && required {
        let at = last_end;
        context.insert(
            missing_message,
            last_start..at,
            at,
            ",",
        );
    }
}
