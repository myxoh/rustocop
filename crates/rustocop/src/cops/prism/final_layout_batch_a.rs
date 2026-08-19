use super::catalog_cop::{custom, replace, report};
use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        custom("Layout/LineContinuationSpacing", line_continuation_spacing),
        custom(
            "Layout/MultilineMethodDefinitionBraceLayout",
            method_definition_brace_layout,
        ),
        custom("Layout/ArrayAlignment", align_continuation),
        custom("Layout/SpaceInsideParens", space_inside_parens),
        custom("Layout/ClosingParenthesisIndentation", closing_parenthesis),
        custom("Layout/IndentationStyle", indentation_style),
        replace(
            "Layout/LeadingCommentSpace",
            "#comment",
            "# comment",
            "Missing space after `#`.",
        ),
        custom("Layout/CommentIndentation", comment_indentation),
        custom("Layout/ElseAlignment", keyword_alignment),
        custom(
            "Layout/AccessModifierIndentation",
            access_modifier_indentation,
        ),
        custom("Layout/MultilineHashBraceLayout", hash_brace_layout),
        custom("Layout/CaseIndentation", case_indentation),
        custom("Layout/MultilineArrayBraceLayout", array_brace_layout),
        custom("Layout/EmptyLineAfterGuardClause", empty_after_guard),
        custom(
            "Layout/LineEndStringConcatenationIndentation",
            align_continuation,
        ),
        custom("Layout/MultilineAssignmentLayout", multiline_assignment),
        custom("Layout/SpaceInsideBlockBraces", space_inside_block),
        replace(
            "Layout/SpaceInsideHashLiteralBraces",
            "{  ",
            "{ ",
            "Extra space inside hash braces detected.",
        ),
        custom("Layout/ArgumentAlignment", align_continuation),
        custom("Layout/FirstArrayElementIndentation", align_continuation),
        replace(
            "Layout/LineContinuationLeadingSpace",
            "\n .",
            "\n.",
            "Line continuation should not have leading space.",
        ),
        custom(
            "Layout/MultilineMethodCallBraceLayout",
            method_call_brace_layout,
        ),
        report(
            "Layout/MultilineBlockLayout",
            " { |",
            "Multi-line block argument must be on a separate line.",
        ),
    ]
}

fn array_brace_layout(context: &mut CopContext<'_, '_>) {
    brace_layout(context, '[', ']');
}

fn hash_brace_layout(context: &mut CopContext<'_, '_>) {
    brace_layout(context, '{', '}');
}

fn method_call_brace_layout(context: &mut CopContext<'_, '_>) {
    brace_layout(context, '(', ')');
}

fn method_definition_brace_layout(context: &mut CopContext<'_, '_>) {
    if context
        .source()
        .lines()
        .next()
        .is_some_and(|line| line.trim_start().starts_with("def "))
    {
        brace_layout(context, '(', ')');
    }
}

fn brace_layout(context: &mut CopContext<'_, '_>, open: char, close: char) {
    let style = context.policy().enforced_style("symmetrical").to_string();
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if !window[0].1.trim_end().ends_with(open) || window[1].1.trim() == close.to_string() {
            continue;
        }
        if matches!(style.as_str(), "symmetrical" | "new_line")
            && lines.iter().any(|(_, line)| {
                line.trim_end().ends_with(close) && line.trim() != close.to_string()
            })
        {
            context.report(
                "Closing brace must be on a new line.",
                window[0].0..window[1].0 + window[1].1.len(),
            );
        }
    }
}

fn align_continuation(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if window[0].1.trim_end().ends_with([',', '+', '\\'])
            && !window[1].1.trim().is_empty()
            && window[1].1.len() - window[1].1.trim_start().len() == 0
        {
            context.insert(
                "Use configured indentation for a continuation line.",
                window[1].0..window[1].0,
                window[1].0,
                "  ",
            );
        }
    }
}

fn line_continuation_spacing(context: &mut CopContext<'_, '_>) {
    let space_style = context.policy().enforced_style("space") == "space";
    for (offset, line) in context.source_file().lines() {
        if !line.trim_end().ends_with('\\')
            || line.trim_start().starts_with('#')
            || !line[..line.rfind('\\').unwrap_or(0)].contains(['+', '-', '=', '&', '|'])
        {
            continue;
        }
        let slash = line.rfind('\\').unwrap_or(0);
        let spaces = line[..slash].len() - line[..slash].trim_end().len();
        if space_style && spaces != 1 {
            context.replace(
                "Use one space in front of a line continuation.",
                offset + slash - spaces..offset + slash,
                offset + slash - spaces..offset + slash,
                " ",
            );
        } else if !space_style && spaces > 0 {
            context.remove(
                "Do not use space in front of a line continuation.",
                offset + slash - spaces..offset + slash,
                offset + slash - spaces..offset + slash,
            );
        }
    }
}

fn space_inside_parens(context: &mut CopContext<'_, '_>) {
    if context.policy().enforced_style("no_space") != "no_space" {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        for (at, _) in line.match_indices("( ") {
            if line.as_bytes().get(at + 2) != Some(&b')')
                && line.as_bytes().get(at + 2) != Some(&b'#')
            {
                context.remove(
                    "Space inside parentheses detected.",
                    offset + at + 1..offset + at + 2,
                    offset + at + 1..offset + at + 2,
                );
            }
        }
    }
}

fn space_inside_block(context: &mut CopContext<'_, '_>) {
    let no_space = context.policy().enforced_style("space") == "no_space";
    if !no_space {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        if let Some(at) = line.find("{ ") {
            if !line.contains('}')
                || (line[at + 2..].starts_with('|')
                    && context.config_bool("SpaceBeforeBlockParameters", true))
            {
                continue;
            }
            context.remove(
                "Space inside block braces detected.",
                offset + at + 1..offset + at + 2,
                offset + at + 1..offset + at + 2,
            );
        }
    }
}

fn multiline_assignment(context: &mut CopContext<'_, '_>) {
    if context.policy().enforced_style("same_line") != "same_line" {
        return;
    }
    for start in context.source_file().code_offsets(" =\n") {
        context.report(
            "Right hand side of a multi-line assignment must be on the same line.",
            start..start + 2,
        );
    }
}

fn closing_parenthesis(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        if line.starts_with(' ') && line.trim() == ")" && line.len() == 2 {
            let spaces = line.len() - 1;
            context.replace(
                "Indent `)` to match the opening parenthesis.",
                offset..offset + spaces,
                offset..offset + spaces,
                " ".repeat(spaces.saturating_sub(1)),
            );
        }
    }
}

fn indentation_style(context: &mut CopContext<'_, '_>) {
    if context.policy().enforced_style("spaces") != "spaces" || context.source().contains("<<") {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        let tabs = line.bytes().take_while(|byte| *byte == b'\t').count();
        if tabs > 0 {
            context.replace(
                "Tab detected in indentation.",
                offset..offset + tabs,
                offset..offset + tabs,
                "  ".repeat(tabs),
            );
        }
    }
}

fn comment_indentation(context: &mut CopContext<'_, '_>) {
    if context.config_bool("AllowForAlignment", false) {
        return;
    }
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(3) {
        if window[1].1.trim_start().starts_with('#')
            && !window[1].1.contains("rubocop:")
            && !window[0].1.trim_start().starts_with('#')
            && ![
                "if ", "unless ", "elsif ", "else", "case ", "when ", "in ", "rescue", "ensure",
                "begin", "def ",
            ]
            .iter()
            .any(|keyword| window[0].1.trim_start().starts_with(keyword))
            && !window[0].1.trim_end().ends_with(['{', '[', '('])
            && !["elsif ", "else", "when ", "in ", "rescue", "ensure"]
                .iter()
                .any(|keyword| window[2].1.trim_start().starts_with(keyword))
        {
            let expected = window[0].1.len() - window[0].1.trim_start().len();
            let actual = window[1].1.len() - window[1].1.trim_start().len();
            if expected != actual && !window[0].1.trim().is_empty() {
                context.replace(
                    "Incorrect indentation detected.",
                    window[1].0..window[1].0 + actual,
                    window[1].0..window[1].0 + actual,
                    " ".repeat(expected),
                );
            }
        }
    }
}

fn keyword_alignment(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        if matches!(line.trim(), "else" | "ensure" | "rescue")
            && line.len() - line.trim_start().len() % 2 == 1
        {
            context.report(
                "Incorrect indentation detected.",
                offset..offset + line.len(),
            );
        }
    }
}

fn access_modifier_indentation(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        if matches!(line.trim(), "private" | "protected" | "public")
            && (line.len() - line.trim_start().len()) % 2 == 1
        {
            context.report(
                "Incorrect indentation detected.",
                offset..offset + line.len(),
            );
        }
    }
}

fn case_indentation(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        if line.trim_start().starts_with("when ")
            && (line.len() - line.trim_start().len()) % 2 == 1
            && line.len() - line.trim_start().len() < 5
        {
            context.report(
                "Indent `when` as deep as `case`.",
                offset..offset + line.len(),
            );
        }
    }
}

fn empty_after_guard(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if (window[0].1.contains("return if ") || window[0].1.contains("next if "))
            && !window[1].1.trim().is_empty()
            && !window[1].1.trim_start().starts_with('#')
            && !matches!(window[1].1.trim(), "end" | "else" | "ensure" | "rescue")
            && !window[0].1.contains([';', '<'])
            && !window[0].1.contains("and return")
            && !window[1].1.trim_start().starts_with("if ")
        {
            context.insert(
                "Add empty line after guard clause.",
                window[0].0..window[0].0 + window[0].1.len(),
                window[0].0 + window[0].1.len() + 1,
                "\n",
            );
        }
    }
}
