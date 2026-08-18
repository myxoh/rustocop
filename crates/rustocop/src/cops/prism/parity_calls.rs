use std::ops::Range;
use std::path::Path;

use super::source_helpers::*;
use super::*;

define_cops! {
    GemVersion => "Bundler/GemVersion" => source(gem_version),
    MultilineArrayLineBreaks => "Layout/MultilineArrayLineBreaks" => source(multiline_array_line_breaks),
    ErbNewArguments => "Lint/ErbNewArguments" => source(erb_new_arguments),
    HashNewWithKeywordArgumentsAsDefault => "Lint/HashNewWithKeywordArgumentsAsDefault" => source(hash_new_with_keyword_arguments_as_default),
    LambdaWithoutLiteralBlock => "Lint/LambdaWithoutLiteralBlock" => source(lambda_without_literal_block),
    RequireRelativeSelfPath => "Lint/RequireRelativeSelfPath" => source(require_relative_self_path),
    SharedMutableDefault => "Lint/SharedMutableDefault" => source(shared_mutable_default),
    TopLevelReturnWithArgument => "Lint/TopLevelReturnWithArgument" => source(top_level_return_with_argument),
    OptionalArguments => "Style/OptionalArguments" => source(optional_arguments),
    OptionalBooleanParameter => "Style/OptionalBooleanParameter" => source(optional_boolean_parameter),
    Send => "Style/Send" => call(send),
}

fn send(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if match_call(node).named(b"send").with_arguments().matches() {
        context.report_selector(
            node,
            "Prefer `Object#__send__` or `Object#public_send` to `send`.",
        );
    }
}

fn gem_version(context: &mut CopContext<'_, '_>) {
    let forbidden = context.policy().enforced_style("required") == "forbidden";
    let allowed = context.config_values("AllowedGems").to_vec();
    for (start, line) in source_lines(context.source()) {
        let leading = line.len() - line.trim_start().len();
        let call = line.trim();
        if !call.starts_with("gem ") {
            continue;
        }
        let Some(name) = first_quoted(call) else {
            continue;
        };
        if allowed.iter().any(|allowed| allowed == name) {
            continue;
        }
        let rest = &call[name.len() + call.find(name).unwrap_or(0)..];
        let metadata = rest.contains("branch:") || rest.contains("ref:") || rest.contains("tag:");
        let positional_version = rest.matches(['\'', '"']).count() >= 2;
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
    let own_name = own_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    let source = context.source();
    for (start, line) in source_lines(source) {
        let call = line.trim();
        if !call.starts_with("require_relative ") {
            continue;
        }
        let Some(required) = first_quoted(call) else {
            continue;
        };
        let required_path = Path::new(required);
        let required_name = required_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let required_extension = required_path.extension().and_then(|value| value.to_str());
        if required_name != own_name
            || required_extension.is_some_and(|extension| extension != "rb")
        {
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

fn shared_mutable_default(context: &mut CopContext<'_, '_>) {
    const MESSAGE: &str = "Do not create a Hash with a mutable default value as the default value can accidentally be changed.";
    let source = context.source();
    for call in call_ranges(source, "Hash.new(") {
        let arguments = source[call.start + 9..call.end - 1].trim();
        let default = arguments.split(',').next().unwrap_or_default().trim();
        if matches!(default, "[]" | "{}" | "Array.new" | "Hash.new")
            && !arguments.contains(".freeze")
        {
            context.report(MESSAGE, call);
        }
    }
    for (start, line) in source_lines(source) {
        let trimmed = line.trim();
        if trimmed.starts_with("Hash.new Array.new") {
            let leading = line.len() - line.trim_start().len();
            context.report(MESSAGE, start + leading..start + line.len());
        }
    }
}

fn top_level_return_with_argument(context: &mut CopContext<'_, '_>) {
    const MESSAGE: &str = "Top level return with argument detected.";
    let source = context.source();
    let mut method_depth = 0_usize;
    for (line_start, line) in source_lines(source) {
        let trimmed = line.trim_start();
        if trimmed.starts_with("def ") {
            method_depth += 1;
            continue;
        }
        if method_depth > 0 {
            if trimmed == "end" {
                method_depth -= 1;
            }
            continue;
        }
        for (relative, _) in line.match_indices("return ") {
            if relative > 0 && identifier_byte(line.as_bytes()[relative - 1]) {
                continue;
            }
            if line[..relative].contains('{') && !line[..relative].contains('}') {
                continue;
            }
            let tail = &line[relative + 7..];
            if tail.starts_with("if ") || tail.starts_with("unless ") {
                continue;
            }
            let argument_len = tail
                .find(" if ")
                .or_else(|| tail.find(" unless "))
                .or_else(|| tail.find(';'))
                .unwrap_or(tail.len());
            if argument_len == 0 {
                continue;
            }
            let offense = line_start + relative..line_start + relative + 7 + argument_len;
            context.replace(MESSAGE, offense.clone(), offense, "return");
        }
    }
}

fn optional_arguments(context: &mut CopContext<'_, '_>) {
    for definition in definitions(context.source()) {
        let args = split_arguments(
            context.source(),
            definition.arguments.start,
            definition.arguments.end,
        );
        let positional = args
            .iter()
            .take_while(|arg| !context.source()[(*arg).clone()].contains(':'))
            .cloned()
            .collect::<Vec<_>>();
        let last_required = positional
            .iter()
            .rposition(|arg| !context.source()[arg.clone()].contains('='));
        let Some(last_required) = last_required else {
            continue;
        };
        for argument in positional.iter().take(last_required) {
            if context.source()[argument.clone()].contains('=') {
                let range = trim_range(context.source(), argument.clone());
                context.report(
                    "Optional arguments should appear at the end of the argument list.",
                    range,
                );
            }
        }
    }
}

fn optional_boolean_parameter(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for definition in definitions(source) {
        if context.policy().allows_method(definition.name.as_bytes()) {
            continue;
        }
        for argument in
            split_arguments(source, definition.arguments.start, definition.arguments.end)
        {
            let range = trim_range(source, argument);
            let text = &source[range.clone()];
            let Some((name, value)) = text.split_once('=') else {
                continue;
            };
            let value = value.trim();
            if !matches!(value, "true" | "false") {
                continue;
            }
            context.report(
                format!("Prefer keyword arguments for arguments with a boolean default value; use `{}: {value}` instead of `{text}`.", name.trim()),
                range,
            );
        }
    }
}

struct Definition<'source> {
    name: &'source str,
    arguments: Range<usize>,
}

fn definitions(source: &str) -> Vec<Definition<'_>> {
    let mut definitions = Vec::new();
    for (start, line) in source_lines(source) {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("def ") {
            continue;
        }
        let Some(open) = line.find('(') else { continue };
        let Some(close) = line.rfind(')') else {
            continue;
        };
        let name_start = trimmed.find(' ').unwrap_or(0) + 1;
        let name = trimmed[name_start..]
            .split('(')
            .next()
            .unwrap_or_default()
            .rsplit('.')
            .next()
            .unwrap_or_default();
        definitions.push(Definition {
            name,
            arguments: start + open + 1..start + close,
        });
    }
    definitions
}

fn call_ranges(source: &str, needle: &str) -> Vec<Range<usize>> {
    let mut calls = Vec::new();
    for start in all_offsets(source, needle) {
        let open = start + needle.len() - 1;
        if let Some(close) = matching_delimiter(source, open, b'(', b')') {
            calls.push(start..close + 1);
        }
    }
    calls
}

fn matching_delimiter(source: &str, open: usize, left: u8, right: u8) -> Option<usize> {
    let mut depth = 0_usize;
    let mut quote = None;
    for (index, byte) in source.as_bytes().iter().copied().enumerate().skip(open) {
        if let Some(delimiter) = quote {
            if byte == delimiter && source.as_bytes().get(index.wrapping_sub(1)) != Some(&b'\\') {
                quote = None;
            }
            continue;
        }
        if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
            continue;
        }
        if byte == left {
            depth += 1;
        }
        if byte == right {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn split_arguments(source: &str, start: usize, end: usize) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut element = start;
    let mut depths = (0_i32, 0_i32, 0_i32);
    let mut quote = None;
    for (relative, character) in source[start..end].char_indices() {
        let offset = start + relative;
        if let Some(delimiter) = quote {
            if character == delimiter
                && source.as_bytes().get(offset.wrapping_sub(1)) != Some(&b'\\')
            {
                quote = None;
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            quote = Some(character);
            continue;
        }
        match character {
            '(' => depths.0 += 1,
            ')' => depths.0 -= 1,
            '[' => depths.1 += 1,
            ']' => depths.1 -= 1,
            '{' => depths.2 += 1,
            '}' => depths.2 -= 1,
            ',' if depths == (0, 0, 0) => {
                ranges.push(element..offset);
                element = offset + 1;
            }
            _ => {}
        }
    }
    if !source[element..end].trim().is_empty() {
        ranges.push(element..end);
    }
    ranges
}

fn top_level_elements(source: &str, start: usize, end: usize) -> Vec<Range<usize>> {
    split_arguments(source, start, end)
        .into_iter()
        .map(|range| trim_range(source, range))
        .collect()
}

fn trim_range(source: &str, range: Range<usize>) -> Range<usize> {
    let value = &source[range.clone()];
    let leading = value.len() - value.trim_start().len();
    let trailing = value.len() - value.trim_end().len();
    range.start + leading..range.end - trailing
}

fn first_quoted(source: &str) -> Option<&str> {
    let (start, quote) = source
        .char_indices()
        .find(|(_, character)| matches!(character, '\'' | '"'))?;
    let rest = &source[start + 1..];
    let end = rest.find(quote)?;
    Some(&rest[..end])
}
