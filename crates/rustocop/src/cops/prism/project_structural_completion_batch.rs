use super::*;

define_cops! {
    RequiredRubyVersion => "Gemspec/RequiredRubyVersion" => source(required_ruby_version),
    ClassStructure => "Layout/ClassStructure" => source(class_structure),
    ModuleLength => "Metrics/ModuleLength" => source(module_length),
    EmptyLineAfterMultilineCondition => "Layout/EmptyLineAfterMultilineCondition" => any_node(empty_after_multiline_condition),
    DeprecatedOpenSSLConstant => "Lint/DeprecatedOpenSSLConstant" => source(deprecated_openssl),
    HashConversion => "Style/HashConversion" => source(hash_conversion),
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

fn empty_after_multiline_condition(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(condition) = node.as_if_node() {
        let predicate = condition.predicate();
        let modifier = condition
            .if_keyword_loc()
            .is_some_and(|keyword| keyword.start_offset() != condition.location().start_offset());
        if !modifier || has_right_sibling(node, context.ancestors()) {
            check_multiline_condition(&predicate, &predicate, true, context);
        }
    } else if let Some(condition) = node.as_unless_node() {
        let predicate = condition.predicate();
        let modifier = condition.keyword_loc().start_offset() != condition.location().start_offset();
        if !modifier || has_right_sibling(node, context.ancestors()) {
            check_multiline_condition(&predicate, &predicate, true, context);
        }
    } else if let Some(condition) = node.as_while_node() {
        let predicate = condition.predicate();
        if !condition.is_begin_modifier() || has_right_sibling(node, context.ancestors()) {
            check_multiline_condition(&predicate, &predicate, true, context);
        }
    } else if let Some(condition) = node.as_until_node() {
        let predicate = condition.predicate();
        if !condition.is_begin_modifier() || has_right_sibling(node, context.ancestors()) {
            check_multiline_condition(&predicate, &predicate, true, context);
        }
    } else if let Some(branch) = node.as_when_node() {
        let conditions = branch.conditions().iter().collect::<Vec<_>>();
        if let (Some(first), Some(last)) = (conditions.first(), conditions.last()) {
            if !context
                .source_file()
                .same_line(first.location().start_offset(), last.location().end_offset())
            {
                check_multiline_condition(last, &branch.as_node(), false, context);
            }
        }
    } else if let Some(rescue) = node.as_rescue_node() {
        let exceptions = rescue.exceptions().iter().collect::<Vec<_>>();
        if exceptions.len() > 1 {
            let first = &exceptions[0];
            let last = exceptions.last().expect("multiple rescue exceptions");
            if !context
                .source_file()
                .same_line(first.location().start_offset(), last.location().end_offset())
            {
                check_multiline_condition(last, &rescue.as_node(), false, context);
            }
        }
    }
}

fn check_multiline_condition(
    condition_end: &Node<'_>,
    offense: &Node<'_>,
    require_multiline_node: bool,
    context: &mut CopContext<'_, '_>,
) {
    let location = condition_end.location();
    let file = context.source_file();
    if require_multiline_node && file.same_line(location.start_offset(), location.end_offset()) {
        return;
    }
    let condition_line = file.line_range(location.end_offset().saturating_sub(1));
    if condition_line.end >= context.source().len()
        || file.line(condition_line.end).trim().is_empty()
    {
        return;
    }
    context.insert(
        "Use empty line after multiline condition.",
        offense.location().start_offset()..offense.location().end_offset(),
        condition_line.end,
        "\n",
    );
}

fn has_right_sibling(node: &Node<'_>, ancestors: &[Node<'_>]) -> bool {
    ancestors.iter().rev().any(|ancestor| {
        ancestor.as_statements_node().is_some_and(|statements| {
            let body = statements.body().iter().collect::<Vec<_>>();
            body.iter().position(|sibling| {
                sibling.location().start_offset() == node.location().start_offset()
                    && sibling.location().end_offset() == node.location().end_offset()
            }).is_some_and(|index| index + 1 < body.len())
        })
    })
}

fn deprecated_openssl(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let needle = "OpenSSL::Cipher::";
    let mut search = 0;
    while let Some(relative) = source[search..].find(needle) {
        let start = search + relative;
        let Some(end_relative) = source[start..].find(')') else {
            break;
        };
        let end = start + end_relative + 1;
        let call = &source[start..end];
        let Some((cipher, args)) = call[needle.len()..].split_once(".new(") else {
            search = end;
            continue;
        };
        let pieces = args
            .trim_end_matches(')')
            .split(',')
            .map(|piece| piece.trim().trim_matches(['\'', '"', ':']))
            .collect::<Vec<_>>();
        let name = if pieces.len() == 2 {
            format!(
                "{}-{}-{}",
                cipher.to_lowercase(),
                pieces[0].to_lowercase(),
                pieces[1].to_lowercase()
            )
        } else {
            format!(
                "{}-{}",
                cipher.to_lowercase(),
                pieces.first().unwrap_or(&"").to_lowercase()
            )
        };
        let replacement = format!("OpenSSL::Cipher.new('{name}')");
        context.replace(
            format!("Use `{replacement}` instead of `{call}`."),
            start..end,
            start..end,
            replacement,
        );
        search = end;
    }
}

fn hash_conversion(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let mut search = 0;
    while let Some(relative) = source[search..].find("Hash[") {
        let start = search + relative;
        let Some(close) = source[start + 5..].find(']').map(|at| start + 5 + at) else {
            break;
        };
        let arguments = &source[start + 5..close];
        let end = close + 1;
        if !arguments.contains(',') {
            let replacement = format!("{}.to_h", arguments.trim());
            context.replace(
                format!("Prefer `{replacement}` to `{}`.", &source[start..end]),
                start..end,
                start..end,
                replacement,
            );
        } else {
            let values = arguments.split(',').map(str::trim).collect::<Vec<_>>();
            let replacement = values
                .chunks(2)
                .filter(|pair| pair.len() == 2)
                .map(|pair| format!("{} => {}", pair[0], pair[1]))
                .collect::<Vec<_>>()
                .join(", ");
            context.replace(
                "Prefer literal hash to `Hash[arg1, arg2, ...]`.",
                start..end,
                start..end,
                format!("{{{replacement}}}"),
            );
        }
        search = end;
    }
}
