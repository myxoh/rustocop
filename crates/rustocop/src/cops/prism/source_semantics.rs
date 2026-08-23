use std::path::{Component, Path, PathBuf};

use super::source_helpers::*;
use super::source_syntax::*;
use super::*;

mod parameters;
use parameters::*;

define_cops! {
    GemVersion => "Bundler/GemVersion" => source(gem_version),
    MultilineArrayLineBreaks => "Layout/MultilineArrayLineBreaks" => source(multiline_array_line_breaks),
    ErbNewArguments => "Lint/ErbNewArguments" => source(erb_new_arguments),
    HashNewWithKeywordArgumentsAsDefault => "Lint/HashNewWithKeywordArgumentsAsDefault" => source(hash_new_with_keyword_arguments_as_default),
    LambdaWithoutLiteralBlock => "Lint/LambdaWithoutLiteralBlock" => source(lambda_without_literal_block),
    RequireRelativeSelfPath => "Lint/RequireRelativeSelfPath" => source(require_relative_self_path),
    SharedMutableDefault => "Lint/SharedMutableDefault" => source(shared_mutable_default),
    TopLevelReturnWithArgument => "Lint/TopLevelReturnWithArgument" => source(top_level_return_with_argument),
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

fn multiline_array_line_breaks(context: &mut CopContext<'_, '_>) {
    const MESSAGE: &str = "Each item in a multi-line array must start on a separate line.";
    let source = context.source();
    let bytes = source.as_bytes();
    let mut stack = Vec::new();
    let mut arrays = Vec::new();
    let mut quote = None;
    for (index, byte) in bytes.iter().copied().enumerate() {
        if let Some(delimiter) = quote {
            if byte == delimiter && bytes.get(index.wrapping_sub(1)) != Some(&b'\\') {
                quote = None;
            }
            continue;
        }
        if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
        } else if byte == b'[' {
            stack.push(index);
        } else if byte == b']' {
            if let Some(open) = stack.pop() {
                arrays.push(open..index + 1);
            }
        }
    }
    for array in arrays {
        if !source[array.clone()].contains('\n') {
            continue;
        }
        let elements = top_level_elements(source, array.start + 1, array.end - 1);
        if context.config_bool("AllowMultilineFinalElement", false)
            && elements
                .last()
                .is_some_and(|element| source[element.clone()].contains('\n'))
        {
            continue;
        }
        if elements
            .first()
            .is_some_and(|element| source[array.start + 1..element.start].contains('\n'))
        {
            continue;
        }
        for (index, pair) in elements.windows(2).enumerate() {
            let previous = &pair[0];
            let element = &pair[1];
            if context.config_bool("AllowMultilineFinalElement", false)
                && index + 2 == elements.len()
            {
                continue;
            }
            if source[previous.end..element.start].contains('\n') {
                continue;
            }
            let comma = source[previous.end..element.start]
                .find(',')
                .map_or(previous.end, |at| previous.end + at);
            let expands_final_element = context.config_bool("AllowMultilineFinalElement", false)
                && index + 3 == elements.len()
                && source[element.clone()].contains('\n');
            let edit = if expands_final_element {
                comma + 1..elements[index + 2].start
            } else {
                comma + 1..element.start
            };
            let replacement = if expands_final_element {
                format!(" \n{}, \n", &source[element.clone()])
            } else {
                " \n".to_string()
            };
            context.replace(MESSAGE, element.clone(), edit, replacement);
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

fn lambda_without_literal_block(context: &mut CopContext<'_, '_>) {
    const MESSAGE: &str =
        "lambda without a literal block is deprecated; use the proc without lambda instead.";
    let source = context.source();
    for call in call_ranges(source, "lambda(") {
        let argument = source[call.start + 7..call.end - 1].trim();
        if !argument.starts_with('&') || argument.starts_with("&:") {
            continue;
        }
        context.replace(MESSAGE, call.clone(), call, argument[1..].to_string());
    }
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
