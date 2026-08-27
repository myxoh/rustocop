use super::*;

define_cops! {
    CopDirectiveSyntax => "Lint/CopDirectiveSyntax" => compatibility_source(cop_directive_syntax),
    MissingCopEnableDirective => "Lint/MissingCopEnableDirective" => compatibility_source(missing_cop_enable_directive),
}

fn cop_directive_syntax(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    let comment_ranges = context.comment_ranges();
    for (offset, line) in context.source_file().lines() {
        let Some(comment_start) = line.find("# rubocop:") else {
            continue;
        };
        let absolute_comment = offset + comment_start;
        if !comment_ranges
            .iter()
            .any(|range| range.start == absolute_comment)
        {
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
                name.trim()
                    .bytes()
                    .any(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'_')))
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

fn missing_cop_enable_directive(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    let source = context.source();
    let maximum = context.config_usize("MaximumRangeSize", usize::MAX);
    let comment_ranges = context.comment_ranges();
    for (line_index, (offset, line)) in context.source_file().lines().enumerate() {
        let directive = line
            .find("# rubocop:disable ")
            .map(|start| (start, "# rubocop:disable "))
            .or_else(|| {
                line.find("# rubocop: disable ")
                    .map(|start| (start, "# rubocop: disable "))
            })
            .or_else(|| {
                line.find("# rubocop:todo ")
                    .map(|start| (start, "# rubocop:todo "))
            })
            .or_else(|| {
                line.find("# rubocop: todo ")
                    .map(|start| (start, "# rubocop: todo "))
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
        let names = trimmed[marker.len()..]
            .split("--")
            .next()
            .unwrap_or_default()
            .trim();
        let push_depth =
            source
                .lines()
                .take(line_index)
                .fold(0usize, |depth, line| match line.trim() {
                    "# rubocop:push" | "# rubocop: push" => depth + 1,
                    "# rubocop:pop" | "# rubocop: pop" => depth.saturating_sub(1),
                    _ => depth,
                });
        let mut future_depth = push_depth;
        let wrapped_by_push_pop = push_depth > 0
            && source.lines().skip(line_index + 1).any(|line| {
                match line.trim() {
                    "# rubocop:push" | "# rubocop: push" => future_depth += 1,
                    "# rubocop:pop" | "# rubocop: pop" => {
                        future_depth = future_depth.saturating_sub(1)
                    }
                    _ => {}
                }
                future_depth < push_depth
            });
        if wrapped_by_push_pop {
            continue;
        }
        let names = names.split(',').map(str::trim).collect::<Vec<_>>();
        for name in names {
            if name.is_empty()
                || name
                    .bytes()
                    .any(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'_')))
            {
                continue;
            }
            let built_in_department = name.split_once('/').is_some_and(|(department, _)| {
                matches!(
                    department,
                    "Bundler"
                        | "Gemspec"
                        | "Layout"
                        | "Lint"
                        | "Metrics"
                        | "Migration"
                        | "Naming"
                        | "Security"
                        | "Style"
                )
            });
            let registered_built_in =
                built_in_department && context.related_config_value(name, "Enabled").is_some();
            if maximum == usize::MAX
                && registered_built_in
                && !context.cop_enabled(name)
                && !context.related_cop_normally_enabled(name)
            {
                continue;
            }
            let enable_line =
                source
                    .lines()
                    .enumerate()
                    .skip(line_index + 1)
                    .find_map(|(index, line)| {
                        let trimmed = line.trim();
                        let list = trimmed
                            .strip_prefix("# rubocop:enable ")
                            .or_else(|| trimmed.strip_prefix("# rubocop: enable "))?;
                        list.split("--")
                            .next()
                            .unwrap_or_default()
                            .split(',')
                            .map(str::trim)
                            .any(|enabled| {
                                enabled == name
                                    || name
                                        .split_once('/')
                                        .is_some_and(|(department, _)| enabled == department)
                            })
                            .then_some(index)
                    });
            let repeated_disable_line =
                source
                    .lines()
                    .enumerate()
                    .skip(line_index + 1)
                    .find_map(|(index, line)| {
                        directive_names(
                            line,
                            &[
                                "# rubocop:disable ",
                                "# rubocop: disable ",
                                "# rubocop:todo ",
                                "# rubocop: todo ",
                            ],
                        )?
                        .iter()
                        .any(|disabled| *disabled == name)
                        .then_some(index)
                    });
            if repeated_disable_line
                .is_some_and(|repeat| enable_line.is_none_or(|enable| repeat < enable))
            {
                continue;
            }
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

fn directive_names<'a>(line: &'a str, markers: &[&str]) -> Option<Vec<&'a str>> {
    let trimmed = line.trim();
    let list = markers
        .iter()
        .find_map(|marker| trimmed.strip_prefix(marker))?;
    Some(
        list.split("--")
            .next()
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .collect(),
    )
}
