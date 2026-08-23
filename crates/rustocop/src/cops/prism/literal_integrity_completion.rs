use std::collections::{HashMap, HashSet};

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
    let mut inspected = HashSet::new();
    for (open, _) in source.match_indices('[') {
        if source[..open].ends_with("%i") || !inspected.insert(open) {
            continue;
        }
        let Some(close) = super::source_syntax::matching_delimiter(source, open, b'[', b']') else {
            continue;
        };
        let Some(name) = set_constructor_name(source, open, close) else {
            continue;
        };
        report_duplicate_set_entries(source, open, close, name, context);
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
        let before = &source[..open.saturating_sub(2)];
        let after = &source[close + 1..];
        let name = if before.ends_with("SortedSet.new(") {
            "SortedSet"
        } else if before.ends_with("Set.new(") {
            "Set"
        } else if after.starts_with(".to_set") || after.starts_with("&.to_set") {
            "Set"
        } else {
            search = close + 1;
            continue;
        };
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
                    format!("Remove the duplicate element in {name}."),
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

fn set_constructor_name(source: &str, open: usize, close: usize) -> Option<&'static str> {
    let before = &source[..open];
    if before.ends_with("SortedSet") || before.ends_with("SortedSet.new(") {
        Some("SortedSet")
    } else if before.ends_with("Set") || before.ends_with("Set.new(") {
        let boundary = before
            .len()
            .saturating_sub(if before.ends_with("Set.new(") {
                "Set.new(".len()
            } else {
                "Set".len()
            });
        if boundary > 0 && source.as_bytes()[boundary - 1].is_ascii_alphanumeric() {
            None
        } else {
            Some("Set")
        }
    } else {
        let after = &source[close + 1..];
        (after.starts_with(".to_set") || after.starts_with("&.to_set")).then_some("Set")
    }
}

fn report_duplicate_set_entries(
    source: &str,
    open: usize,
    close: usize,
    name: &str,
    context: &mut CopContext<'_, '_>,
) {
    let body = &source[open + 1..close];
    let mut seen = Vec::new();
    for (position, entry) in top_level_entries(body) {
        let value = entry.trim();
        let leading = entry.len() - entry.trim_start().len();
        if !stable_set_element(value, &source[..open]) {
            continue;
        }
        if seen.contains(&value) {
            let value_start = open + 1 + position + leading;
            let comma_start = source[..value_start].rfind(',').unwrap_or(value_start);
            context.remove(
                format!("Remove the duplicate element in {name}."),
                value_start..value_start + value.len(),
                comma_start..value_start + value.len(),
            );
        } else {
            seen.push(value);
        }
    }
}

fn stable_set_element(value: &str, preceding_source: &str) -> bool {
    if value.is_empty()
        || value.contains("#{")
        || value.contains("&.")
        || value.contains(" ? ")
        || value.contains(['(', ')'])
    {
        return false;
    }
    let first = value.as_bytes()[0];
    if matches!(first, b':' | b'@' | b'\'' | b'"')
        || first.is_ascii_uppercase()
        || first.is_ascii_digit()
        || matches!(value, "true" | "false" | "nil")
    {
        return true;
    }
    preceding_source.lines().any(|line| {
        let line = line.trim_start();
        line.strip_prefix(value)
            .is_some_and(|tail| tail.trim_start().starts_with('='))
    })
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
        if source.as_bytes().get(start.wrapping_sub(1)) == Some(&b':')
            || source.as_bytes().get(start + 1) == Some(&b':')
        {
            continue;
        }
        let tail = &source[start + 1..];
        let name_len = tail
            .bytes()
            .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            .count();
        let conversion = source
            .get(start + 1 + name_len..)
            .and_then(symbol_conversion_method);
        if name_len > 0 && conversion.is_some() {
            let end = start + 1 + name_len + conversion.unwrap_or_default().len();
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
        for method in [".to_sym", ".intern"] {
            let needle = format!("{quote}{method}");
            let mut search = 0;
            while let Some(relative) = source[search..].find(&needle) {
                let closing = search + relative;
                let end = closing + needle.len();
                let Some(start) = source[..closing].rfind(quote) else {
                    break;
                };
                let value = &source[start + 1..closing];
                if value.contains(' ') || value.is_empty() {
                    search = end;
                    continue;
                }
                let replacement = symbol_literal(value, quote);
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

    for quote in ['\'', '"'] {
        let needle = format!(":{quote}");
        let mut search = 0;
        while let Some(relative) = source[search..].find(&needle) {
            let start = search + relative;
            let content_start = start + 2;
            let Some(relative_close) = source[content_start..].find(quote) else {
                break;
            };
            let close = content_start + relative_close;
            let value = &source[content_start..close];
            if bare_symbol_name(value, true) {
                let replacement = format!(":{value}");
                context.replace(
                    format!("Unnecessary symbol conversion; use `{replacement}` instead."),
                    start..close + 1,
                    start..close + 1,
                    replacement,
                );
            }
            search = close + 1;
        }
    }

    check_symbol_hash_labels(context);
}

fn symbol_conversion_method(source: &str) -> Option<&'static str> {
    [".to_sym", ".intern"]
        .into_iter()
        .find(|method| source.starts_with(method))
}

fn symbol_literal(value: &str, quote: char) -> String {
    if bare_symbol_name(value, true) && !value.contains("#{") {
        format!(":{value}")
    } else {
        format!(":{quote}{value}{quote}")
    }
}

fn bare_symbol_name(value: &str, allow_suffix: bool) -> bool {
    !value.is_empty()
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphanumeric()
                || byte == b'_'
                || (allow_suffix && index + 1 == value.len() && matches!(byte, b'!' | b'?' | b'='))
        })
}

fn check_symbol_hash_labels(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let style = context.policy().enforced_style("strict");
    let quoted = quoted_hash_labels(source);
    let quote_all = style == "consistent"
        && quoted
            .iter()
            .any(|label| !bare_symbol_name(&source[label.content.clone()], false));

    if !quote_all {
        for label in quoted {
            let value = &source[label.content.clone()];
            if !bare_symbol_name(value, true) || value.ends_with('=') {
                continue;
            }
            let replacement = format!("{value}:");
            context.replace(
                format!("Unnecessary symbol conversion; use `{replacement}` instead."),
                label.start..label.close + 1,
                label.start..label.close + 2,
                replacement,
            );
        }
        return;
    }

    for label in unquoted_hash_labels(source) {
        let value = &source[label.start..label.end];
        let replacement = format!("\"{value}\":");
        context.replace(
            format!(
                "Symbol hash key should be quoted for consistency; use `{replacement}` instead."
            ),
            label.start..label.end,
            label.start..label.end + 1,
            replacement,
        );
    }
}

struct QuotedHashLabel {
    start: usize,
    close: usize,
    content: std::ops::Range<usize>,
}

fn quoted_hash_labels(source: &str) -> Vec<QuotedHashLabel> {
    let bytes = source.as_bytes();
    let mut labels = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let quote = bytes[at];
        if !matches!(quote, b'\'' | b'"') {
            at += 1;
            continue;
        }
        let mut close = at + 1;
        while close < bytes.len() && bytes[close] != quote {
            close += 1;
        }
        if (at == 0 || bytes[at - 1] != b':') && bytes.get(close + 1) == Some(&b':') {
            labels.push(QuotedHashLabel {
                start: at,
                close,
                content: at + 1..close,
            });
        }
        at = close.saturating_add(1);
    }
    labels
}

struct UnquotedHashLabel {
    start: usize,
    end: usize,
}

fn unquoted_hash_labels(source: &str) -> Vec<UnquotedHashLabel> {
    let bytes = source.as_bytes();
    let mut labels = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        if !bytes[at].is_ascii_alphabetic()
            || at > 0 && !matches!(bytes[at - 1], b'{' | b',' | b' ' | b'\t' | b'\n')
        {
            at += 1;
            continue;
        }
        let start = at;
        at += 1;
        while at < bytes.len()
            && (bytes[at].is_ascii_alphanumeric() || matches!(bytes[at], b'_' | b'!' | b'?'))
        {
            at += 1;
        }
        if bytes.get(at) == Some(&b':') && bytes.get(at + 1) != Some(&b':') {
            labels.push(UnquotedHashLabel { start, end: at });
        }
    }
    labels
}

fn double_negation(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let code_offsets = context
        .source_file()
        .code_offsets("!!")
        .into_iter()
        .collect::<std::collections::HashSet<_>>();
    let literal_ranges = context.source_file().literal_ranges();
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        for (at, _) in line.match_indices("!!") {
            if !code_offsets.contains(&(offset + at)) {
                continue;
            }
            if literal_ranges
                .iter()
                .any(|range| range.start <= offset + at && offset + at < range.end)
            {
                continue;
            }
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
            && (line[..at].trim() == "return"
                || (line[..at].trim().is_empty()
                    && (lines[index + 1..]
                        .iter()
                        .find(|(_, next)| !next.trim().is_empty())
                        .is_some_and(|(_, next)| {
                            let next = next.trim();
                            next == "end"
                                || next == "else"
                                || next.starts_with("rescue")
                                || next.starts_with("ensure")
                                || next.starts_with("elsif ")
                                || next.starts_with("when ")
                                || next.starts_with("in ")
                        })
                        || returns_after_continuation(
                            &lines[index + 1..],
                            line.len() - line.trim_start().len(),
                        )))
                || (in_conditional_branch(&lines[..index], line.len() - line.trim_start().len())
                    && lines[index + 1..]
                        .iter()
                        .find(|(_, next)| !next.trim().is_empty())
                        .is_some_and(|(_, next)| {
                            let next = next.trim();
                            next == "end"
                                || next == "else"
                                || next.starts_with("rescue")
                                || next.starts_with("ensure")
                                || next.starts_with("elsif ")
                                || next.starts_with("when ")
                                || next.starts_with("in ")
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
}

fn in_conditional_branch(lines: &[(usize, &str)], indent: usize) -> bool {
    lines
        .iter()
        .rev()
        .find(|(_, line)| !line.trim().is_empty() && line.len() - line.trim_start().len() < indent)
        .is_some_and(|(_, line)| {
            let line = line.trim_start();
            line.starts_with("if ")
                || line.starts_with("elsif ")
                || line == "else"
                || line.starts_with("case ")
                || line.starts_with("when ")
                || line.starts_with("in ")
        })
}

fn empty_literal(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let string_literal = if context.related_config_value("Style/StringLiterals", "EnforcedStyle")
        == Some("double_quotes")
    {
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
            _ => {
                context.related_config_value("Style/FrozenStringLiteralComment", "Enabled")
                    == Some("true")
            }
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
            let root_qualified = source.get(start.saturating_sub(2)..start) == Some("::")
                && (start == 2
                    || source.as_bytes().get(start - 3).is_none_or(|byte| {
                        !byte.is_ascii_alphanumeric() && !matches!(byte, b'_' | b':' | b'@')
                    }));
            let bare_constant = start == 0
                || source.as_bytes().get(start - 1).is_some_and(|byte| {
                    !byte.is_ascii_alphanumeric()
                        && !matches!(byte, b'_' | b':' | b'.' | b'@')
                });
            if !root_qualified && !bare_constant {
                search = start + constructor.len();
                continue;
            }
            let offense_start = if root_qualified {
                start - 2
            } else {
                start
            };
            if kind == "string" && frozen_strings {
                search = start + constructor.len();
                continue;
            }
            let mut end = start + constructor.len();
            let same_line_tail = source[end..]
                .split_once('\n')
                .map_or(&source[end..], |(line, _)| line)
                .trim_start();
            let unparenthesized_argument = same_line_tail
                .as_bytes()
                .first()
                .is_some_and(|byte| {
                    !matches!(
                        byte,
                        b',' | b')' | b']' | b'}' | b'.' | b'&' | b';' | b'#' | b'?'
                    )
                })
                && !same_line_tail.starts_with("if ")
                && !same_line_tail.starts_with("unless ")
                && !same_line_tail.starts_with(": ");
            if source.get(end..end + 2) == Some("()") {
                end += 2;
            } else if kind == "array"
                && source
                .get(end..end + literal.len() + 2)
                .is_some_and(|arguments| arguments == format!("({literal})"))
            {
                end += literal.len() + 2;
            } else if source.as_bytes().get(end) == Some(&b'(')
                || source[end..].trim_start().starts_with(['{'])
                || source[end..].trim_start().starts_with("do")
                || unparenthesized_argument
            {
                search = end + 1;
                continue;
            }
            let message = format!(
                "Use {kind} literal `{literal}` instead of `{}`.",
                if kind == "string" {
                    constructor
                } else {
                    &source[offense_start..end]
                }
            );
            let wraps_unparenthesized_hash = kind == "hash"
                && source.as_bytes().get(offense_start.wrapping_sub(1)) == Some(&b' ')
                && !source[..offense_start].trim_end().ends_with('=')
                && !source[..offense_start].trim_end().ends_with('{');
            if wraps_unparenthesized_hash && source.as_bytes().get(end) == Some(&b',') {
                let line_end = source[end..].find('\n').map_or(source.len(), |at| end + at);
                context.replace_many(
                    message,
                    offense_start..end,
                    vec![
                        (offense_start - 1..end, "({}".to_string()),
                        (line_end..line_end, ")".to_string()),
                    ],
                );
            } else {
                context.replace(
                    message,
                    offense_start..end,
                    if wraps_unparenthesized_hash {
                        offense_start - 1..end
                    } else {
                        offense_start..end
                    },
                    if wraps_unparenthesized_hash {
                        "({})"
                    } else {
                        literal
                    },
                );
            }
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
            if start > 0
                && source.as_bytes().get(start - 1).is_some_and(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b':' | b'.' | b'@')
                })
            {
                continue;
            }
            context.replace(
                format!("Use {kind} literal `{literal}` instead of `{constructor}`."),
                start..start + constructor.len(),
                start..start + constructor.len(),
                literal,
            );
        }
    }
}
