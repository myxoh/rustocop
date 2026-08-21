use super::*;

define_cops! {
    PercentStringArray => "Lint/PercentStringArray" => node(as_array_node, percent_string_array),
    PercentSymbolArray => "Lint/PercentSymbolArray" => node(as_array_node, percent_symbol_array),
    RedundantPercentQ => "Style/RedundantPercentQ" => any_node(redundant_percent_q),
}

fn percent_symbol_array(node: &ruby_prism::ArrayNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(opening) = node.opening_loc() else {
        return;
    };
    if !matches!(opening.as_slice(), bytes if bytes.starts_with(b"%i") || bytes.starts_with(b"%I"))
    {
        return;
    }
    let source = context.source_file().node(&node.as_node());
    let opening_len = opening.as_slice().len();
    if source.len() <= opening_len + 1 {
        return;
    }
    let content = &source[opening_len..source.len() - 1];
    let bytes = content.as_bytes();
    let unwanted = bytes.iter().enumerate().any(|(index, byte)| {
        *byte == b':' && (index == 0 || bytes[index - 1].is_ascii_whitespace())
            || *byte == b',' && index > 0 && bytes[index - 1] != b'$'
    });
    if !unwanted {
        return;
    }
    let mut clean = String::with_capacity(content.len());
    for (index, character) in content.char_indices() {
        let previous = index
            .checked_sub(1)
            .and_then(|at| content.as_bytes().get(at));
        if character == ':' && (index == 0 || previous.is_some_and(u8::is_ascii_whitespace))
            || character == ',' && previous != Some(&b'$')
        {
            continue;
        }
        clean.push(character);
    }
    let replacement = format!(
        "{}{}{}",
        context.source_file().at(&opening),
        clean,
        &source[source.len() - 1..]
    );
    context.replace(
        "Within `%i`/`%I`, ':' and ',' are unnecessary and may be unwanted in the resulting symbols.",
        node.location(),
        node.location(),
        replacement,
    );
}

fn percent_string_array(node: &ruby_prism::ArrayNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(opening) = node.opening_loc() else {
        return;
    };
    if !matches!(opening.as_slice(), bytes if bytes.starts_with(b"%w") || bytes.starts_with(b"%W"))
    {
        return;
    }
    let source = context.source_file().node(&node.as_node());
    let opening_len = opening.as_slice().len();
    if source.len() <= opening_len + 1 {
        return;
    }
    let content = &source[opening_len..source.len() - 1];
    let quoted = content.split_ascii_whitespace().any(|token| {
        let token = token.trim_end_matches(',');
        token.len() > 2
            && (token.starts_with('\'') && token.ends_with('\'')
                || token.starts_with('"') && token.ends_with('"'))
    });
    let attached_comma = content.as_bytes().iter().enumerate().any(|(index, byte)| {
        *byte == b',' && index > 0 && !content.as_bytes()[index - 1].is_ascii_whitespace()
    });
    if !quoted && !attached_comma {
        return;
    }
    let clean = content
        .chars()
        .filter(|character| !matches!(character, '\'' | '"' | ','))
        .collect::<String>();
    let replacement = format!(
        "{}{}{}",
        context.source_file().at(&opening),
        clean,
        &source[source.len() - 1..]
    );
    context.replace(
        "Within `%w`/`%W`, quotes and ',' are unnecessary and may be unwanted in the resulting strings.",
        node.location(),
        node.location(),
        replacement,
    );
}

fn redundant_percent_q(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let dynamic = node.as_interpolated_string_node().is_some();
    let opening = if let Some(string) = node.as_string_node() {
        string.opening_loc()
    } else {
        node.as_interpolated_string_node()
            .and_then(|string| string.opening_loc())
    };
    let Some(opening) = opening else {
        return;
    };
    let source = context.source_file().node(node);
    let kind = if opening.as_slice().starts_with(b"%q") {
        'q'
    } else if opening.as_slice().starts_with(b"%Q") {
        'Q'
    } else {
        return;
    };
    if source.len() < 4 {
        return;
    }
    let content = &source[3..source.len() - 1];
    let single = content.contains('\'');
    let double = content.contains('"');
    let escaped_non_backslash = has_escaped_non_backslash(content);
    let allowed = if kind == 'q' {
        single && double || escaped_non_backslash || single && content.contains("#{")
    } else {
        double && (single || dynamic || super::string_conversion_rules::double_quotes_required(source))
    };
    if allowed {
        return;
    }
    let quote = if kind == 'q' {
        if single {
            '"'
        } else {
            '\''
        }
    } else if !dynamic && double {
        '\''
    } else {
        '"'
    };
    let correction = format!("{quote}{content}{quote}");
    let message = if kind == 'q' {
        "Use `%q` only for strings that contain both single quotes and double quotes."
    } else {
        "Use `%Q` only for strings that contain both single quotes and double quotes, or for dynamic strings that contain double quotes."
    };
    context.replace(message, node.location(), node.location(), correction);
}

fn has_escaped_non_backslash(content: &str) -> bool {
    let bytes = content.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'\\' {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && bytes[index] == b'\\' {
            index += 1;
        }
        if (index - start) % 2 == 1 && index < bytes.len() {
            return true;
        }
    }
    false
}
