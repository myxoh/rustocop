use std::collections::HashSet;

use super::source_helpers::*;
use super::*;

mod literal_layout;
use literal_layout::*;

declare_source_cops! {
    RubyVersionGlobalsUsage => "Gemspec/RubyVersionGlobalsUsage" => ruby_version_globals,
    InsecureProtocolSource => "Bundler/InsecureProtocolSource" => insecure_protocol_source,
    DisjunctiveAssignmentInConstructor => "Lint/DisjunctiveAssignmentInConstructor" => disjunctive_assignment,
    RefinementImportMethods => "Lint/RefinementImportMethods" => refinement_import_methods,
    AttributeAssignment => "Gemspec/AttributeAssignment" => attribute_assignment,
    EachWithObjectArgument => "Lint/EachWithObjectArgument" => each_with_object_argument,
    UselessDefined => "Lint/UselessDefined" => useless_defined,
    AutoResourceCleanup => "Style/AutoResourceCleanup" => auto_resource_cleanup,
    InPatternThen => "Style/InPatternThen" => in_pattern_then,
    EmptyHeredoc => "Style/EmptyHeredoc" => empty_heredoc,
    SpaceAfterMethodName => "Layout/SpaceAfterMethodName" => space_after_method_name,
}

fn ruby_version_globals(source: &str, reporter: &mut Reporter<'_>) {
    if !reporter.path().ends_with("(string)") && !reporter.path().ends_with(".gemspec") {
        return;
    }
    for name in [
        "::Ruby::VERSION",
        "Ruby::VERSION",
        "::RUBY_VERSION",
        "RUBY_VERSION",
    ] {
        for start in code_offsets(source, name) {
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

fn code_offsets(source: &str, needle: &str) -> Vec<usize> {
    let bytes = source.as_bytes();
    let needle = needle.as_bytes();
    let mut offsets = Vec::new();
    let mut quote = None;
    let mut escaped = false;
    let mut comment = false;
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if comment {
            comment = byte != b'\n';
        } else if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == delimiter {
                quote = None;
            }
        } else if byte == b'#' {
            comment = true;
        } else if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
        } else if bytes[index..].starts_with(needle) {
            offsets.push(index);
            index += needle.len().saturating_sub(1);
        }
        index += 1;
    }
    offsets
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
    let mut rescued_constructor = false;
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim();
        if trimmed.strip_prefix("def initialize").is_some_and(|rest| {
            rest.is_empty() || rest.starts_with('(') || rest.starts_with(char::is_whitespace)
        }) {
            in_initialize = true;
            unsafe_call_seen = false;
            rescued_constructor = constructor_has_rescue(source, offset, line);
            continue;
        }
        if in_initialize && trimmed == "end" {
            in_initialize = false;
        }
        if in_initialize
            && !rescued_constructor
            && !unsafe_call_seen
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
        {
            let operator = trimmed.find("||=");
            let instance_variable = operator.is_some_and(|operator| {
                let lhs = trimmed[..operator].trim();
                lhs.strip_prefix('@').is_some_and(|name| {
                    !name.is_empty()
                        && name
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                })
            });
            if instance_variable {
                let operator = line.find("||=").expect("trimmed line contains operator");
                let start = offset + operator;
                reporter.replace(
                    "Unnecessary disjunctive assignment. Use plain assignment.",
                    start..start + 3,
                    start..start + 3,
                    "=",
                );
            } else {
                unsafe_call_seen = true;
            }
        }
    }
}

fn constructor_has_rescue(source: &str, definition_offset: usize, definition_line: &str) -> bool {
    let indentation = definition_line.len() - definition_line.trim_start().len();
    for (offset, line) in source_lines(source) {
        if offset <= definition_offset {
            continue;
        }
        let trimmed = line.trim();
        let line_indentation = line.len() - line.trim_start().len();
        if trimmed == "end" && line_indentation <= indentation {
            return false;
        }
        if trimmed.starts_with("rescue") && line_indentation == indentation {
            return true;
        }
    }
    false
}

fn refinement_import_methods(source: &str, reporter: &mut Reporter<'_>) {
    if !reporter.target_ruby_version().at_least(3, 1) {
        return;
    }
    let mut refinement_indent = None;
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim_start();
        let indentation = line.len() - trimmed.len();
        if refinement_indent.is_none() && trimmed.starts_with("refine ") && trimmed.ends_with(" do") {
            refinement_indent = Some(indentation);
            continue;
        }
        let Some(refine_indent) = refinement_indent else {
            continue;
        };
        if trimmed == "end" && indentation == refine_indent {
            refinement_indent = None;
            continue;
        }
        if indentation <= refine_indent {
            continue;
        }
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
    let Some(specification) = source_lines(source).find_map(|(_, line)| {
        line.contains("Gem::Specification.new")
            .then(|| line.split('|').nth(1).map(str::trim))
            .flatten()
    }) else {
        return;
    };
    let mut direct = HashSet::new();
    for (_, line) in source_lines(source) {
        if let Some(rest) = line
            .trim()
            .strip_prefix(&format!("{specification}."))
        {
            if let Some((name, _)) = rest.split_once(" = ") {
                direct.insert(name.to_string());
            }
        }
    }
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix(&format!("{specification}.")) else {
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
        if start > 0
            && (source.as_bytes()[start - 1].is_ascii_alphanumeric()
                || matches!(source.as_bytes()[start - 1], b'_' | b'.'))
        {
            continue;
        }
        let Some(close) = source[start..].find(')') else {
            continue;
        };
        let end = start + close + 1;
        let argument = source[start + 9..end - 1].trim_start();
        let kind = if argument.starts_with(['\'', '"']) {
            "string"
        } else if argument.starts_with(':')
            && !argument.starts_with("::")
            && !argument.contains(".to_proc")
        {
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
