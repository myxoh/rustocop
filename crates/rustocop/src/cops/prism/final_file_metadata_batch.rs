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
    let mut disabled = HashSet::new();
    for (offset, line) in context.source_file().lines() {
        let Some(list) = line.split("rubocop:disable ").nth(1) else {
            continue;
        };
        for cop in list.split(',').map(str::trim) {
            if !disabled.insert(cop.to_string()) {
                let start = offset + line.find(cop).unwrap_or(0);
                context.report(
                    format!("Unnecessary disabling of {cop}."),
                    start..start + cop.len(),
                );
            }
        }
    }
}
