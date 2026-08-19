use std::collections::HashSet;

use super::source_helpers::*;
use super::*;

mod literal_layout;
use literal_layout::*;

declare_source_cops! {
    RubyVersionGlobalsUsage => "Gemspec/RubyVersionGlobalsUsage" => ruby_version_globals,
    BlockComments => "Style/BlockComments" => block_comments,
    InsecureProtocolSource => "Bundler/InsecureProtocolSource" => insecure_protocol_source,
    DisjunctiveAssignmentInConstructor => "Lint/DisjunctiveAssignmentInConstructor" => disjunctive_assignment,
    RefinementImportMethods => "Lint/RefinementImportMethods" => refinement_import_methods,
    AttributeAssignment => "Gemspec/AttributeAssignment" => attribute_assignment,
    EachWithObjectArgument => "Lint/EachWithObjectArgument" => each_with_object_argument,
    UselessDefined => "Lint/UselessDefined" => useless_defined,
    AutoResourceCleanup => "Style/AutoResourceCleanup" => auto_resource_cleanup,
    InPatternThen => "Style/InPatternThen" => in_pattern_then,
    EmptyHeredoc => "Style/EmptyHeredoc" => empty_heredoc,
    SpaceInsideRangeLiteral => "Layout/SpaceInsideRangeLiteral" => space_inside_range,
    SpaceAfterMethodName => "Layout/SpaceAfterMethodName" => space_after_method_name,
    RedundantConstantBase => "Style/RedundantConstantBase" => redundant_constant_base,
}

fn ruby_version_globals(source: &str, reporter: &mut Reporter<'_>) {
    for name in [
        "::Ruby::VERSION",
        "Ruby::VERSION",
        "::RUBY_VERSION",
        "RUBY_VERSION",
    ] {
        for start in all_offsets(source, name) {
            let prefixed =
                start > 0 && matches!(source.as_bytes()[start - 1], b':' | b'_' | b'A'..=b'Z');
            if !prefixed {
                reporter.report(
                    format!("Do not use `{name}` in gemspec file."),
                    start..start + name.len(),
                );
            }
        }
    }
}

fn block_comments(source: &str, reporter: &mut Reporter<'_>) {
    let mut cursor = 0;
    while let Some(relative_start) = source[cursor..].find("=begin\n") {
        let start = cursor + relative_start;
        let Some(relative_end) = source[start + 7..].find("=end") else {
            break;
        };
        let marker_end = start + 7 + relative_end + 4;
        let end = if source.as_bytes().get(marker_end) == Some(&b'\n') {
            marker_end + 1
        } else {
            marker_end
        };
        let body = &source[start + 7..start + 7 + relative_end];
        let replacement = body
            .lines()
            .map(|line| {
                if line.is_empty() {
                    "#".to_string()
                } else {
                    format!("# {line}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let replacement = if replacement.is_empty() {
            String::new()
        } else {
            format!("{replacement}\n")
        };
        reporter.replace(
            "Do not use block comments.",
            start..end,
            start..end,
            replacement,
        );
        cursor = end;
    }
}

fn insecure_protocol_source(source: &str, reporter: &mut Reporter<'_>) {
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim_start();
        let Some(argument) = trimmed
            .strip_prefix("source ")
            .or_else(|| trimmed.strip_prefix("source("))
        else {
            continue;
        };
        let leading = line.len() - trimmed.len();
        for symbol in [":gemcutter", ":rubygems", ":rubyforge"] {
            if !argument.starts_with(symbol) {
                continue;
            }
            let start = offset + leading + trimmed.find(symbol).unwrap_or(0);
            let end = start + symbol.len();
            reporter.replace(
                format!("The source `{symbol}` is deprecated because HTTP requests are insecure. Please change your source to 'https://rubygems.org' if possible, or 'http://rubygems.org' if not."),
                start..end,
                start..end,
                "'https://rubygems.org'",
            );
        }
        if !reporter.config_bool("AllowHttpProtocol", true) {
            for literal in ["'http://rubygems.org'", "\"http://rubygems.org\""] {
                if !argument.starts_with(literal) {
                    continue;
                }
                let start = offset + leading + trimmed.find(literal).unwrap_or(0);
                let end = start + literal.len();
                reporter.replace(
                    "Use `https://rubygems.org` instead of `http://rubygems.org`.",
                    start..end,
                    start..end,
                    "'https://rubygems.org'",
                );
            }
        }
    }
}

fn disjunctive_assignment(source: &str, reporter: &mut Reporter<'_>) {
    let mut in_initialize = false;
    let mut unsafe_call_seen = false;
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim();
        if trimmed.starts_with("def initialize") {
            in_initialize = true;
            unsafe_call_seen = false;
            continue;
        }
        if in_initialize && trimmed == "end" {
            in_initialize = false;
        } else if in_initialize
            && (trimmed == "super"
                || (!trimmed.starts_with('@') && !trimmed.starts_with('#') && !trimmed.is_empty()))
        {
            unsafe_call_seen = true;
        }
        if in_initialize && !unsafe_call_seen && trimmed.starts_with('@') {
            if let Some(operator) = line.find("||=") {
                let start = offset + operator;
                reporter.replace(
                    "Unnecessary disjunctive assignment. Use plain assignment.",
                    start..start + 3,
                    start..start + 3,
                    "=",
                );
            }
        }
    }
}

fn refinement_import_methods(source: &str, reporter: &mut Reporter<'_>) {
    if !reporter.target_ruby_version().at_least(3, 1)
        || !source.contains("refine ")
        || !source.contains(" do\n")
    {
        return;
    }
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim_start();
        let method = if trimmed.starts_with("include ") {
            "include"
        } else if trimmed.starts_with("prepend ") {
            "prepend"
        } else {
            continue;
        };
        let start = offset + line.len() - trimmed.len();
        reporter.report(
            format!(
                "Use `import_methods` instead of `{method}` because it is deprecated in Ruby 3.1."
            ),
            start..start + method.len(),
        );
    }
}

fn attribute_assignment(source: &str, reporter: &mut Reporter<'_>) {
    let mut direct = HashSet::new();
    for (_, line) in source_lines(source) {
        if let Some(rest) = line.trim().strip_prefix("spec.") {
            if let Some((name, _)) = rest.split_once(" = ") {
                direct.insert(name.to_string());
            }
        }
    }
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix("spec.") else {
            continue;
        };
        let Some(bracket) = rest.find('[') else {
            continue;
        };
        if direct.contains(&rest[..bracket]) {
            let start = offset + line.len() - trimmed.len();
            reporter.report(
                "Use consistent style for Gemspec attributes assignment.",
                start..offset + line.len(),
            );
        }
    }
}

fn each_with_object_argument(source: &str, reporter: &mut Reporter<'_>) {
    for start in all_offsets(source, "each_with_object(") {
        let argument_start = start + "each_with_object(".len();
        let Some(close) = source[argument_start..].find(')') else {
            continue;
        };
        let argument = &source[argument_start..argument_start + close];
        if !argument.contains(',') && argument.parse::<f64>().is_ok() {
            let line_start = source[..start].rfind('\n').map_or(0, |offset| offset + 1);
            reporter.report(
                "The argument to each_with_object cannot be immutable.",
                line_start..argument_start + close + 1,
            );
        }
    }
}

fn useless_defined(source: &str, reporter: &mut Reporter<'_>) {
    for start in all_offsets(source, "defined?(") {
        let Some(close) = source[start..].find(')') else {
            continue;
        };
        let end = start + close + 1;
        let argument = source[start + 9..end - 1].trim_start();
        let kind = if argument.starts_with(['\'', '"']) {
            "string"
        } else if argument.starts_with(':') && !argument.contains(".to_proc") {
            "symbol"
        } else {
            continue;
        };
        reporter.report(
            format!("Calling `defined?` with a {kind} argument will always return a truthy value."),
            start..end,
        );
    }
}

fn auto_resource_cleanup(source: &str, reporter: &mut Reporter<'_>) {
    for receiver in ["::Tempfile", "Tempfile", "::File", "File"] {
        let needle = format!("{receiver}.open(");
        for start in all_offsets(source, &needle) {
            if !receiver.starts_with("::") && start >= 2 && &source[start - 2..start] == "::" {
                continue;
            }
            let end =
                line_end(source, start).saturating_sub(usize::from(source[start..].contains('\n')));
            let line = &source[start..end];
            if !line.contains('{') && !line.contains("&:") && !line.ends_with(".close") {
                reporter.report(
                    format!("Use the block version of `{receiver}.open`."),
                    start..end,
                );
            }
        }
    }
}

fn in_pattern_then(source: &str, reporter: &mut Reporter<'_>) {
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("in ") || trimmed.contains(" then ") {
            continue;
        }
        let Some(semi) = line.find(';') else { continue };
        let prefix = line[..=semi].trim_start();
        reporter.replace(
            format!(
                "Do not use `{prefix}`. Use `{} then` instead.",
                prefix.trim_end_matches(';')
            ),
            offset + semi..offset + semi + 1,
            offset + semi..offset + semi + 1,
            " then",
        );
    }
}
