use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        custom("Lint/ShadowedArgument", shadowed_argument),
        custom("Naming/InclusiveLanguage", inclusive_language),
    ]
}

fn shadowed_argument(context: &mut CopContext<'_, '_>) {
    if context.source().contains(" if ") || context.source().contains(" unless ") {
        return;
    }
    if context.config_bool("IgnoreImplicitReferences", false)
        && (context.source().contains("super") || context.source().contains("binding"))
    {
        return;
    }
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for (index, (_, line)) in lines.iter().copied().enumerate() {
        if line.trim_start().starts_with("def ") {
            let arguments = line
                .split_once('(')
                .and_then(|(_, rest)| rest.rsplit_once(')'))
                .map_or_else(Vec::new, |(args, _)| {
                    args.split(',').map(|arg| arg.trim().to_string()).collect()
                });
            let Some((offset, assignment)) = lines.get(index + 1).copied() else {
                continue;
            };
            for argument in &arguments {
                if let Some(at) = assignment.find(&format!("{argument} =")) {
                    if assignment[..at].contains('{') {
                        continue;
                    }
                    let used_later = lines[index + 2..]
                        .iter()
                        .take_while(|(_, line)| line.trim() != "end")
                        .any(|(_, line)| {
                            line.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                                .any(|word| word == argument)
                        });
                    if used_later {
                        context.report(
                            "Method argument is overwritten before it is used.",
                            offset + at..offset + at + argument.len(),
                        );
                    }
                }
            }
        }
    }
}

fn inclusive_language(context: &mut CopContext<'_, '_>) {
    for (old, new) in [
        ("whitelist", "allowlist"),
        ("blacklist", "denylist"),
        ("master", "primary"),
        ("slave", "replica"),
    ] {
        for start in context
            .source()
            .match_indices(old)
            .map(|(start, _)| start)
            .collect::<Vec<_>>()
        {
            let before = &context.source()[..start];
            let line_start = before.rfind('\n').map_or(0, |at| at + 1);
            let line_end = context.source()[start..]
                .find('\n')
                .map_or(context.source().len(), |len| start + len);
            let line = &context.source()[line_start..line_end];
            let in_string = line[..start - line_start].matches('"').count() % 2 == 1;
            let symbol = start > 0 && context.source().as_bytes()[start - 1] == b':';
            let variable =
                start > 0 && matches!(context.source().as_bytes()[start - 1], b'@' | b'$');
            if (!context.config_bool("CheckIdentifiers", true)
                && !in_string
                && !symbol
                && !variable)
                || (!context.config_bool("CheckVariables", true) && variable)
                || (!context.config_bool("CheckStrings", false) && in_string)
                || (!context.config_bool("CheckSymbols", true) && symbol)
            {
                continue;
            }
            context.replace(
                format!("Use inclusive language: replace `{old}` with `{new}`."),
                start..start + old.len(),
                start..start + old.len(),
                new,
            );
        }
    }
}
