use std::collections::HashSet;

use super::source_helpers::*;
use super::*;

mod interpolation;
use interpolation::*;

define_cops! {
    InitialIndentation => "Layout/InitialIndentation" => source(initial_indentation),
    DuplicateMagicComment => "Lint/DuplicateMagicComment" => source(duplicate_magic_comment),
    EmptyInterpolation => "Lint/EmptyInterpolation" => any_node(empty_interpolation),
    InterpolationCheck => "Lint/InterpolationCheck" => source(interpolation_check),
    RequireRangeParentheses => "Lint/RequireRangeParentheses" => source(require_range_parentheses),
    AsciiIdentifiers => "Naming/AsciiIdentifiers" => source(ascii_identifiers),
    MultilineIfThen => "Style/MultilineIfThen" => any_node(multiline_if_then),
    ReturnNil => "Style/ReturnNil" => node(as_return_node, return_nil),
    VariableInterpolation => "Style/VariableInterpolation" => node(as_embedded_variable_node, variable_interpolation),
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

fn empty_interpolation(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let Some(interpolation) = node.as_embedded_statements_node() else {
        return;
    };
    let source = context.source();
    let range = interpolation.location().start_offset()..interpolation.location().end_offset();
    let Some(inner) = source.get(range.start + 2..range.end.saturating_sub(1)) else {
        return;
    };
    if matches!(inner.trim(), "" | "''" | "\"\"" | "nil") && !inside_percent_word_array(context) {
        context.remove("Empty interpolation detected.", range.clone(), range);
    }
}

fn inside_percent_word_array(context: &CopContext<'_, '_>) -> bool {
    context.ancestors().iter().rev().any(|ancestor| {
        ancestor
            .as_array_node()
            .and_then(|array| array.opening_loc())
            .is_some_and(|opening| matches!(opening.as_slice(), b"%W[" | b"%I["))
    })
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
    let literal_ranges = context.source_file().literal_ranges();
    let comment_ranges = context.source_file().comment_ranges();
    let mut reported_through = 0;
    for (offset, character) in source.char_indices() {
        if literal_ranges
            .iter()
            .chain(comment_ranges.iter())
            .any(|range| range.start <= offset && offset < range.end)
        {
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

fn multiline_if_then(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (then_keyword, statements, keyword) = if let Some(if_node) = node.as_if_node() {
        (
            if_node.then_keyword_loc(),
            if_node.statements(),
            if_node
                .if_keyword_loc()
                .map(|location| context.source_file().at(&location))
                .unwrap_or("if"),
        )
    } else if let Some(unless_node) = node.as_unless_node() {
        (
            unless_node.then_keyword_loc(),
            unless_node.statements(),
            "unless",
        )
    } else {
        return;
    };
    let Some(then_keyword) = then_keyword else { return };
    if then_keyword.as_slice() != b"then" {
        return;
    }
    if statements.is_some_and(|statements| {
        context.source_file().same_line(
            then_keyword.start_offset(),
            statements.location().start_offset(),
        )
    }) {
        return;
    }
    let token = then_keyword.start_offset()..then_keyword.end_offset();
    let line_start = context.source_file().line_start(token.start);
    let edit = if context.source()[line_start..token.start].trim().is_empty() {
        line_start..line_end(context.source(), line_start)
    } else if context.source().as_bytes().get(token.end) == Some(&b' ') {
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

fn return_nil(node: &ruby_prism::ReturnNode<'_>, context: &mut CopContext<'_, '_>) {
    for ancestor in context.ancestors().iter().rev() {
        if ancestor.as_def_node().is_some() || ancestor.as_lambda_node().is_some() {
            break;
        }
        let Some(block) = ancestor.as_block_node() else {
            continue;
        };
        let owner = context.ancestors().iter().rev().find_map(|candidate| {
            let call = candidate.as_call_node()?;
            call.block()
                .and_then(|candidate| candidate.as_block_node())
                .filter(|candidate| {
                    candidate.location().start_offset() == block.location().start_offset()
                })?;
            Some(call)
        });
        if owner.as_ref().is_some_and(|owner| {
            matches!(
                owner.name().as_slice(),
                b"define_method" | b"define_singleton_method"
            )
        }) {
            break;
        }
        if block.parameters().is_some() && owner.is_some_and(|owner| owner.receiver().is_some()) {
            return;
        }
    }
    let return_nil = context.policy().enforced_style("return") == "return_nil";
    let arguments = node.arguments();
    if return_nil
        && arguments
            .as_ref()
            .is_none_or(|arguments| arguments.arguments().is_empty())
    {
        context.replace(
            "Use `return nil` instead of `return`.",
            node.location(),
            node.location(),
            "return nil",
        );
    } else if !return_nil
        && arguments.as_ref().is_some_and(|arguments| {
            arguments.arguments().len() == 1
                && arguments
                    .arguments()
                    .first()
                    .is_some_and(|argument| argument.as_nil_node().is_some())
        })
    {
        context.replace(
            "Use `return` instead of `return nil`.",
            node.location(),
            node.location(),
            "return",
        );
    }
}
