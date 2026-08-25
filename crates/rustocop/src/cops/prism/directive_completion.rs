use super::*;

define_cops! {
    CopDirectiveSyntax => "Lint/CopDirectiveSyntax" => source(cop_directive_syntax),
    MissingCopEnableDirective => "Lint/MissingCopEnableDirective" => source(missing_cop_enable_directive),
}

fn cop_directive_syntax(context: &mut CopContext<'_, '_>) {
    let comment_ranges = context.source_file().comment_ranges();
    for (offset, line) in context.source_file().lines() {
        let Some(comment_start) = line.find("# rubocop:") else {
            continue;
        };
        let absolute_comment = offset + comment_start;
        if !comment_ranges.iter().any(|range| range.start == absolute_comment) {
            continue;
        }
        let trimmed = &line[comment_start..];
        let rest = trimmed["# rubocop:".len()..].trim_start();
        let mode = rest.split_whitespace().next().unwrap_or_default();
        let names = rest[mode.len()..]
            .split("--")
            .next()
            .unwrap_or_default()
            .trim();
        let message = if mode.is_empty() {
            Some("Malformed directive comment detected. The mode name is missing.")
        } else if !matches!(mode, "enable" | "disable" | "todo" | "push" | "pop") {
            Some("Malformed directive comment detected. The mode name must be one of `enable`, `disable`, `todo`, `push`, or `pop`.")
        } else if !matches!(mode, "push" | "pop") && rest[mode.len()..].trim().is_empty() {
            Some("Malformed directive comment detected. The cop name is missing.")
        } else if rest[mode.len()..].contains("# rubocop:")
            || rest[mode.len()..].contains(" == ")
            || names.split(',').any(|name| name.trim().ends_with(':'))
            || names.split(',').any(|name| name.trim().ends_with('/'))
            || names.ends_with(',')
            || names.split(',').any(|name| {
                name.trim().bytes().any(|byte| {
                    !(byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'_'))
                })
            })
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
            let start = offset + comment_start;
            context.report(message, start..offset + line.len());
        }
    }
}

fn missing_cop_enable_directive(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let maximum = context.config_usize("MaximumRangeSize", usize::MAX);
    let comment_ranges = context.source_file().comment_ranges();
    for (line_index, (offset, line)) in context.source_file().lines().enumerate() {
        let directive = line
            .find("# rubocop:disable ")
            .map(|start| (start, "# rubocop:disable "))
            .or_else(|| {
                line.find("# rubocop: disable ")
                    .map(|start| (start, "# rubocop: disable "))
            });
        let Some((comment_start, marker)) = directive else {
            continue;
        };
        if !comment_ranges
            .iter()
            .any(|range| range.start == offset + comment_start)
        {
            continue;
        }
        if !line[..comment_start].trim().is_empty() {
            continue;
        }
        let trimmed = &line[comment_start..];
        let names = &trimmed[marker.len()..];
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
        let names = names.split(',').map(str::trim).collect::<Vec<_>>();
        let prefer_explicit = names
            .iter()
            .any(|name| context.related_config_explicit(name, "Enabled"));
        for name in names {
            if prefer_explicit && !context.related_config_explicit(name, "Enabled") {
                continue;
            }
            if context.related_config_value("AllCops", "DisabledByDefault") == Some("true")
                && !context.related_config_explicit(name, "Enabled")
                && context.related_config_value(name, "Enabled").is_some()
            {
                continue;
            }
            if context.related_config_value(name, "Enabled") == Some("false") {
                continue;
            }
            let enable_line = source
                .lines()
                .enumerate()
                .skip(line_index + 1)
                .find_map(|(index, line)| {
                    let list = line.trim().strip_prefix("# rubocop:enable ")?;
                    list.split("--")
                        .next()
                        .unwrap_or_default()
                        .split(',')
                        .map(str::trim)
                        .any(|enabled| enabled == name)
                        .then_some(index)
                });
            let missing = enable_line.is_none();
            let too_far =
                enable_line.is_some_and(|index| index.saturating_sub(line_index + 1) > maximum);
            if !missing && !too_far {
                continue;
            }
            let kind = if matches!(
                name,
                "Bundler"
                    | "Gemspec"
                    | "Layout"
                    | "Lint"
                    | "Metrics"
                    | "Migration"
                    | "Naming"
                    | "Security"
                    | "Style"
            ) {
                "department"
            } else {
                "cop"
            };
            let limit = if maximum == usize::MAX {
                " with `# rubocop:enable`".to_string()
            } else {
                format!(" within {maximum} lines")
            };
            let start = offset + comment_start;
            context.report(
                format!("Re-enable {name} {kind}{limit} after disabling it."),
                start..offset + line.len(),
            );
            break;
        }
    }
}
