use super::catalog_cop::custom;
use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        custom("Metrics/BlockNesting", block_nesting),
        custom("Metrics/PerceivedComplexity", perceived_complexity),
        custom("Metrics/ClassLength", class_length),
        custom("Metrics/CyclomaticComplexity", cyclomatic_complexity),
    ]
}

fn block_nesting(context: &mut CopContext<'_, '_>) {
    let max = context.config_usize("Max", 3);
    let mut depth = 0_usize;
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        if ["if ", "unless ", "case ", "while ", "until ", "for "]
            .iter()
            .any(|keyword| trimmed.starts_with(keyword))
        {
            depth += 1;
            if depth > max {
                context.report(
                    format!("Avoid more than {max} levels of block nesting."),
                    offset..offset + line.len(),
                );
            }
        }
        if trimmed.trim() == "end" {
            depth = depth.saturating_sub(1);
        }
    }
}

fn perceived_complexity(context: &mut CopContext<'_, '_>) {
    complexity(context, true);
}

fn cyclomatic_complexity(context: &mut CopContext<'_, '_>) {
    complexity(context, false);
}

fn complexity(context: &mut CopContext<'_, '_>, perceived: bool) {
    let max = context.config_usize("Max", if perceived { 8 } else { 7 });
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let Some(start) = lines
        .iter()
        .position(|(_, line)| line.trim_start().starts_with("def "))
    else {
        return;
    };
    let method = lines[start].1.trim_start()[4..]
        .split(['(', ' '])
        .next()
        .unwrap_or("")
        .rsplit('.')
        .next()
        .unwrap_or("");
    if context
        .config_values("AllowedMethods")
        .iter()
        .any(|name| name == method)
        || !context.config_values("AllowedPatterns").is_empty()
    {
        return;
    }
    let end = lines[start + 1..]
        .iter()
        .position(|(_, line)| line.trim() == "end")
        .map_or(lines.len(), |relative| start + relative + 2);
    let source = lines[start..end]
        .iter()
        .map(|(_, line)| *line)
        .collect::<Vec<_>>()
        .join("\n");
    let score = 1
        + [
            " if ", " unless ", " while ", " until ", " && ", " || ", " rescue ",
        ]
        .iter()
        .map(|token| source.matches(token).count())
        .sum::<usize>()
        + if perceived {
            source.matches(" when ").count()
        } else {
            0
        };
    if score > max {
        let metric = if perceived {
            "Perceived complexity"
        } else {
            "Cyclomatic complexity"
        };
        context.report(
            format!("{metric} for `{method}` is too high. [{score}/{max}]"),
            lines[start].0..lines[end.saturating_sub(1)].0 + lines[end.saturating_sub(1)].1.len(),
        );
    }
}

fn class_length(context: &mut CopContext<'_, '_>) {
    let max = context.config_usize("Max", 100);
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut start = None;
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        if line.trim_start().starts_with("class ") {
            start = Some((index, offset));
        }
        if line.trim() == "end" {
            if let Some((start_index, start_offset)) = start.take() {
                let length = index.saturating_sub(start_index + 1);
                if length > max {
                    context.report(
                        format!("Class has too many lines. [{length}/{max}]"),
                        start_offset..offset + line.len(),
                    );
                }
            }
        }
    }
}
