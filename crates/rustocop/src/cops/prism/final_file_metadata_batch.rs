use super::catalog_cop::custom;
use super::*;
use std::collections::HashSet;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        custom("Lint/ScriptPermission", script_permission),
        custom("Style/MagicCommentFormat", magic_comment_format),
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
            context.report("Script file is not executable.", 0..2);
        }
    }
}

fn magic_comment_format(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines().take(3) {
        for (old, new) in [
            ("#frozen_string_literal:", "# frozen_string_literal:"),
            ("# encoding =", "# encoding:"),
            ("# coding =", "# coding:"),
        ] {
            if let Some(at) = line.find(old) {
                context.replace(
                    "Incorrect magic comment format.",
                    offset + at..offset + at + old.len(),
                    offset + at..offset + at + old.len(),
                    new,
                );
            }
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
