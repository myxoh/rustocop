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
    report_duplicate_percent_symbol_sets(source, context);
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

fn report_duplicate_percent_symbol_sets(source: &str, context: &mut CopContext<'_, '_>) {
    let mut search = 0;
    while let Some(relative) = source[search..].find("%i[") {
        let open = search + relative + 2;
        let Some(close_relative) = source[open + 1..].find(']') else {
            break;
        };
        let close = open + 1 + close_relative;
        if !source[close + 1..].trim_start().starts_with(".to_set") {
            search = close + 1;
            continue;
        }
        let body = &source[open + 1..close];
        let mut seen = Vec::<&str>::new();
        let mut cursor = 0;
        for value in body.split_whitespace() {
            let relative_start = body[cursor..].find(value).unwrap_or(0) + cursor;
            let value_start = open + 1 + relative_start;
            if seen.contains(&value) {
                let removal_start = source[..value_start]
                    .rfind(char::is_whitespace)
                    .unwrap_or(value_start);
                context.remove(
                    "Remove the duplicate element in Set.",
                    value_start..value_start + value.len(),
                    removal_start..value_start + value.len(),
                );
            } else {
                seen.push(value);
            }
            cursor = relative_start + value.len();
        }
        search = close + 1;
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
                        .is_some_and(|(_, next)| {
                            let next = next.trim();
                            next == "end"
                                || next == "else"
                                || next.starts_with("elsif ")
                                || next.starts_with("when ")
                                || next.starts_with("in ")
                        })
                        || returns_after_continuation(
                            &lines[index + 1..],
                            line.len() - line.trim_start().len(),
                        )))
                || (in_conditional_branch(&lines[..index], line.len() - line.trim_start().len())
                    && lines[index + 1..].iter().find(|(_, next)| !next.trim().is_empty()).is_some_and(|(_, next)| {
                        let next = next.trim();
                        next == "end" || next == "else" || next.starts_with("elsif ") || next.starts_with("when ") || next.starts_with("in ")
                    })))
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

fn in_conditional_branch(lines: &[(usize, &str)], indent: usize) -> bool {
    lines.iter().rev().find(|(_, line)| {
        !line.trim().is_empty() && line.len() - line.trim_start().len() < indent
    }).is_some_and(|(_, line)| {
        let line = line.trim_start();
        line.starts_with("if ") || line.starts_with("elsif ") || line == "else"
            || line.starts_with("case ") || line.starts_with("when ") || line.starts_with("in ")
    })
}

fn empty_literal(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let string_literal = if context.related_config_value("Style/StringLiterals", "EnforcedStyle") == Some("double_quotes") {
        "\"\""
    } else {
        "''"
    };
    let frozen_comment = source.lines().take(2).find_map(|line| {
        let value = line.trim().strip_prefix("# frozen_string_literal:")?.trim();
        matches!(value, "true" | "false").then_some(value == "true")
    });
    let frozen_strings = frozen_comment.unwrap_or_else(|| {
        match context.related_config_value("AllCops", "StringLiteralsFrozenByDefault") {
            Some("true") => true,
            Some("false") => false,
            _ => context.related_config_value("Style/FrozenStringLiteralComment", "Enabled") == Some("true"),
        }
    });
    for (constructor, literal, kind) in [
        ("Array.new", "[]", "array"),
        ("Hash.new", "{}", "hash"),
        ("String.new", string_literal, "string"),
    ] {
        let mut search = 0;
        while let Some(relative) = source[search..].find(constructor) {
            let start = search + relative;
            let offense_start = if source.get(start.saturating_sub(2)..start) == Some("::") {
                start - 2
            } else {
                start
            };
            if kind == "string" && frozen_strings {
                search = start + constructor.len();
                continue;
            }
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
                    if kind == "string" { constructor } else { &source[offense_start..end] }
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
