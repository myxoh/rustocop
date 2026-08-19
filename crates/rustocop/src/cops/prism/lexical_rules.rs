use std::collections::HashSet;

use super::source_helpers::*;
use super::*;

mod interpolation;
use interpolation::*;

define_cops! {
    InitialIndentation => "Layout/InitialIndentation" => source(initial_indentation),
    DuplicateMagicComment => "Lint/DuplicateMagicComment" => source(duplicate_magic_comment),
    EmptyInterpolation => "Lint/EmptyInterpolation" => source(empty_interpolation),
    InterpolationCheck => "Lint/InterpolationCheck" => source(interpolation_check),
    RequireRangeParentheses => "Lint/RequireRangeParentheses" => source(require_range_parentheses),
    AsciiIdentifiers => "Naming/AsciiIdentifiers" => source(ascii_identifiers),
    MultilineIfThen => "Style/MultilineIfThen" => source(multiline_if_then),
    ReturnNil => "Style/ReturnNil" => source(return_nil),
    VariableInterpolation => "Style/VariableInterpolation" => source(variable_interpolation),
}

fn initial_indentation(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for (line_start, line) in source_lines(source) {
        let logical = line.strip_prefix('\u{feff}').unwrap_or(line);
        let bom = line.len() - logical.len();
        let trimmed = logical.trim_start_matches([' ', '\t']);
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indentation = logical.len() - trimmed.len();
        if indentation == 0 {
            return;
        }
        let token_len = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
        let token_start = line_start + bom + indentation;
        context.remove(
            "Indentation of first line in file detected.",
            token_start..token_start + token_len,
            line_start + bom..token_start,
        );
        return;
    }
}

fn duplicate_magic_comment(context: &mut CopContext<'_, '_>) {
    let mut seen = HashSet::new();
    for (start, line) in source_lines(context.source()) {
        let trimmed = line.trim_start_matches('\u{feff}');
        let kind = if trimmed.starts_with("# frozen_string_literal:") {
            "frozen"
        } else if trimmed.starts_with("# encoding:") || trimmed.starts_with("# coding:") {
            "encoding"
        } else {
            continue;
        };
        if seen.insert(kind) {
            continue;
        }
        let offense_start = start + line.len() - trimmed.len();
        let edit_end = line_end(context.source(), start);
        context.remove(
            "Duplicate magic comment detected.",
            offense_start..start + line.len(),
            start..edit_end,
        );
    }
}

fn empty_interpolation(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for (start, end) in interpolation_ranges(source) {
        let inner = source[start + 2..end - 1].trim();
        if !matches!(inner, "" | "''" | "\"\"" | "nil") || percent_word_literal(source, start) {
            continue;
        }
        context.remove("Empty interpolation detected.", start..end, start..end);
    }
}

fn interpolation_check(context: &mut CopContext<'_, '_>) {
    const MESSAGE: &str = "Interpolation in single quoted string detected. Use double quoted strings if you need interpolation.";
    let source = context.source();
    for range in single_quoted_ranges(source) {
        let content = &source[range.start + 1..range.end - 1];
        let Some(open) = content.find("#{") else {
            continue;
        };
        let Some(close) = content[open + 2..].find('}') else {
            continue;
        };
        let expression = &content[open + 2..open + 2 + close];
        if expression.is_empty()
            || !expression
                .bytes()
                .all(|byte| identifier_byte(byte) || matches!(byte, b'.' | b'@' | b'$'))
        {
            continue;
        }
        let replacement = if content.contains('"') {
            if unmatched_closing_brace(content) {
                continue;
            }
            format!("%{{{content}}}")
        } else {
            format!("\"{content}\"")
        };
        context.replace(MESSAGE, range.clone(), range, replacement);
    }
}

fn require_range_parentheses(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let lines = source_lines(source).collect::<Vec<_>>();
    for (index, (start, line)) in lines.iter().enumerate() {
        let trimmed = line.trim_end();
        let operator_len = if trimmed.ends_with("...") {
            3
        } else if trimmed.ends_with("..") {
            2
        } else {
            continue;
        };
        if index + 1 >= lines.len() || lines[index + 1].1.trim().is_empty() {
            continue;
        }
        let expression_start = *start + line.len() - line.trim_start().len();
        if source[..expression_start].trim_end().ends_with('(') || trimmed.starts_with('(') {
            continue;
        }
        let expression = &trimmed[..trimmed.len() - operator_len];
        if expression.is_empty() {
            continue;
        }
        let end = lines[index + 1].0 + lines[index + 1].1.len();
        context.report(
            format!("Wrap the endless range literal `{trimmed}` to avoid precedence ambiguity."),
            expression_start..end,
        );
    }
}

fn ascii_identifiers(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let ascii_constants = context.config_bool("AsciiConstants", true);
    let mut in_quote = None;
    let mut comment = false;
    let mut reported_through = 0;
    for (offset, character) in source.char_indices() {
        if character == '\n' {
            comment = false;
            continue;
        }
        if comment {
            continue;
        }
        if let Some(quote) = in_quote {
            if character == quote {
                in_quote = None;
            }
            continue;
        }
        if character == '#' {
            comment = true;
            continue;
        }
        if matches!(character, '\'' | '"' | '`') {
            in_quote = Some(character);
            continue;
        }
        if offset < reported_through
            || character.is_ascii()
            || character == '\u{feff}'
            || character.is_whitespace()
        {
            continue;
        }
        let line_start = source[..offset].rfind('\n').map_or(0, |at| at + 1);
        let identifier_start = source[line_start..offset]
            .char_indices()
            .rev()
            .find(|(_, character)| !(character.is_alphanumeric() || *character == '_'))
            .map_or(line_start, |(at, character)| {
                line_start + at + character.len_utf8()
            });
        let line_prefix = source[line_start..].trim_start();
        let is_constant = line_prefix.starts_with("class ")
            || line_prefix.starts_with("module ")
            || source[identifier_start..]
                .chars()
                .next()
                .is_some_and(char::is_uppercase);
        if is_constant && !ascii_constants {
            continue;
        }
        let mut end = offset + character.len_utf8();
        while let Some(next) = source[end..].chars().next() {
            if next.is_ascii() || next.is_whitespace() {
                break;
            }
            end += next.len_utf8();
        }
        reported_through = end;
        context.report(
            if is_constant {
                "Use only ascii symbols in constants."
            } else {
                "Use only ascii symbols in identifiers."
            },
            offset..end,
        );
    }
}

fn multiline_if_then(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let lines = source_lines(source).collect::<Vec<_>>();
    for (line_index, (line_start, line)) in lines.iter().copied().enumerate() {
        let code = line.split('#').next().unwrap_or_default();
        let trimmed = code.trim_start();
        let mut keyword = ["if", "elsif", "unless"]
            .into_iter()
            .find(|keyword| trimmed.starts_with(&format!("{keyword} ")) || trimmed == *keyword);
        if keyword.is_none() && trimmed == "then" {
            keyword = lines[..line_index].iter().rev().find_map(|(_, previous)| {
                let previous = previous.trim_start();
                ["if", "elsif", "unless"]
                    .into_iter()
                    .find(|word| previous.starts_with(&format!("{word} ")))
            });
        }
        let Some(keyword) = keyword else { continue };
        let Some(relative) = code.find("then") else {
            continue;
        };
        let before = code.as_bytes().get(relative.wrapping_sub(1)).copied();
        let after = code.as_bytes().get(relative + 4).copied();
        if before.is_some_and(identifier_byte) || after.is_some_and(identifier_byte) {
            continue;
        }
        let has_body = !line[relative + 4..].trim().is_empty()
            && !line[relative + 4..].trim_start().starts_with('#');
        if has_body {
            continue;
        }
        let token = line_start + relative..line_start + relative + 4;
        let edit = if line[..relative].trim().is_empty() {
            line_start..line_end(source, line_start)
        } else if line[relative + 4..].starts_with(' ') {
            token.start..token.end + 1
        } else {
            token.start.saturating_sub(1)..token.end
        };
        context.remove(
            format!("Do not use `then` for multi-line `{keyword}`."),
            token,
            edit,
        );
    }
}

fn return_nil(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let return_nil = context.policy().enforced_style("return") == "return_nil";
    for (start, line) in source_lines(source) {
        let leading = line.len() - line.trim_start().len();
        let code = line.trim_start();
        if return_nil {
            if code == "return" {
                let range = start + leading..start + leading + 6;
                context.replace(
                    "Use `return nil` instead of `return`.",
                    range.clone(),
                    range,
                    "return nil",
                );
            }
        } else if code.starts_with("return nil")
            && code
                .as_bytes()
                .get(10)
                .is_none_or(|byte| !identifier_byte(*byte))
        {
            let range = start + leading..start + leading + 10;
            context.replace(
                "Use `return` instead of `return nil`.",
                range.clone(),
                range,
                "return",
            );
        }
    }
}
