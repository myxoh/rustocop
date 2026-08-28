use super::*;

define_cops! {
    RequiredRubyVersion => "Gemspec/RequiredRubyVersion" => source(required_ruby_version),
    ClassStructure => "Layout/ClassStructure" => source(class_structure),
    ModuleLength => "Metrics/ModuleLength" => source(module_length),
    EmptyLineAfterMultilineCondition => "Layout/EmptyLineAfterMultilineCondition" => source(empty_after_multiline_condition),
    DeprecatedOpenSSLConstant => "Lint/DeprecatedOpenSSLConstant" => call(deprecated_openssl),
}

fn required_ruby_version(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    if !source.contains("required_ruby_version") {
        if context.path().ends_with(".gemspec") {
            context.report("`required_ruby_version` should be specified.", 0..0);
        }
        return;
    }
    let target = context.target_ruby_version();
    let target_text = format!("{}.{}", target.major(), target.minor());
    for (offset, line) in context.source_file().lines() {
        if !line.contains("required_ruby_version") {
            continue;
        }
        let assigned = line.split_once('=').map_or("", |(_, value)| value.trim());
        if assigned.starts_with('[') && assigned != "[]" && !assigned.contains(['\'', '"']) {
            continue;
        }
        if requirement_includes_target(line, target.major(), target.minor()) {
            continue;
        }
        if let Some(start) = line
            .find("Gem::Requirement.new")
            .or_else(|| line.find('['))
            .or_else(|| line.find(['\'', '"']))
        {
            let end = if line[start..].starts_with("Gem::Requirement.new") {
                line[start..]
                    .find(')')
                    .map_or(line.len(), |at| start + at + 1)
            } else if line.as_bytes().get(start) == Some(&b'[') {
                line.rfind(']').map_or(line.len(), |at| at + 1)
            } else {
                line[start + 1..]
                    .find(['\'', '"'])
                    .map_or(line.len(), |at| start + at + 2)
            };
            context.report(format!("`required_ruby_version` and `TargetRubyVersion` ({target_text}, which may be specified in .rubocop.yml) should be equal."), offset + start..offset + end);
        }
    }
}

fn requirement_includes_target(line: &str, target_major: u16, target_minor: u16) -> bool {
    let mut rest = line;
    while let Some(open) = rest.find(['\'', '"']) {
        let quote = rest.as_bytes()[open];
        let after = &rest[open + 1..];
        let Some(close) = after.as_bytes().iter().position(|byte| *byte == quote) else {
            break;
        };
        let requirement = after[..close].trim();
        let version = requirement
            .strip_prefix(">=")
            .or_else(|| requirement.strip_prefix("~>"))
            .unwrap_or(requirement)
            .trim();
        if !requirement.starts_with('<') && version_matches(version, target_major, target_minor) {
            return true;
        }
        rest = &after[close + 1..];
    }
    false
}

fn version_matches(version: &str, target_major: u16, target_minor: u16) -> bool {
    let mut components = version.split('.');
    let major = components
        .next()
        .and_then(|value| value.parse::<u16>().ok());
    let minor = components
        .next()
        .and_then(|value| value.parse::<u16>().ok());
    major == Some(target_major) && minor == Some(target_minor)
}

fn class_structure(context: &mut CopContext<'_, '_>) {
    let mut seen_instance = false;
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("def ") && !trimmed.starts_with("def self.") {
            seen_instance = true;
        }
        if seen_instance && trimmed.starts_with("def self.") {
            let indent = line.len() - trimmed.len();
            context.report(
                "`public_class_methods` is supposed to appear before `public_methods`.",
                offset + indent..offset + line.len(),
            );
        }
    }
}

fn module_length(context: &mut CopContext<'_, '_>) {
    let max = context.config_usize("Max", 100);
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut stack = Vec::new();
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        if line.trim_start().starts_with("module ") {
            stack.push((index, offset));
        }
        if line.trim() == "end" {
            let Some((start_index, start)) = stack.pop() else {
                continue;
            };
            let count = index.saturating_sub(start_index + 1);
            if count > max {
                context.report(
                    format!("Module has too many lines. [{count}/{max}]"),
                    start..offset + line.len(),
                );
            }
        }
    }
}

fn empty_after_multiline_condition(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for index in 0..lines.len().saturating_sub(2) {
        let (offset, line) = lines[index];
        if !["if ", "unless ", "while ", "until "]
            .iter()
            .any(|word| line.contains(word))
            || !["&&", "||"].iter().any(|op| line.trim_end().ends_with(op))
        {
            continue;
        }
        if lines[index + 2].1.trim().is_empty() {
            continue;
        }
        let start = offset + line.find(['i', 'u', 'w']).unwrap_or(0);
        let end = lines[index + 1].0 + lines[index + 1].1.len();
        context.insert(
            "Use empty line after multiline condition.",
            start..end,
            end + 1,
            "\n",
        );
    }
}

fn deprecated_openssl(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !matches!(call_name(node), b"new" | b"digest") {
        return;
    }
    let Some(receiver) = node.receiver() else {
        return;
    };
    let Some(path) = constant_path(&receiver) else {
        return;
    };
    if path.len() != 3
        || path[0] != b"OpenSSL"
        || !matches!(path[1], b"Cipher" | b"Digest")
        || path[1] == b"Digest" && path[2] == b"Digest"
        || rejected_openssl_argument(node)
    {
        return;
    }

    let algorithm = String::from_utf8_lossy(path[2]);
    let method = String::from_utf8_lossy(call_name(node));
    let replacement_args = if path[1] == b"Cipher" {
        cipher_replacement_args(node, &algorithm, context)
    } else {
        let mut arguments = vec![format!("'{algorithm}'")];
        if let Some(call_arguments) = node.arguments() {
            arguments.extend(
                call_arguments
                    .arguments()
                    .iter()
                    .map(|argument| context.source_file().node(&argument).to_string()),
            );
        }
        arguments.join(", ")
    };
    let parent = String::from_utf8_lossy(path[1]);
    let replacement = format!("OpenSSL::{parent}.{method}({replacement_args})");
    let original = context.source_file().node(&node.as_node());
    context.replace_call(
        node,
        format!("Use `{replacement}` instead of `{original}`."),
        replacement,
    );
}

fn rejected_openssl_argument(node: &CallNode<'_>) -> bool {
    node.arguments().is_some_and(|arguments| {
        arguments.arguments().iter().any(|argument| {
            argument.as_local_variable_read_node().is_some()
                || argument.as_instance_variable_read_node().is_some()
                || argument.as_class_variable_read_node().is_some()
                || argument.as_global_variable_read_node().is_some()
                || argument.as_call_node().is_some()
                || constant_path(&argument).is_some()
        })
    })
}

fn cipher_replacement_args(
    node: &CallNode<'_>,
    algorithm: &str,
    context: &CopContext<'_, '_>,
) -> String {
    if algorithm == "Cipher" {
        return first_argument(node)
            .map(|argument| context.source_file().node(&argument).to_string())
            .unwrap_or_default();
    }

    let no_argument_algorithm = matches!(algorithm, "BF" | "DES" | "IDEA" | "RC4");
    let mut parts = if no_argument_algorithm {
        vec![algorithm.to_lowercase()]
    } else {
        algorithm
            .as_bytes()
            .chunks(3)
            .map(|part| String::from_utf8_lossy(part).to_lowercase())
            .collect::<Vec<_>>()
    };
    let no_arguments = argument_count(node) == 0;
    if let Some(arguments) = node.arguments() {
        for argument in arguments.arguments().iter() {
            let source = if let Some(string) = argument.as_string_node() {
                String::from_utf8_lossy(string.unescaped()).into_owned()
            } else {
                context.source_file().node(&argument).to_string()
            };
            parts.extend(
                source
                    .replace([':', '\''], "")
                    .split('-')
                    .map(str::to_lowercase),
            );
        }
    }
    if no_arguments && !no_argument_algorithm {
        parts.push("cbc".to_string());
    }
    parts.truncate(3);
    format!("'{}'", parts.join("-"))
}
