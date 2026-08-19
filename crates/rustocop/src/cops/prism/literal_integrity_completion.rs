use std::collections::HashMap;

use super::*;

mod helpers;
use helpers::*;

define_cops! {
    DuplicateHashKey => "Lint/DuplicateHashKey" => source(duplicate_hash_key),
    DuplicateSetElement => "Lint/DuplicateSetElement" => source(duplicate_set_element),
    NumericOperationWithConstantResult => "Lint/NumericOperationWithConstantResult" => source(numeric_constant_result),
    SymbolConversion => "Lint/SymbolConversion" => source(symbol_conversion),
    DoubleNegation => "Style/DoubleNegation" => source(double_negation),
    EmptyLiteral => "Style/EmptyLiteral" => source(empty_literal),
}

fn duplicate_hash_key(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let mut cursor = 0;
    while let Some(open_relative) = source[cursor..].find('{') {
        let open = cursor + open_relative;
        let Some(close_relative) = source[open..].rfind('}') else {
            break;
        };
        let close = open + close_relative;
        let body = &source[open + 1..close];
        let mut seen = HashMap::<String, usize>::new();
        for (position, entry) in top_level_entries(body) {
            let leading = entry.len() - entry.trim_start().len();
            let trimmed = entry.trim();
            let key = if let Some((key, _)) = trimmed
                .split_once(" => ")
                .or_else(|| trimmed.split_once("=>"))
            {
                key.trim()
            } else if let Some((key, _)) = trimmed.split_once(':') {
                key.trim()
            } else {
                continue;
            };
            if !key.is_empty() && seen.contains_key(key) {
                let start = open + 1 + position + leading;
                context.report("Duplicated key in hash literal.", start..start + key.len());
            } else {
                seen.insert(key.to_string(), position);
            }
        }
        cursor = close + 1;
    }
}

fn duplicate_set_element(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for marker in ["Set["] {
        let mut search = 0;
        while let Some(relative) = source[search..].find(marker) {
            let start = search + relative;
            let open = start + marker.len() - 1;
            let Some(close_relative) = source[open..].find(']') else {
                break;
            };
            let close = open + close_relative;
            let body = &source[open + 1..close];
            let mut seen = Vec::new();
            for (position, entry) in top_level_entries(body) {
                let value = entry.trim();
                let leading = entry.len() - entry.trim_start().len();
                if value.contains(['(', ')']) || value.contains("#{") {
                    continue;
                }
                if seen.contains(&value) {
                    let value_start = open + 1 + position + leading;
                    let comma_start = source[..value_start].rfind(',').unwrap_or(value_start);
                    context.remove(
                        "Remove the duplicate element in Set.",
                        value_start..value_start + value.len(),
                        comma_start..value_start + value.len(),
                    );
                } else {
                    seen.push(value);
                }
            }
            search = close + 1;
        }
    }
}

fn numeric_constant_result(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let code = line.split('#').next().unwrap_or(line).trim();
        let replacement = if let Some(left) = code.strip_suffix(" * 0") {
            (!left.trim().bytes().all(|b| b.is_ascii_digit())).then(|| "0".to_string())
        } else if let Some(right) = code.strip_prefix("0 * ") {
            (!right.trim().bytes().all(|b| b.is_ascii_digit())).then(|| "0".to_string())
        } else if code.ends_with(" ** 0") {
            Some("1".to_string())
        } else if code.ends_with(" & 0") {
            Some("0".to_string())
        } else if let Some((left, right)) = code.split_once(" / ") {
            (left.trim() == right.trim()).then(|| "1".to_string())
        } else if let Some(left) = code.strip_suffix(" *= 0") {
            (!left.trim().is_empty()).then(|| format!("{} = 0", left.trim()))
        } else if let Some((left, right)) = code.split_once(" /= ") {
            (left.trim() == right.trim()).then(|| format!("{} = 1", left.trim()))
        } else if code.ends_with(" **= 0") {
            Some(format!("{} = 1", code.trim_end_matches(" **= 0").trim()))
        } else if code.ends_with(".*(0)") || code.ends_with("&.*(0)") {
            Some("0".to_string())
        } else if let Some((left, right)) =
            code.split_once("&./(").or_else(|| code.split_once("./("))
        {
            (left.trim() == right.trim_end_matches(')').trim()).then(|| "1".to_string())
        } else {
            None
        };
        if let Some(replacement) = replacement {
            let start = offset + line.find(code).unwrap_or(0);
            context.replace(
                "Numeric operation with a constant result detected.",
                start..start + code.len(),
                start..start + code.len(),
                replacement,
            );
        }
    }
}

fn symbol_conversion(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for (start, _) in source.match_indices(':') {
        let tail = &source[start + 1..];
        let name_len = tail
            .bytes()
            .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            .count();
        if name_len > 0
            && source.get(start + 1 + name_len..start + 1 + name_len + 7) == Some(".to_sym")
        {
            let end = start + 1 + name_len + 7;
            let symbol = &source[start..start + 1 + name_len];
            context.replace(
                format!("Unnecessary symbol conversion; use `{symbol}` instead."),
                start..end,
                start..end,
                symbol,
            );
        }
    }
    for quote in ['\'', '"'] {
        let needle = format!("{quote}.to_sym");
        let mut search = 0;
        while let Some(relative) = source[search..].find(&needle) {
            let end = search + relative + needle.len();
            let Some(start) = source[..end - needle.len()].rfind(quote) else {
                break;
            };
            let value = &source[start + 1..end - needle.len()];
            if value.contains("#{") || value.contains(' ') || value.is_empty() {
                search = end;
                continue;
            }
            let bare = value.bytes().enumerate().all(|(index, byte)| {
                byte.is_ascii_alphanumeric()
                    || byte == b'_'
                    || (index + 1 == value.len() && matches!(byte, b'!' | b'?' | b'='))
            });
            let replacement = if bare {
                format!(":{value}")
            } else {
                format!(":{quote}{value}{quote}")
            };
            context.replace(
                format!("Unnecessary symbol conversion; use `{replacement}` instead."),
                start..end,
                start..end,
                replacement,
            );
            search = end;
        }
    }
}

fn double_negation(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        let Some(at) = line.find("!!") else { continue };
        let tail = &line[at + 2..];
        let leading = tail.len() - tail.trim_start().len();
        let expression_start = at + 2 + leading;
        let expression_len = if line.as_bytes().get(expression_start) == Some(&b'(') {
            matching_delimiter(&line[expression_start..], b'(', b')').unwrap_or(tail.len())
        } else {
            line[expression_start..]
                .bytes()
                .take_while(|byte| {
                    byte.is_ascii_alphanumeric()
                        || matches!(byte, b'_' | b'@' | b'$' | b'.' | b'?' | b'!')
                })
                .count()
        };
        let expression = &line[expression_start..expression_start + expression_len];
        if expression.is_empty() {
            continue;
        }
        if context.policy().enforced_style("allowed_in_returns") == "allowed_in_returns"
            && (line[..at].trim().starts_with("return")
                || (line[..at].trim().is_empty()
                    && (lines[index + 1..]
                        .iter()
                        .find(|(_, next)| !next.trim().is_empty())
                        .is_some_and(|(_, next)| next.trim() == "end")
                        || returns_after_continuation(
                            &lines[index + 1..],
                            line.len() - line.trim_start().len(),
                        ))))
        {
            continue;
        }
        context.replace(
            "Avoid the use of double negation (`!!`).",
            offset + at..offset + at + 1,
            offset + at..offset + expression_start + expression_len,
            format!("!{expression}.nil?"),
        );
    }
}

fn empty_literal(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for (constructor, literal, kind) in [
        ("Array.new", "[]", "array"),
        ("Hash.new", "{}", "hash"),
        ("String.new", "''", "string"),
    ] {
        let mut search = 0;
        while let Some(relative) = source[search..].find(constructor) {
            let start = search + relative;
            let offense_start = if source.get(start.saturating_sub(2)..start) == Some("::") {
                start - 2
            } else {
                start
            };
            let mut end = start + constructor.len();
            if source.get(end..end + 2) == Some("()") {
                end += 2;
            } else if source
                .get(end..end + literal.len() + 2)
                .is_some_and(|arguments| arguments == format!("({literal})"))
            {
                end += literal.len() + 2;
            } else if source.as_bytes().get(end) == Some(&b'(')
                || source[end..].trim_start().starts_with(['{', 'd'])
            {
                search = end + 1;
                continue;
            }
            context.replace(
                format!(
                    "Use {kind} literal `{literal}` instead of `{}`.",
                    &source[offense_start..end]
                ),
                offense_start..end,
                if kind == "hash"
                    && source.as_bytes().get(offense_start.wrapping_sub(1)) == Some(&b' ')
                    && !source[..offense_start].trim_end().ends_with('=')
                    && !source[..offense_start].trim_end().ends_with('{')
                {
                    offense_start - 1..end
                } else {
                    offense_start..end
                },
                if kind == "hash"
                    && source.as_bytes().get(offense_start.wrapping_sub(1)) == Some(&b' ')
                    && !source[..offense_start].trim_end().ends_with('=')
                    && !source[..offense_start].trim_end().ends_with('{')
                {
                    if matches!(source.as_bytes().get(end), Some(b',' | b')')) {
                        "({}"
                    } else {
                        "({})"
                    }
                } else {
                    literal
                },
            );
            search = end;
        }
    }
    for (constructor, literal, kind) in [
        ("Array[]", "[]", "array"),
        ("Array([])", "[]", "array"),
        ("Hash[]", "{}", "hash"),
        ("Hash([])", "{}", "hash"),
    ] {
        for (start, _) in source.match_indices(constructor) {
            context.replace(
                format!("Use {kind} literal `{literal}` instead of `{constructor}`."),
                start..start + constructor.len(),
                start..start + constructor.len(),
                literal,
            );
        }
    }
}
