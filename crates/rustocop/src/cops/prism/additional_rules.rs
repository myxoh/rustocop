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
        AttributeAssignment => "Gemspec/AttributeAssignment" => super::attribute_assignment,
        AutoResourceCleanup => "Style/AutoResourceCleanup" => super::auto_resource_cleanup,
        EmptyHeredoc => "Style/EmptyHeredoc" => super::empty_heredoc,
        SpaceAfterMethodName => "Layout/SpaceAfterMethodName" => super::space_after_method_name,
    }
}

define_call_cop!(InsecureProtocolSource => "Bundler/InsecureProtocolSource" => insecure_protocol_source);
define_call_cop!(EachWithObjectArgument => "Lint/EachWithObjectArgument" => each_with_object_argument);
define_call_cop!(RefinementImportMethods => "Lint/RefinementImportMethods" => refinement_import_methods);
define_node_cop!(DisjunctiveAssignmentInConstructor => "Lint/DisjunctiveAssignmentInConstructor" => as_def_node => disjunctive_assignment);
define_node_cop!(UselessDefined => "Lint/UselessDefined" => as_defined_node => useless_defined);

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    let mut cops = source_registry::cops();
    cops.push(Box::new(InsecureProtocolSource));
    cops.push(Box::new(EachWithObjectArgument));
    cops.push(Box::new(RefinementImportMethods));
    cops.push(Box::new(DisjunctiveAssignmentInConstructor));
    cops.push(Box::new(UselessDefined));
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

fn refinement_import_methods(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(3, 1)
        || node.receiver().is_some()
        || !matches!(node.name().as_slice(), b"include" | b"prepend")
    {
        return;
    }
    let mut ancestors = context.ancestors().iter().rev();
    let Some(statements) = ancestors.next().and_then(Node::as_statements_node) else {
        return;
    };
    if statements.body().len() != 1 {
        return;
    }
    let Some(block) = ancestors.find_map(Node::as_block_node) else {
        return;
    };
    let Some(refine_call) = context.ancestors().iter().rev().find_map(|ancestor| {
        let call = ancestor.as_call_node()?;
        call.block()
            .and_then(|candidate| candidate.as_block_node())
            .filter(|candidate| candidate.location().start_offset() == block.location().start_offset())?;
        Some(call)
    }) else {
        return;
    };
    if refine_call.name().as_slice() != b"refine" {
        return;
    }
    let method = String::from_utf8_lossy(node.name().as_slice());
    context.report(
        format!(
            "Use `import_methods` instead of `{method}` because it is deprecated in Ruby 3.1."
        ),
        node.message_loc().expect("include/prepend selector"),
    );
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

fn each_with_object_argument(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) != b"each_with_object" || argument_count(node) != 1 {
        return;
    }
    if only_argument(node).is_some_and(|argument| immutable_literal(&argument)) {
        let end = node
            .closing_loc()
            .map_or_else(|| node.arguments().map_or(node.location().end_offset(), |arguments| arguments.location().end_offset()), |closing| closing.end_offset());
        context.report(
            "The argument to each_with_object cannot be immutable.",
            node.location().start_offset()..end,
        );
    }
}

fn useless_defined(node: &ruby_prism::DefinedNode<'_>, context: &mut CopContext<'_, '_>) {
    let argument = node.value();
    let kind = if argument.as_string_node().is_some()
        || argument.as_interpolated_string_node().is_some()
    {
        "string"
    } else if argument.as_symbol_node().is_some()
        || argument.as_interpolated_symbol_node().is_some()
    {
        "symbol"
    } else {
        return;
    };
    context.report(
        format!("Calling `defined?` with a {kind} argument will always return a truthy value."),
        node.location(),
    );
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
