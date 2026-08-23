use std::collections::HashMap;

use super::source_helpers::*;
use super::*;

declare_source_cops! {
    LeadingEmptyLines => "Layout/LeadingEmptyLines" => leading_empty_lines,
    EmptyBlockParameter => "Style/EmptyBlockParameter" => empty_block_parameter,
    TripleQuotes => "Lint/TripleQuotes" => triple_quotes,
    UriEscapeUnescape => "Lint/UriEscapeUnescape" => uri_escape_unescape,
    OrAssignmentToConstant => "Lint/OrAssignmentToConstant" => or_assignment_to_constant,
    OrderedMagicComments => "Lint/OrderedMagicComments" => ordered_magic_comments,
    DuplicateRequire => "Lint/DuplicateRequire" => duplicate_require,
}

fn leading_empty_lines(source: &str, reporter: &mut Reporter<'_>) {
    let leading = source.bytes().take_while(|byte| *byte == b'\n').count();
    if leading == 0 || leading == source.len() {
        return;
    }
    let line = source[leading..].split('\n').next().unwrap_or_default();
    let token_end = if line.starts_with('#') {
        leading + line.len()
    } else {
        line.find(char::is_whitespace)
            .map_or(leading + line.len(), |end| leading + end)
    };
    reporter.replace(
        "Unnecessary blank line at the beginning of the source.",
        leading..token_end,
        0..leading,
        "",
    );
}

fn empty_block_parameter(source: &str, reporter: &mut Reporter<'_>) {
    for start in SourceFile::new(source).code_offsets("||") {
        let before = source[..start].trim_end();
        let do_block = before.strip_suffix("do").is_some_and(|prefix| {
            prefix
                .chars()
                .last()
                .is_none_or(|character| !character.is_alphanumeric() && character != '_')
        });
        if do_block || before.ends_with('{') {
            let edit = if do_block {
                start.saturating_sub(1)..start + 2
            } else {
                start..start + 2 + usize::from(source.as_bytes().get(start + 2) == Some(&b' '))
            };
            reporter.remove(
                "Omit pipes for the empty block parameters.",
                start..start + 2,
                edit,
            );
        }
    }
}

fn triple_quotes(source: &str, reporter: &mut Reporter<'_>) {
    let bytes = source.as_bytes();
    let literal_ranges = SourceFile::new(source).literal_ranges();
    let triple_starts = literal_ranges
        .iter()
        .filter_map(|range| {
            let literal = &source[range.clone()];
            ((literal.starts_with("\"\"\"") && literal.ends_with("\"\"\""))
                || (literal.starts_with("'''") && literal.ends_with("'''")))
                .then_some(range.start)
        })
        .collect::<std::collections::HashSet<_>>();
    let mut start = 0;
    while start + 2 < bytes.len() {
        let quote = bytes[start];
        if !matches!(quote, b'\'' | b'"') || bytes[start + 1] != quote || bytes[start + 2] != quote
        {
            start += 1;
            continue;
        }
        if !triple_starts.contains(&start) {
            start += 3;
            continue;
        }
        let run = bytes[start..]
            .iter()
            .take_while(|byte| **byte == quote)
            .count();
        if source[start..].lines().next().is_some_and(|line| {
            !line.is_empty() && line.bytes().all(|byte| byte == quote) && line.len() >= 6
        }) {
            let end = start + run;
            reporter.replace("Delimiting a string with multiple quotes has no effect, use a single quote instead.", start..end, start..end, format!("{}{}", quote as char, quote as char));
            start = end;
            continue;
        }
        let delimiter = String::from_utf8(vec![quote; 3]).unwrap();
        let Some(relative_end) = source[start + run..].find(&delimiter) else {
            start += run;
            continue;
        };
        let end_quote = start + run + relative_end;
        let end_run = bytes[end_quote..]
            .iter()
            .take_while(|byte| **byte == quote)
            .count();
        let end = end_quote + end_run;
        if end_run >= 3 {
            let content = &source[start + run..end_quote];
            reporter.replace("Delimiting a string with multiple quotes has no effect, use a single quote instead.", start..end, start..end, format!("{}{}{}", quote as char, content, quote as char));
            start = end;
        } else {
            start += run;
        }
    }
}

fn uri_escape_unescape(source: &str, reporter: &mut Reporter<'_>) {
    if !reporter.config_bool("Enabled", true)
        || reporter.related_config_value("AllCops", "DisabledByDefault") == Some("true")
            && !reporter.related_config_explicit("Lint/UriEscapeUnescape", "Enabled")
    {
        return;
    }
    for method in ["escape", "encode", "unescape", "decode"] {
        for prefix in ["::URI.", "URI."] {
            let needle = format!("{prefix}{method}(");
            for start in all_offsets(source, &needle) {
                if prefix == "URI." && start >= 2 && &source[start - 2..start] == "::" {
                    continue;
                }
                let Some(close) = source[start..].find(')') else {
                    continue;
                };
                let end = start + close + 1;
                let alternatives = if matches!(method, "escape" | "encode") {
                    "`CGI.escape`, `URI.encode_www_form` or `URI.encode_www_form_component`"
                } else {
                    "`CGI.unescape`, `URI.decode_www_form` or `URI.decode_www_form_component`"
                };
                reporter.report(format!("`{prefix}{method}` method is obsolete and should not be used. Instead, use {alternatives} depending on your specific use case."), start..end);
            }
        }
    }
}

fn or_assignment_to_constant(source: &str, reporter: &mut Reporter<'_>) {
    for operator in all_offsets(source, "||=") {
        let line_start = source[..operator]
            .rfind('\n')
            .map_or(0, |offset| offset + 1);
        let left = source[line_start..operator].trim();
        if left.is_empty()
            || !left
                .bytes()
                .last()
                .is_some_and(|byte| byte.is_ascii_uppercase())
        {
            continue;
        }
        let inside_def = source[..line_start]
            .lines()
            .rev()
            .take_while(|line| line.trim() != "end")
            .any(|line| line.trim_start().starts_with("def "));
        if inside_def && left.contains("::") {
            reporter.report(
                "Avoid using or-assignment with constants.",
                operator..operator + 3,
            );
        } else {
            reporter.replace(
                "Avoid using or-assignment with constants.",
                operator..operator + 3,
                operator..operator + 3,
                "=",
            );
        }
    }
}

fn ordered_magic_comments(source: &str, reporter: &mut Reporter<'_>) {
    let lines = source_lines(source).collect::<Vec<_>>();
    let leading = lines.iter().take_while(|(offset, line)| {
        let trimmed = line.trim();
        trimmed.is_empty()
            || trimmed.starts_with('#')
            || (*offset == 0 && trimmed.starts_with("#!"))
    });
    let encoding = leading.clone().position(|(_, line)| {
        let trimmed = line.trim();
        trimmed.starts_with("# encoding:")
            || trimmed.starts_with("# coding:")
            || trimmed.starts_with("# -*- encoding")
    });
    let frozen = lines
        .iter()
        .take_while(|(offset, line)| {
            let trimmed = line.trim();
            trimmed.is_empty()
                || trimmed.starts_with('#')
                || (*offset == 0 && trimmed.starts_with("#!"))
        })
        .position(|(_, line)| line.trim().starts_with("# frozen_string_literal:"));
    let (Some(encoding), Some(frozen)) = (encoding, frozen) else {
        return;
    };
    if encoding <= frozen {
        return;
    }
    let (encoding_offset, encoding_line) = lines[encoding];
    let (frozen_offset, frozen_line) = lines[frozen];
    let end = encoding_offset + encoding_line.len();
    let replacement = format!("{encoding_line}\n{frozen_line}");
    reporter.replace(
        "The encoding magic comment should precede all other magic comments.",
        encoding_offset..end,
        frozen_offset..end,
        replacement,
    );
}

fn duplicate_require(source: &str, reporter: &mut Reporter<'_>) {
    let mut seen = HashMap::<(&str, &str), usize>::new();
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim_start();
        let normalized = trimmed.strip_prefix("Kernel.").unwrap_or(trimmed);
        let method = if normalized.starts_with("require_relative ") {
            "require_relative"
        } else if normalized.starts_with("require ") {
            "require"
        } else {
            continue;
        };
        let argument = normalized[method.len()..].trim();
        if seen.insert((method, argument), offset).is_some() {
            let start = offset + line.len() - trimmed.len();
            reporter.remove(
                format!("Duplicate `{method}` detected."),
                start..offset + line.len(),
                offset..line_end(source, offset),
            );
        }
    }
}
