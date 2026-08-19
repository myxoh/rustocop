use super::*;

define_cops! {
    CopDirectiveSyntax => "Lint/CopDirectiveSyntax" => source(cop_directive_syntax),
    MissingCopEnableDirective => "Lint/MissingCopEnableDirective" => source(missing_cop_enable_directive),
}

fn cop_directive_syntax(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix("# rubocop:") else {
            continue;
        };
        let mode = rest.split_whitespace().next().unwrap_or_default();
        let message = if mode.is_empty() {
            Some("Malformed directive comment detected. The mode name is missing.")
        } else if !matches!(mode, "enable" | "disable" | "todo" | "push" | "pop") {
            Some("Malformed directive comment detected. The mode name must be one of `enable`, `disable`, `todo`, `push`, or `pop`.")
        } else if !matches!(mode, "push" | "pop") && rest[mode.len()..].trim().is_empty() {
            Some("Malformed directive comment detected. The cop name is missing.")
        } else if rest[mode.len()..].contains("# rubocop:")
            || rest[mode.len()..].contains(" == ")
            || (!rest[mode.len()..].contains(',')
                && rest[mode.len()..]
                    .split("--")
                    .next()
                    .unwrap_or_default()
                    .split_whitespace()
                    .count()
                    > 1)
        {
            Some("Malformed directive comment detected. Cop names must be separated by commas. Comment in the directive must start with `--`.")
        } else {
            None
        };
        if let Some(message) = message {
            let start = offset + line.len() - trimmed.len();
            context.report(message, start..offset + line.len());
        }
    }
}

fn missing_cop_enable_directive(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let maximum = context.config_usize("MaximumRangeSize", usize::MAX);
    for (line_index, (offset, line)) in context.source_file().lines().enumerate() {
        let trimmed = line.trim_start();
        let Some(names) = trimmed.strip_prefix("# rubocop:disable ") else {
            continue;
        };
        let wrapped_by_push_pop = source
            .lines()
            .take(line_index)
            .any(|line| line.trim() == "# rubocop:push")
            && source
                .lines()
                .skip(line_index + 1)
                .any(|line| line.trim() == "# rubocop:pop");
        if wrapped_by_push_pop {
            continue;
        }
        for name in names.split(',').map(str::trim) {
            if context.related_config_value(name, "Enabled") == Some("false") {
                continue;
            }
            let enable = format!("# rubocop:enable {name}");
            let enable_line = source
                .lines()
                .enumerate()
                .skip(line_index + 1)
                .find_map(|(index, line)| line.trim().starts_with(&enable).then_some(index));
            let missing = enable_line.is_none();
            let too_far =
                enable_line.is_some_and(|index| index.saturating_sub(line_index + 1) > maximum);
            if !missing && !too_far {
                continue;
            }
            let kind = if name.contains('/') {
                "cop"
            } else {
                "department"
            };
            let limit = if maximum == usize::MAX {
                " with `# rubocop:enable`".to_string()
            } else {
                format!(" within {maximum} lines")
            };
            let start = offset + line.len() - trimmed.len();
            context.report(
                format!("Re-enable {name} {kind}{limit} after disabling it."),
                start..offset + line.len(),
            );
        }
    }
}
