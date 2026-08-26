use std::collections::HashSet;

use super::source_helpers::*;
use super::*;

mod interpolation;
use interpolation::*;

define_cops! {
    InitialIndentation => "Layout/InitialIndentation" => source(initial_indentation),
    DuplicateMagicComment => "Lint/DuplicateMagicComment" => source(duplicate_magic_comment),
    DoubleCopDisableDirective => "Style/DoubleCopDisableDirective" => source(double_cop_disable_directive),
    EmptyInterpolation => "Lint/EmptyInterpolation" => any_node(empty_interpolation),
    RequireRangeParentheses => "Lint/RequireRangeParentheses" => node(as_range_node, require_range_parentheses),
    AsciiIdentifiers => "Naming/AsciiIdentifiers" => source(ascii_identifiers),
    MultilineIfThen => "Style/MultilineIfThen" => any_node(multiline_if_then),
    ReturnNil => "Style/ReturnNil" => node(as_return_node, return_nil),
    InPatternThen => "Style/InPatternThen" => node(as_in_node, in_pattern_then),
    EmptyEnsure => "Lint/EmptyEnsure" => node(as_begin_node, empty_ensure),
    BigDecimalNew => "Lint/BigDecimalNew" => call(big_decimal_new),
    ColonMethodDefinition => "Style/ColonMethodDefinition" => node(as_def_node, colon_method_definition),
    EnsureReturn => "Lint/EnsureReturn" => node(as_begin_node, ensure_return),
    VariableInterpolation => "Style/VariableInterpolation" => node(as_embedded_variable_node, variable_interpolation),
}

fn ensure_return(node: &ruby_prism::BeginNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(ensure) = node.ensure_clause() else {
        return;
    };
    #[derive(Default)]
    struct Returns(Vec<std::ops::Range<usize>>);
    impl<'pr> Visit<'pr> for Returns {
        fn visit_return_node(&mut self, node: &ruby_prism::ReturnNode<'pr>) {
            self.0
                .push(node.location().start_offset()..node.location().end_offset());
            ruby_prism::visit_return_node(self, node);
        }
    }
    let mut returns = Returns::default();
    if let Some(statements) = ensure.statements() {
        returns.visit(&statements.as_node());
    }
    for offense in returns.0 {
        context.report("Do not return from an `ensure` block.", offense);
    }
}

fn colon_method_definition(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(operator) = node
        .operator_loc()
        .filter(|operator| operator.as_slice() == b"::")
    else {
        return;
    };
    let range = operator.start_offset()..operator.end_offset();
    context.replace(
        "Do not use `::` for defining class methods.",
        range.clone(),
        range,
        ".",
    );
}

fn big_decimal_new(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.name().as_slice() != b"new" {
        return;
    }
    let Some(receiver) = node.receiver() else {
        return;
    };
    let receiver_source = context.source_file().node(&receiver);
    if !matches!(receiver_source, "BigDecimal" | "::BigDecimal") {
        return;
    }
    let (Some(selector), Some(dot)) = (node.message_loc(), node.call_operator_loc()) else {
        return;
    };
    let mut edits = vec![(dot.start_offset()..selector.end_offset(), String::new())];
    if receiver_source.starts_with("::") {
        edits.push((
            receiver.location().start_offset()..receiver.location().start_offset() + 2,
            String::new(),
        ));
    }
    context.replace_many(
        "`BigDecimal.new()` is deprecated. Use `BigDecimal()` instead.",
        selector,
        edits,
    );
}

fn empty_ensure(node: &ruby_prism::BeginNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(ensure) = node.ensure_clause() else {
        return;
    };
    if ensure
        .statements()
        .is_none_or(|statements| statements.body().is_empty())
    {
        let keyword = ensure.ensure_keyword_loc();
        let range = keyword.start_offset()..keyword.end_offset();
        context.remove("Empty `ensure` block detected.", range.clone(), range);
    }
}

fn in_pattern_then(node: &ruby_prism::InNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(body) = node
        .statements()
        .and_then(|statements| statements.body().first())
    else {
        return;
    };
    let pattern_node = node.pattern();
    let pattern_location = pattern_node.location();
    if !context.source_file().same_line(
        pattern_location.start_offset(),
        body.location().start_offset(),
    ) {
        return;
    }
    let gap = pattern_location.end_offset()..body.location().start_offset();
    let Some(relative_separator) = context.source()[gap.clone()].find(';') else {
        return;
    };
    let separator = gap.start + relative_separator;
    let pattern = context.source_file().node(&pattern_node);
    context.replace(
        format!("Do not use `in {pattern};`. Use `in {pattern} then` instead."),
        separator..separator + 1,
        separator..separator + 1,
        " then",
    );
}

fn double_cop_disable_directive(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for comment in ruby_prism::parse(source.as_bytes()).comments() {
        let location = comment.location();
        let text = &source[location.start_offset()..location.end_offset()];
        let (marker, directive) = if text.matches("# rubocop:disable ").count() > 1 {
            ("# rubocop:disable ", "disable")
        } else if text.matches("# rubocop:todo ").count() > 1 {
            ("# rubocop:todo ", "todo")
        } else {
            continue;
        };
        let relative = text.find(marker).expect("duplicate directive marker");
        if relative != 0 {
            continue;
        }
        let start = location.start_offset() + relative;
        let names = text[relative..]
            .split(marker)
            .filter(|name| !name.is_empty())
            .map(str::trim)
            .collect::<Vec<_>>()
            .join(", ");
        context.replace(
            "More than one disable comment on one line.",
            start..location.end_offset(),
            start..location.end_offset(),
            format!("# rubocop:{directive} {names}"),
        );
    }
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
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            break;
        }
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

fn require_range_parentheses(node: &ruby_prism::RangeNode<'_>, context: &mut CopContext<'_, '_>) {
    if context
        .ancestors()
        .iter()
        .rev()
        .find(|parent| parent.as_statements_node().is_none())
        .is_some_and(|parent| parent.as_parentheses_node().is_some())
    {
        return;
    }
    let (Some(left), Some(right)) = (node.left(), node.right()) else {
        return;
    };
    let operator = node.operator_loc();
    let operator_line = context.source()[..operator.start_offset()].rfind('\n');
    let right_line = context.source()[..right.location().start_offset()].rfind('\n');
    if operator_line == right_line {
        return;
    }
    let prefix = format!(
        "{}{}",
        context.source_file().node(&left),
        context.source_file().at(&operator)
    );
    context.report(
        format!("Wrap the endless range literal `{prefix}` to avoid precedence ambiguity."),
        node.location(),
    );
}

fn ascii_identifiers(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let ascii_constants = context.config_bool("AsciiConstants", true);
    let mut excluded_ranges = context.source_file().literal_ranges();
    excluded_ranges.extend(context.source_file().comment_ranges());
    excluded_ranges.sort_by_key(|range| (range.start, range.end));
    let excluded_ranges = excluded_ranges.into_iter().fold(
        Vec::<std::ops::Range<usize>>::new(),
        |mut merged, range| {
            if let Some(previous) = merged.last_mut() {
                if range.start < previous.end {
                    previous.end = previous.end.max(range.end);
                    return merged;
                }
            }
            merged.push(range);
            merged
        },
    );
    let data_section = source
        .find("\n__END__\n")
        .map_or(source.len(), |offset| offset + 1);
    let mut reported_through = 0;
    let mut excluded_index = 0;
    for (offset, character) in source.char_indices() {
        if offset >= data_section {
            break;
        }
        while excluded_ranges
            .get(excluded_index)
            .is_some_and(|range| range.end <= offset)
        {
            excluded_index += 1;
        }
        if excluded_ranges
            .get(excluded_index)
            .is_some_and(|range| range.start <= offset && offset < range.end)
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
    let Some(then_keyword) = then_keyword else {
        return;
    };
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
    // Parser represents a send-with-block as `(block (send ...arguments...) ...)`, so
    // a return inside one of those arguments still has the outer block as an
    // ancestor. Prism stores the arguments and block as siblings on the call.
    // Restore Parser's ancestry here before applying RuboCop's block-scope test.
    for ancestor in context.ancestors().iter().rev() {
        if ancestor.as_def_node().is_some() || ancestor.as_lambda_node().is_some() {
            break;
        }
        let Some(call) = ancestor.as_call_node() else {
            continue;
        };
        if matches!(
            call.name().as_slice(),
            b"define_method" | b"define_singleton_method"
        ) {
            break;
        }
        if call.receiver().is_some()
            && call
                .block()
                .and_then(|block| block.as_block_node())
                .is_some_and(|block| block.parameters().is_some())
        {
            return;
        }
    }
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
