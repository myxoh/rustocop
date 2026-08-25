use std::collections::HashSet;

use super::source_helpers::*;
use super::source_syntax::matching_delimiter;
use super::*;

mod literal_layout;
use literal_layout::*;

mod source_registry {
    use super::*;

    declare_source_cops! {
        RubyVersionGlobalsUsage => "Gemspec/RubyVersionGlobalsUsage" => super::ruby_version_globals,
        RefinementImportMethods => "Lint/RefinementImportMethods" => super::refinement_import_methods,
        AttributeAssignment => "Gemspec/AttributeAssignment" => super::attribute_assignment,
        EachWithObjectArgument => "Lint/EachWithObjectArgument" => super::each_with_object_argument,
        UselessDefined => "Lint/UselessDefined" => super::useless_defined,
        AutoResourceCleanup => "Style/AutoResourceCleanup" => super::auto_resource_cleanup,
        InPatternThen => "Style/InPatternThen" => super::in_pattern_then,
        EmptyHeredoc => "Style/EmptyHeredoc" => super::empty_heredoc,
        SpaceAfterMethodName => "Layout/SpaceAfterMethodName" => super::space_after_method_name,
    }
}

define_call_cop!(InsecureProtocolSource => "Bundler/InsecureProtocolSource" => insecure_protocol_source);
define_node_cop!(DisjunctiveAssignmentInConstructor => "Lint/DisjunctiveAssignmentInConstructor" => as_def_node => disjunctive_assignment);

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    let mut cops = source_registry::cops();
    cops.push(Box::new(InsecureProtocolSource));
    cops.push(Box::new(DisjunctiveAssignmentInConstructor));
    cops
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

fn insecure_protocol_source(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.receiver().is_some() || call_name(node) != b"source" || argument_count(node) != 1 {
        return;
    }
    let Some(argument) = only_argument(node) else {
        return;
    };
    let (message, insecure) = if let Some(symbol) = argument.as_symbol_node() {
        let source = symbol.unescaped();
        if !matches!(source, b"gemcutter" | b"rubygems" | b"rubyforge") {
            return;
        }
        let source = String::from_utf8_lossy(source);
        (
            format!("The source `:{source}` is deprecated because HTTP requests are insecure. Please change your source to 'https://rubygems.org' if possible, or 'http://rubygems.org' if not."),
            true,
        )
    } else if argument.as_string_node().is_some_and(|string| {
        string.unescaped() == b"http://rubygems.org"
    }) {
        (
            "Use `https://rubygems.org` instead of `http://rubygems.org`.".to_string(),
            !context.config_bool("AllowHttpProtocol", true),
        )
    } else {
        return;
    };
    if insecure {
        context.replace(
            message,
            argument.location(),
            argument.location(),
            "'https://rubygems.org'",
        );
    }
}

fn disjunctive_assignment(
    node: &ruby_prism::DefNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    if node.name().as_slice() != b"initialize" {
        return;
    }
    let Some(body) = node.body() else {
        return;
    };
    if let Some(statements) = body.as_statements_node() {
        for expression in statements.body().iter() {
            if !check_constructor_assignment(&expression, context) {
                break;
            }
        }
    } else {
        check_constructor_assignment(&body, context);
    }
}

fn check_constructor_assignment(node: &Node<'_>, context: &mut CopContext<'_, '_>) -> bool {
    if let Some(write) = node.as_instance_variable_or_write_node() {
        let operator = write.operator_loc();
        context.replace(
            "Unnecessary disjunctive assignment. Use plain assignment.",
            &operator,
            &operator,
            "=",
        );
        true
    } else {
        node.as_local_variable_or_write_node().is_some()
            || node.as_class_variable_or_write_node().is_some()
            || node.as_global_variable_or_write_node().is_some()
            || node.as_constant_or_write_node().is_some()
            || node.as_constant_path_or_write_node().is_some()
            || node.as_call_or_write_node().is_some()
            || node.as_index_or_write_node().is_some()
    }
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
    let file = SourceFile::new(source);
    let comment_ranges = file.comment_ranges();
    for receiver in ["::Tempfile", "Tempfile", "::File", "File"] {
        for separator in ["(", " "] {
            let needle = format!("{receiver}.open{separator}");
            for start in all_offsets(source, &needle) {
            if comment_ranges
                .iter()
                .any(|range| range.start <= start && start < range.end)
            {
                continue;
            }
            if !receiver.starts_with("::") && start >= 2 && &source[start - 2..start] == "::" {
                continue;
            }
            let line_start = source[..start].rfind('\n').map_or(0, |newline| newline + 1);
            let prefix = source[line_start..start].trim_end();
            if prefix.ends_with(':') || prefix.ends_with("=>") {
                continue;
            }
            let line_end_offset =
                line_end(source, start).saturating_sub(usize::from(source[start..].contains('\n')));
            let end = if separator == "(" {
                let opening = start + needle.len() - 1;
                matching_delimiter(source, opening, b'(', b')')
                    .map_or(line_end_offset, |closing| closing + 1)
            } else {
                line_end_offset
            };
            let line = &source[start..end];
            let full_line = &source[start..line_end_offset];
            let block_brace = full_line
                .rfind(')')
                .is_some_and(|closing| full_line[closing + 1..].contains('{'));
            if !block_brace
                && !line.contains("&:")
                && !line.contains(", &")
                && !full_line.contains(" do")
                && !full_line.ends_with(".close")
            {
                reporter.report(
                    format!("Use the block version of `{receiver}.open`."),
                    start..end,
                );
            }
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
