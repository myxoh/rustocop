use super::*;

declare_cops!(TrailingCommaInArrayLiteral, TrailingCommaInHashLiteral);
define_any_node_cop!(TrailingCommaInArrayLiteral => "Style/TrailingCommaInArrayLiteral" => array_trailing_comma);
define_any_node_cop!(TrailingCommaInHashLiteral => "Style/TrailingCommaInHashLiteral" => hash_trailing_comma);

fn array_trailing_comma(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let Some(array) = node.as_array_node() else {
        return;
    };
    let Some(opening) = array.opening_loc() else {
        return;
    };
    if context.source_file().at(&opening) != "[" {
        return;
    }
    check_trailing_comma(
        &array.location(),
        &array
            .elements()
            .iter()
            .map(|element| {
                (
                    element.location().start_offset(),
                    element.location().end_offset(),
                )
            })
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
        &hash
            .elements()
            .iter()
            .map(|element| {
                (
                    element.location().start_offset(),
                    element.location().end_offset(),
                )
            })
            .collect::<Vec<_>>(),
        "hash",
        "Put a comma after the last item of a multiline hash.",
        "Avoid comma after the last item of a hash.",
        context,
    );
}

fn check_trailing_comma(
    location: &ruby_prism::Location<'_>,
    elements: &[(usize, usize)],
    kind: &str,
    missing_message: &str,
    extra_message: &str,
    context: &mut CopContext<'_, '_>,
) {
    let source = context.source_file().at(location);
    if source.len() < 2 {
        return;
    }
    let close = location.end_offset().saturating_sub(1);
    let Some(&(last_start, last_end)) = elements.last() else {
        return;
    };
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
    let close_begins_line = context.source()
        [context.source_file().line_start(close)..close]
        .trim()
        .is_empty();
    let multiline = (close_begins_line || elements.len() != 1) && source.contains('\n');
    let style = context
        .config_value("EnforcedStyleForMultiline")
        .unwrap_or("no_comma");
    let elements_on_separate_lines = elements
        .windows(2)
        .all(|pair| {
            !context.source_file().same_line(
                pair[0].1.saturating_sub(1),
                pair[1].0,
            )
        })
        && !context
            .source_file()
            .same_line(last_end.saturating_sub(1), close);
    let required = match style {
        "comma" => multiline && elements_on_separate_lines,
        "consistent_comma" => multiline,
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
        let last_source = &context.source()[last_start..last_end];
        let final_line = last_source.rfind('\n').map_or(0, |offset| offset + 1);
        let offense_start = last_start
            + final_line
            + last_source[final_line..]
                .bytes()
                .take_while(|byte| byte.is_ascii_whitespace())
                .count();
        context.insert(
            missing_message,
            offense_start..at,
            at,
            ",",
        );
    }
}
