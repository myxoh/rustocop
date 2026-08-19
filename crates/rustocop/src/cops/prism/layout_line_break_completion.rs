use super::source_syntax::{matching_delimiter, top_level_elements};
use super::*;

define_cops! {
    FirstArrayElementLineBreak => "Layout/FirstArrayElementLineBreak" => source(first_array_element_line_break),
    FirstHashElementLineBreak => "Layout/FirstHashElementLineBreak" => source(first_hash_element_line_break),
    FirstMethodArgumentLineBreak => "Layout/FirstMethodArgumentLineBreak" => source(first_method_argument_line_break),
    FirstMethodParameterLineBreak => "Layout/FirstMethodParameterLineBreak" => source(first_method_parameter_line_break),
    MultilineHashKeyLineBreaks => "Layout/MultilineHashKeyLineBreaks" => source(multiline_hash_key_line_breaks),
    SingleLineBlockChain => "Layout/SingleLineBlockChain" => source(single_line_block_chain),
    ConditionPosition => "Layout/ConditionPosition" => source(condition_position),
}

fn first_array_element_line_break(context: &mut CopContext<'_, '_>) {
    first_literal_element(context, b'[', b']', "array", |source, opening| {
        opening == 0
            || !matches!(source.as_bytes()[opening - 1], b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b']')
    });
    for marker in ["%w(", "%W(", "%i(", "%I("] {
        for (opening, _) in context.source().match_indices(marker) {
            let delimiter = opening + marker.len() - 1;
            report_percent_first_element(context, delimiter);
        }
    }
    implicit_array_assignment(context);
}

fn first_hash_element_line_break(context: &mut CopContext<'_, '_>) {
    first_literal_element(context, b'{', b'}', "hash", |source, opening| {
        let rest = &source[opening + 1..];
        rest.find('}').is_some_and(|end| {
            let body = &rest[..end];
            body.contains(':') || body.contains("=>")
        })
    });
}

fn first_literal_element(
    context: &mut CopContext<'_, '_>,
    opening_byte: u8,
    closing_byte: u8,
    collection: &str,
    allowed: impl Fn(&str, usize) -> bool,
) {
    let source = context.source();
    for opening in source
        .bytes()
        .enumerate()
        .filter_map(|(offset, byte)| (byte == opening_byte).then_some(offset))
        .collect::<Vec<_>>()
    {
        if allowed(source, opening) {
            report_first_element(context, opening, opening_byte, closing_byte, collection);
        }
    }
}

fn report_first_element(
    context: &mut CopContext<'_, '_>,
    opening: usize,
    opening_byte: u8,
    closing_byte: u8,
    collection: &str,
) {
    let source = context.source();
    let Some(closing) = matching_delimiter(source, opening, opening_byte, closing_byte) else {
        return;
    };
    if !source[opening..=closing].contains('\n') || source[opening + 1..].starts_with('\n') {
        return;
    }
    let elements = top_level_elements(source, opening + 1, closing);
    let Some(first) = elements.first() else {
        return;
    };
    if context.config_bool("AllowMultilineFinalElement", false)
        && elements
            .last()
            .is_some_and(|last| source[last.clone()].contains('\n'))
    {
        return;
    }
    let start =
        first.start + source[first.clone()].len() - source[first.clone()].trim_start().len();
    if source[opening + 1..start].contains('\n') {
        return;
    }
    let end = first.end - (source[first.clone()].len() - source[first.clone()].trim_end().len());
    context.insert(
        format!("Add a line break before the first element of a multi-line {collection}."),
        start..end.max(start),
        start,
        "\n",
    );
}

fn report_percent_first_element(context: &mut CopContext<'_, '_>, opening: usize) {
    let source = context.source();
    let Some(closing) = matching_delimiter(source, opening, b'(', b')') else {
        return;
    };
    if !source[opening..=closing].contains('\n') || source[opening + 1..].starts_with('\n') {
        return;
    }
    context.insert(
        "Add a line break before the first element of a multi-line array.",
        opening + 1..opening + 1,
        opening + 1,
        "\n",
    );
}

fn implicit_array_assignment(context: &mut CopContext<'_, '_>) {
    if context.config_bool("AllowImplicitArrayLiterals", false) {
        return;
    }
    let source = context.source();
    for (line_start, line) in context.source_file().lines() {
        let Some(equal) = line.find("= ") else {
            continue;
        };
        let after = line_start + equal + 2;
        if line[equal + 2..].trim_start().starts_with('[')
            || !source[after..].contains("\n")
            || !line[equal + 2..].contains(',')
        {
            continue;
        }
        let offense = after..after;
        context.insert(
            "Add a line break before the first element of a multi-line array.",
            offense.clone(),
            offense.start,
            "\n",
        );
    }
}

fn first_method_argument_line_break(context: &mut CopContext<'_, '_>) {
    first_parenthesized_list(context, false);
}

fn first_method_parameter_line_break(context: &mut CopContext<'_, '_>) {
    first_parenthesized_list(context, true);
}

fn first_parenthesized_list(context: &mut CopContext<'_, '_>, definition: bool) {
    let source = context.source();
    for opening in source
        .bytes()
        .enumerate()
        .filter_map(|(offset, byte)| (byte == b'(').then_some(offset))
        .collect::<Vec<_>>()
    {
        let line_start = source[..opening].rfind('\n').map_or(0, |at| at + 1);
        let prefix = source[line_start..opening].trim_start();
        if prefix.starts_with("def ") != definition {
            continue;
        }
        if !definition
            && (prefix.is_empty()
                || ["if", "unless", "while", "until"]
                    .iter()
                    .any(|word| prefix.ends_with(word)))
        {
            continue;
        }
        if !definition {
            let method = prefix
                .split(|character: char| {
                    !(character.is_alphanumeric() || matches!(character, '_' | '!' | '?'))
                })
                .next_back()
                .unwrap_or_default();
            if context
                .config_values("AllowedMethods")
                .iter()
                .any(|allowed| allowed == method)
            {
                continue;
            }
        }
        let Some(closing) = matching_delimiter(source, opening, b'(', b')') else {
            continue;
        };
        if !source[opening..=closing].contains('\n') || source[opening + 1..].starts_with('\n') {
            continue;
        }
        let Some(first) = top_level_elements(source, opening + 1, closing)
            .first()
            .cloned()
        else {
            continue;
        };
        let elements = top_level_elements(source, opening + 1, closing);
        if context.config_bool("AllowMultilineFinalElement", false)
            && elements
                .last()
                .is_some_and(|last| source[last.clone()].contains('\n'))
        {
            continue;
        }
        let start =
            first.start + source[first.clone()].len() - source[first.clone()].trim_start().len();
        let end =
            first.end - (source[first.clone()].len() - source[first.clone()].trim_end().len());
        let kind = if definition { "parameter" } else { "argument" };
        let list = if definition {
            "method parameter list"
        } else {
            "method argument list"
        };
        context.insert(
            format!("Add a line break before the first {kind} of a multi-line {list}."),
            start..end.max(start),
            start,
            "\n",
        );
    }
}

fn multiline_hash_key_line_breaks(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for opening in source
        .bytes()
        .enumerate()
        .filter_map(|(offset, byte)| (byte == b'{').then_some(offset))
        .collect::<Vec<_>>()
    {
        let Some(closing) = matching_delimiter(source, opening, b'{', b'}') else {
            continue;
        };
        if !source[opening..=closing].contains('\n') {
            continue;
        }
        let elements = top_level_elements(source, opening + 1, closing);
        let element_lines = elements
            .iter()
            .map(|element| {
                source[..element.start]
                    .bytes()
                    .filter(|byte| *byte == b'\n')
                    .count()
            })
            .collect::<std::collections::BTreeSet<_>>();
        if element_lines.len() <= 1 {
            continue;
        }
        for (index, pair) in elements.windows(2).enumerate() {
            let previous = &pair[0];
            let current = &pair[1];
            if context.config_bool("AllowMultilineFinalElement", false)
                && index + 1 == elements.len() - 1
            {
                continue;
            }
            let start = current.start + source[current.clone()].len()
                - source[current.clone()].trim_start().len();
            if source[previous.end..start].contains('\n') {
                continue;
            }
            let end = current.end
                - (source[current.clone()].len() - source[current.clone()].trim_end().len());
            let mut edits = vec![(start..start, "\n".to_string())];
            if context.config_bool("AllowMultilineFinalElement", false)
                && index + 2 == elements.len() - 1
                && source[current.clone()].contains('\n')
            {
                let final_element = &elements[index + 2];
                let final_start = final_element.start + source[final_element.clone()].len()
                    - source[final_element.clone()].trim_start().len();
                edits.push((final_start..final_start, "\n".to_string()));
            }
            context.replace_many(
                "Each key in a multi-line hash must start on a separate line.",
                start..end,
                edits,
            );
        }
    }
}

fn single_line_block_chain(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for (closing, _) in source.match_indices('}') {
        let tail = &source[closing + 1..];
        let whitespace = tail.len() - tail.trim_start_matches([' ', '\t']).len();
        let start = closing + 1 + whitespace;
        if source[closing + 1..start].contains('\n') {
            continue;
        }
        let rest = &source[start..];
        if rest.starts_with(".\n") {
            continue;
        }
        let length = if let Some(after_operator) = rest.strip_prefix("&.") {
            2 + after_operator
                .bytes()
                .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                .count()
        } else if rest.starts_with(".(") {
            2
        } else if let Some(after_dot) = rest.strip_prefix('.') {
            1 + after_dot
                .bytes()
                .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_' || *byte == b'?')
                .count()
        } else {
            0
        };
        if length > 0 {
            context.insert(
                "Put method call on a separate line if chained to a single line block.",
                start..start + length,
                start,
                "\n",
            );
        }
    }
}

fn condition_position(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for pair in lines.windows(2) {
        let (keyword_start, keyword_line) = pair[0];
        let (condition_line_start, condition_line) = pair[1];
        let keyword = keyword_line.trim();
        if !matches!(keyword, "if" | "unless" | "while" | "until" | "elsif") {
            continue;
        }
        let condition = condition_line.trim();
        if condition.is_empty() {
            continue;
        }
        let start = condition_line_start + condition_line.len() - condition_line.trim_start().len();
        let end = start + condition.len();
        context.replace(
            format!("Place the condition on the same line as `{keyword}`."),
            start..end,
            keyword_start + keyword_line.len()..start,
            " ",
        );
    }
}
