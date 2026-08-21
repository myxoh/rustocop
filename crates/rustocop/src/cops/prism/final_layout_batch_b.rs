use super::catalog_cop::{custom, replace};
use super::*;

mod registry;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    let mut cops = vec![
        replace(
            "Layout/SpaceAroundBlockParameters",
            "{|",
            "{ |",
            "Space missing around block parameters.",
        ),
        custom(
            "Layout/SpaceInsideReferenceBrackets",
            reference_bracket_spacing,
        ),
        replace(
            "Layout/SpaceAroundMethodCallOperator",
            " &. ",
            "&.",
            "Space around method call operator detected.",
        ),
        custom(
            "Layout/MultilineOperationIndentation",
            continuation_indentation,
        ),
        replace(
            "Layout/RedundantLineBreak",
            "(\nvalue\n)",
            "value",
            "Redundant line break detected.",
        ),
        custom(
            "Layout/SpaceInsideArrayLiteralBrackets",
            array_literal_spacing,
        ),
    ];
    cops.extend(registry::cops());
    cops
}

fn empty_after_inclusion(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if ["include ", "extend ", "prepend "]
            .iter()
            .any(|keyword| window[0].1.trim_start().starts_with(keyword))
            && !window[1].1.trim().is_empty()
            && !window[1].1.trim_start().starts_with('#')
            && !matches!(window[1].1.trim(), "end" | "else" | "ensure" | "rescue")
            && !window[0].1.contains(" do")
            && !["include ", "extend ", "prepend "]
                .iter()
                .any(|keyword| window[1].1.trim_start().starts_with(keyword))
        {
            context.insert(
                "Add an empty line after module inclusion.",
                window[0].0..window[0].0 + window[0].1.len(),
                window[0].0 + window[0].1.len() + 1,
                "\n",
            );
        }
    }
}

fn array_literal_spacing(context: &mut CopContext<'_, '_>) {
    let no_space = context.policy().enforced_style("no_space") == "no_space";
    let empty_no_space = context
        .config_value("EnforcedStyleForEmptyBrackets")
        .unwrap_or("no_space")
        == "no_space";
    if !no_space {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        for (at, _) in line.match_indices("[ ") {
            if line[at + 2..].trim_start().starts_with('#') {
                continue;
            }
            let before = line[..at].trim_end().as_bytes().last().copied();
            if before.is_none() || before.is_some_and(|byte| matches!(byte, b'=' | b'(' | b',')) {
                if line.as_bytes().get(at + 2) == Some(&b']') && !empty_no_space {
                    continue;
                }
                context.remove(
                    "Space inside array brackets detected.",
                    offset + at + 1..offset + at + 2,
                    offset + at + 1..offset + at + 2,
                );
            }
        }
    }
}

fn reference_bracket_spacing(context: &mut CopContext<'_, '_>) {
    if context.policy().enforced_style("no_space") != "no_space" {
        return;
    }
    let empty_no_space = context
        .config_value("EnforcedStyleForEmptyBrackets")
        .unwrap_or("no_space")
        == "no_space";
    for (offset, line) in context.source_file().lines() {
        for (at, _) in line.match_indices("[ ") {
            let receiver = line[..at].trim_end().as_bytes().last().copied();
            if !receiver.is_some_and(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b')' | b']' | b'}')
            }) || (line.as_bytes().get(at + 2) == Some(&b']') && !empty_no_space)
            {
                continue;
            }
            context.remove(
                "Space inside reference brackets detected.",
                offset + at + 1..offset + at + 2,
                offset + at + 1..offset + at + 2,
            );
        }
    }
}

fn empty_between_defs(context: &mut CopContext<'_, '_>) {
    if context
        .config_values("NumberOfEmptyLines")
        .iter()
        .any(|value| value == "0")
    {
        return;
    }
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if window[0].1.trim() == "end" && window[1].1.trim_start().starts_with("def ") {
            context.insert(
                "Use empty lines between method definitions.",
                window[1].0..window[1].0,
                window[1].0,
                "\n",
            );
        }
    }
}

fn end_alignment(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        if matches!(line.trim(), "end" | "rescue" | "ensure")
            && (line.len() - line.trim_start().len()) % 2 == 1
            && line.len() - line.trim_start().len() <= 3
        {
            context.report(
                "`end` is not aligned with its opening keyword.",
                offset..offset + line.len(),
            );
        }
    }
}

fn continuation_indentation(context: &mut CopContext<'_, '_>) {
    if context
        .source()
        .lines()
        .next()
        .is_some_and(|line| line.trim_start().starts_with('('))
    {
        return;
    }
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if window[0].1.trim_end().ends_with(['(', ',', '+'])
            && !window[0].1.trim_start().starts_with('(')
            && !window[1].1.trim().is_empty()
            && !window[1].1.starts_with("  ")
        {
            context.insert(
                "Use configured indentation for a multi-line expression.",
                window[1].0..window[1].0,
                window[1].0,
                "  ",
            );
        }
    }
}

fn heredoc_parenthesis(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if window[0]
            .1
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte == b'_')
            && window[1].1.trim() == ")"
        {
            context.report(
                "Put the closing parenthesis on the same line as the heredoc terminator.",
                window[1].0..window[1].0 + window[1].1.len(),
            );
        }
    }
}

fn hash_alignment(context: &mut CopContext<'_, '_>) {
    if context
        .config_values("EnforcedHashRocketStyle")
        .iter()
        .any(|style| style == "table")
    {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        if line.contains("=>") && line.contains("  =>") {
            let at = line.find("  =>").unwrap_or(0);
            context.remove(
                "Align the elements of a hash literal.",
                offset + at..offset + at + 2,
                offset + at..offset + at + 1,
            );
        }
    }
}

fn dot_position(context: &mut CopContext<'_, '_>) {
    if context.policy().enforced_style("leading") != "leading" {
        return;
    }
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if window[0].1.trim_end().ends_with('.')
            && !window[1].1.trim().is_empty()
            && !window[1].1.trim_start().starts_with('#')
        {
            let dot = window[0].0 + window[0].1.rfind('.').unwrap_or(0);
            context.replace(
                "Place the dot at the beginning of the next line.",
                dot..dot + 2,
                dot..dot + 2,
                "\n.",
            );
        }
    }
}

fn operator_spacing(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        if line.trim_start().starts_with("def ") {
            continue;
        }
        for operator in ['+', '='] {
            for (at, _) in line.match_indices(operator) {
                if at > 0
                    && line.as_bytes()[at - 1].is_ascii_alphanumeric()
                    && line
                        .as_bytes()
                        .get(at + 1)
                        .is_some_and(u8::is_ascii_alphanumeric)
                {
                    context.replace(
                        "Surrounding space missing for operator.",
                        offset + at..offset + at + 1,
                        offset + at..offset + at + 1,
                        format!(" {operator} "),
                    );
                }
            }
        }
    }
}

fn empty_around_access_modifier(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        if matches!(line.trim(), "private" | "protected" | "public")
            && index > 0
            && !lines[index - 1].1.trim().is_empty()
            && !lines[index - 1].1.trim_start().starts_with('#')
            && index + 1 < lines.len()
            && lines[index + 1].1.trim_start().starts_with("def ")
        {
            context.insert(
                "Keep a blank line before access modifiers.",
                offset..offset + line.len(),
                offset,
                "\n",
            );
        }
    }
}

fn heredoc_indentation(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        if line
            .trim()
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte == b'_')
            && line.starts_with(' ')
        {
            let spaces = line.len() - line.trim_start().len();
            context.remove(
                "Use configured indentation for heredoc terminators.",
                offset..offset + spaces,
                offset..offset + spaces,
            );
        }
    }
}
