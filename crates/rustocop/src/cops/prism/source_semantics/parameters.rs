use super::*;

pub(super) fn shared_mutable_default(context: &mut CopContext<'_, '_>) {
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

pub(super) fn top_level_return_with_argument(context: &mut CopContext<'_, '_>) {
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

pub(super) fn optional_arguments(context: &mut CopContext<'_, '_>) {
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

pub(super) fn optional_boolean_parameter(context: &mut CopContext<'_, '_>) {
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
            if matches!(value, "true" | "false") {
                context.report(
                    format!("Prefer keyword arguments for arguments with a boolean default value; use `{}: {value}` instead of `{text}`.", name.trim()),
                    range,
                );
            }
        }
    }
}
