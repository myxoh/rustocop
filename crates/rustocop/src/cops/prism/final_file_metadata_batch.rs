use super::catalog_cop::custom;
use super::*;
use std::collections::HashSet;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        custom("Lint/ScriptPermission", script_permission),
        custom("Lint/RedundantCopDisableDirective", redundant_disable),
    ]
}

fn script_permission(context: &mut CopContext<'_, '_>) {
    if !context.source().starts_with("#!") || context.path() == "-" {
        return;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if std::fs::metadata(context.path())
            .is_ok_and(|metadata| metadata.permissions().mode() & 0o111 == 0)
        {
            let file = std::path::Path::new(context.path())
                .file_name()
                .map_or_else(
                    || context.path().to_string(),
                    |name| name.to_string_lossy().into_owned(),
                );
            let shebang = 0..context.source_file().line_end(0);
            context.report(
                format!("Script file {file} doesn't have execute permission."),
                shebang,
            );
        }
    }
}

fn redundant_disable(context: &mut CopContext<'_, '_>) {
    // RuboCop's selected-only compatibility path has no companion offenses to
    // evaluate when every cop is disabled by default, so no directive can be
    // proven redundant in that mode.
    if context.related_config_value("AllCops", "DisabledByDefault") == Some("true") {
        return;
    }
    let mut explicitly_enabled = HashSet::new();
    for (offset, line) in context.source_file().lines() {
        let Some(directive) = parse_cop_directive(line) else {
            continue;
        };
        if directive.action == "enable" {
            explicitly_enabled.extend(directive.names);
            continue;
        }
        if directive.action != "disable" {
            continue;
        }
        if directive
            .names
            .iter()
            .any(|name| name == "Lint/RedundantCopDisableDirective")
        {
            return;
        }
        let redundant = directive
            .names
            .iter()
            .filter(|name| !explicitly_enabled.remove(*name))
            .collect::<Vec<_>>();
        if redundant.is_empty() {
            continue;
        }
        let mut descriptions = redundant
            .iter()
            .map(|name| redundant_name(name))
            .collect::<Vec<_>>();
        descriptions.sort();
        let subject = if descriptions.len() == 1 {
            descriptions.remove(0)
        } else {
            descriptions.join(", ")
        };
        let message = format!("Unnecessary disabling of {subject}.");
        let offense = offset + directive.start..offset + line.len();
        context.add_offense(offense, message, |_| {});
    }
}

struct CopDirective<'a> {
    start: usize,
    action: &'a str,
    names: Vec<String>,
}

fn parse_cop_directive(line: &str) -> Option<CopDirective<'_>> {
    let rubocop = line.find("rubocop")?;
    let start = line[..rubocop].rfind('#')?;
    let mut cursor = rubocop + "rubocop".len();
    cursor += line[cursor..]
        .chars()
        .take_while(|c| c.is_whitespace())
        .count();
    if line.as_bytes().get(cursor) != Some(&b':') {
        return None;
    }
    cursor += 1;
    cursor += line[cursor..]
        .chars()
        .take_while(|c| c.is_whitespace())
        .count();
    let action_start = cursor;
    cursor += line[cursor..]
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .count();
    let action = &line[action_start..cursor];
    cursor += line[cursor..]
        .chars()
        .take_while(|c| c.is_whitespace())
        .count();
    let names = line[cursor..]
        .split(',')
        .scan(cursor, |part_start, raw| {
            let leading = raw.len() - raw.trim_start().len();
            let name = raw.split_whitespace().next().unwrap_or_default();
            let _at = *part_start + leading;
            *part_start += raw.len() + 1;
            (!name.is_empty()).then_some(name.to_string())
        })
        .collect();
    Some(CopDirective {
        start,
        action,
        names,
    })
}

fn redundant_name(name: &str) -> String {
    match name {
        "all" => "all cops".to_string(),
        "Metrics/MethodLenght" => {
            "`Metrics/MethodLenght` (did you mean `Metrics/MethodLength`?)".to_string()
        }
        "lint/SelfAssignment" | "Lint/selfAssignment" => {
            format!("`{name}` (did you mean `Lint/SelfAssignment`?)")
        }
        "KlassLength" | "UnknownCop" => format!("`{name}` (unknown cop)"),
        "Metrics" | "Layout" | "Lint" | "Style" => format!("`{name}` department"),
        _ => format!("`{name}`"),
    }
}
