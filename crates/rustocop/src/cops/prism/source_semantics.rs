use std::path::{Component, Path, PathBuf};

use super::source_helpers::*;
use super::source_syntax::*;
use super::*;

mod parameters;
use parameters::*;

define_cops! {
    GemVersion => "Bundler/GemVersion" => source(gem_version),
    MultilineArrayLineBreaks => "Layout/MultilineArrayLineBreaks" => any_node(multiline_array_line_breaks),
    ErbNewArguments => "Lint/ErbNewArguments" => source(erb_new_arguments),
    HashNewWithKeywordArgumentsAsDefault => "Lint/HashNewWithKeywordArgumentsAsDefault" => source(hash_new_with_keyword_arguments_as_default),
    LambdaWithoutLiteralBlock => "Lint/LambdaWithoutLiteralBlock" => call(lambda_without_literal_block),
    RequireRelativeSelfPath => "Lint/RequireRelativeSelfPath" => source(require_relative_self_path),
    SharedMutableDefault => "Lint/SharedMutableDefault" => call(shared_mutable_default),
    OptionalArguments => "Style/OptionalArguments" => node(as_def_node, optional_arguments),
    OptionalBooleanParameter => "Style/OptionalBooleanParameter" => node(as_def_node, optional_boolean_parameter),
    Send => "Style/Send" => call(send),
}

fn send(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let has_block_argument = node
        .block()
        .is_some_and(|block| block.as_block_argument_node().is_some());
    if match_call(node).named(b"send").with_arguments().matches()
        || match_call(node).named(b"send").matches() && has_block_argument
    {
        context.report_selector(
            node,
            "Prefer `Object#__send__` or `Object#public_send` to `send`.",
        );
    }
}

fn gem_version(context: &mut CopContext<'_, '_>) {
    if context.related_config_value("AllCops", "DisabledByDefault") == Some("true")
        && !context.related_config_explicit("Bundler/GemVersion", "Enabled")
    {
        return;
    }
    let forbidden = context.policy().enforced_style("required") == "forbidden";
    let allowed = context.config_values("AllowedGems").to_vec();
    for (start, line) in source_lines(context.source()) {
        let leading = line.len() - line.trim_start().len();
        let call = line.trim();
        if !call.starts_with("gem ") {
            continue;
        }
        let argument_source = call["gem".len()..].trim_start();
        if !argument_source.starts_with(['\'', '"', '(']) {
            continue;
        }
        let Some(name) = first_quoted(call) else {
            continue;
        };
        if allowed.iter().any(|allowed| allowed == name) {
            continue;
        }
        let rest = call
            .find(name)
            .map_or("", |at| &call[at + name.len() + 1..]);
        let metadata = ["branch:", "ref:", "tag:"]
            .iter()
            .any(|keyword| rest.contains(keyword));
        let positional_version = rest.split(',').skip(1).any(|argument| {
            let argument = argument.trim();
            if !argument.starts_with(['\'', '"']) {
                return false;
            }
            let version = argument.trim_matches(['\'', '"']).trim_start();
            let version = version
                .trim_start_matches(['~', '<', '>', '='])
                .trim_start();
            version.as_bytes().first().is_some_and(u8::is_ascii_digit)
        });
        let specified = positional_version || metadata;
        if forbidden != specified {
            continue;
        }
        context.report(
            if forbidden {
                "Gem version specification is forbidden."
            } else {
                "Gem version specification is required."
            },
            start + leading..start + line.len(),
        );
    }
}

fn multiline_array_line_breaks(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    const MESSAGE: &str = "Each item in a multi-line array must start on a separate line.";
    let elements = if let Some(array) = node.as_array_node() {
        array.elements().iter().collect::<Vec<_>>()
    } else if let Some(rescue) = node.as_rescue_node() {
        rescue.exceptions().iter().collect::<Vec<_>>()
    } else {
        return;
    };
    let Some((first, last)) = elements.first().zip(elements.last()) else {
        return;
    };
    let file = context.source_file();
    let ignore_last = context.config_bool("AllowMultilineFinalElement", false);
    let all_on_same_line = if ignore_last {
        file.same_line(first.location().start_offset(), last.location().start_offset())
    } else {
        file.same_line(first.location().start_offset(), last.location().end_offset())
    };
    if all_on_same_line {
        return;
    }
    let mut last_seen_line = None;
    for element in elements {
        let location = element.location();
        let first_line = file.line_start(location.start_offset());
        let last_line = file.line_start(location.end_offset().saturating_sub(1));
        if last_seen_line.is_some_and(|seen| seen >= first_line) {
            let start = location.start_offset();
            context.insert(MESSAGE, location, start, "\n");
        } else {
            last_seen_line = Some(last_line);
        }
    }
}

fn erb_new_arguments(context: &mut CopContext<'_, '_>) {
    const SAFE: &str = "Passing safe_level with the 2nd argument of `ERB.new` is deprecated. Do not use it, and specify other arguments as keyword arguments.";
    if !context.target_ruby_version().at_least(2, 6) {
        return;
    }
    let source = context.source();
    for call in call_ranges(source, "ERB.new(") {
        let args = split_arguments(source, call.start + "ERB.new(".len(), call.end - 1);
        if args.len() < 2 {
            continue;
        }
        let has_trim = args
            .iter()
            .any(|arg| source[arg.clone()].trim_start().starts_with("trim_mode:"));
        let has_eout = args
            .iter()
            .any(|arg| source[arg.clone()].trim_start().starts_with("eoutvar:"));
        let optional = args
            .iter()
            .take_while(|arg| !source[(*arg).clone()].contains(':'))
            .count();
        if optional >= 2 {
            let argument = args[1].clone();
            context.remove(
                SAFE,
                trim_range(source, argument.clone()),
                args[0].end..argument.end,
            );
        }
        if optional >= 3 {
            let argument = args[2].clone();
            let value = source[argument.clone()].trim();
            let message = format!("Passing trim_mode with the 3rd argument of `ERB.new` is deprecated. Use keyword argument like `ERB.new(str, trim_mode: {value})` instead.");
            if has_trim {
                context.remove(
                    message,
                    trim_range(source, argument.clone()),
                    args[1].end..argument.end,
                );
            } else {
                context.replace(
                    message,
                    trim_range(source, argument.clone()),
                    trim_range(source, argument),
                    format!("trim_mode: {value}"),
                );
            }
        }
        if optional >= 4 {
            let argument = args[3].clone();
            let value = source[argument.clone()].trim();
            let message = format!("Passing eoutvar with the 4th argument of `ERB.new` is deprecated. Use keyword argument like `ERB.new(str, eoutvar: {value})` instead.");
            if has_eout {
                context.remove(
                    message,
                    trim_range(source, argument.clone()),
                    args[2].end..argument.end,
                );
            } else {
                context.replace(
                    message,
                    trim_range(source, argument.clone()),
                    trim_range(source, argument),
                    format!("eoutvar: {value}"),
                );
            }
        }
    }
}

fn hash_new_with_keyword_arguments_as_default(context: &mut CopContext<'_, '_>) {
    if context.related_config_value("AllCops", "DisabledByDefault") == Some("true")
        && !context.related_config_explicit(
            "Lint/HashNewWithKeywordArgumentsAsDefault",
            "Enabled",
        )
    {
        return;
    }
    const MESSAGE: &str = "Use a hash literal instead of keyword arguments.";
    let source = context.source();
    for call in call_ranges(source, "Hash.new(") {
        let args = call.start + "Hash.new(".len()..call.end - 1;
        let value = source[args.clone()].trim();
        if value.is_empty()
            || value.starts_with('{')
            || (!value.contains(':') && !value.contains("=>"))
            || (value.starts_with("capacity:") && !value.contains(','))
        {
            continue;
        }
        let leading = source[args.clone()].len() - source[args.clone()].trim_start().len();
        let range = args.start + leading..args.end;
        context.replace(
            MESSAGE,
            range.clone(),
            range.clone(),
            format!("{{{}}}", &source[range]),
        );
    }
}

fn lambda_without_literal_block(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    const MESSAGE: &str =
        "lambda without a literal block is deprecated; use the proc without lambda instead.";
    if node.receiver().is_some() || node.name().as_slice() != b"lambda" {
        return;
    }
    let Some(argument) = node.block().and_then(|block| block.as_block_argument_node()) else {
        return;
    };
    let Some(expression) = argument.expression() else {
        return;
    };
    if expression.as_symbol_node().is_some() {
        return;
    }
    let replacement = context.source_file().node(&expression).to_string();
    context.replace(MESSAGE, node.location(), node.location(), replacement);
}

fn require_relative_self_path(context: &mut CopContext<'_, '_>) {
    let own_path = Path::new(context.path());
    if own_path.extension().and_then(|value| value.to_str()) != Some("rb") {
        return;
    }
    let own_path = normalize_lexical_path(own_path);
    let source = context.source();
    for (start, line) in source_lines(source) {
        let call = line.trim();
        if !call.starts_with("require_relative ") {
            continue;
        }
        let Some(required) = first_quoted(call) else {
            continue;
        };
        let mut required_path = own_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(required);
        if required_path.extension().is_none() {
            required_path.set_extension("rb");
        }
        if normalize_lexical_path(&required_path) != own_path {
            continue;
        }
        let leading = line.len() - line.trim_start().len();
        context.remove(
            "Remove the `require_relative` that requires itself.",
            start + leading..start + line.len(),
            start..line_end(source, start),
        );
    }
}

fn normalize_lexical_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}
