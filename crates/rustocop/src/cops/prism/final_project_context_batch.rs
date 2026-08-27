use super::catalog_cop::compatibility_custom;
use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        compatibility_custom("Bundler/GemComment", gem_comment),
        compatibility_custom("Gemspec/DependencyVersion", dependency_version),
    ]
}

fn gem_comment(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    if !context.path().ends_with("Gemfile") {
        return;
    }

    let ignored = context.config_values("IgnoredGems").to_vec();
    let only_for = context.config_values("OnlyFor").to_vec();
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for (index, (offset, line)) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("gem ") && !trimmed.starts_with("gem(") {
            continue;
        }

        let Some(name) = first_quoted_value(trimmed) else {
            continue;
        };
        if ignored.iter().any(|ignored| ignored == name) {
            continue;
        }

        let call_end = trimmed.find('#').unwrap_or(trimmed.len());
        let call = trimmed[..call_end].trim_end();
        let has_inline_comment = call_end < trimmed.len();
        let has_preceding_comment = index > 0
            && lines[index - 1].1.trim_start().starts_with('#')
            && lines[index - 1].0 + lines[index - 1].1.len() + 1 >= *offset;
        if has_inline_comment || has_preceding_comment {
            continue;
        }

        let checked = only_for.is_empty()
            || only_for.iter().any(|option| match option.as_str() {
                "version_specifiers" => has_positional_version(call),
                "restrictive_version_specifiers" => has_restrictive_version(call),
                option => call.contains(&format!("{option}:")),
            });
        if checked {
            let leading = line.len() - trimmed.len();
            context.report(
                "Missing gem description comment.",
                offset + leading..offset + leading + call.len(),
            );
        }
    }
}

fn dependency_version(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    let required = context.policy().enforced_style("required") != "forbidden";
    let allowed = context.config_values("AllowedGems").to_vec();
    let mut specification_variable = None;
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        if trimmed.contains("Gem::Specification.new") {
            specification_variable = trimmed
                .split('|')
                .nth(1)
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_string);
            continue;
        }
        let Some(variable) = specification_variable.as_deref() else {
            continue;
        };
        let receiver = format!("{variable}.");
        let Some(call) = trimmed.strip_prefix(&receiver) else {
            continue;
        };
        if ![
            "add_dependency",
            "add_runtime_dependency",
            "add_development_dependency",
        ]
        .iter()
        .any(|method| call.starts_with(method))
        {
            continue;
        }
        let Some(name) = first_quoted_value(call) else {
            continue;
        };
        if allowed.iter().any(|allowed| allowed == name) {
            continue;
        }

        let after_name = after_first_quoted(call).unwrap_or("");
        let version = quoted_values(after_name).into_iter().any(|value| {
            let value = value.trim_start();
            let value = value.trim_start_matches(['~', '<', '>', '=']).trim_start();
            value.as_bytes().first().is_some_and(u8::is_ascii_digit)
        });
        let commit = ["branch:", "ref:", "tag:"]
            .iter()
            .any(|keyword| after_name.contains(keyword));
        if required != (version || commit) {
            let leading = line.len() - trimmed.len();
            context.report(
                if required {
                    "Dependency version specification is required."
                } else {
                    "Dependency version specification is forbidden."
                },
                offset + leading..offset + line.len(),
            );
        }
    }
}

fn first_quoted_value(source: &str) -> Option<&str> {
    quoted_values(source).into_iter().next()
}

fn after_first_quoted(source: &str) -> Option<&str> {
    let bytes = source.as_bytes();
    let open = bytes.iter().position(|byte| matches!(byte, b'\'' | b'"'))?;
    let delimiter = bytes[open];
    let close = bytes[open + 1..]
        .iter()
        .position(|byte| *byte == delimiter)
        .map(|relative| open + 1 + relative)?;
    Some(&source[close + 1..])
}

fn quoted_values(source: &str) -> Vec<&str> {
    let bytes = source.as_bytes();
    let mut values = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if !matches!(bytes[cursor], b'\'' | b'"') {
            cursor += 1;
            continue;
        }
        let delimiter = bytes[cursor];
        let start = cursor + 1;
        cursor = start;
        while cursor < bytes.len() {
            if bytes[cursor] == delimiter && bytes.get(cursor.wrapping_sub(1)) != Some(&b'\\') {
                values.push(&source[start..cursor]);
                cursor += 1;
                break;
            }
            cursor += 1;
        }
    }
    values
}

fn has_positional_version(call: &str) -> bool {
    after_first_quoted(call).is_some_and(|rest| !quoted_values(rest).is_empty())
}

fn has_restrictive_version(call: &str) -> bool {
    after_first_quoted(call).is_some_and(|rest| {
        quoted_values(rest).into_iter().any(|version| {
            let first = version.trim_start().as_bytes().first().copied();
            matches!(first, Some(b'<' | b'~' | b'=' | b'0'..=b'9'))
        })
    })
}
