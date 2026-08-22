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

pub(super) fn optional_arguments(
    node: &ruby_prism::DefNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let Some(parameters) = node.parameters() else {
        return;
    };
    if parameters.posts().is_empty() {
        return;
    }
    for optional in parameters.optionals().iter() {
        context.report(
            "Optional arguments should appear at the end of the argument list.",
            optional.location(),
        );
    }
}

pub(super) fn optional_boolean_parameter(
    node: &ruby_prism::DefNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    if context.policy().allows_method(node.name().as_slice()) {
        return;
    }
    let Some(parameters) = node.parameters() else {
        return;
    };
    for optional in parameters.optionals().iter() {
        let Some(optional) = optional.as_optional_parameter_node() else {
            continue;
        };
        let value = optional.value();
        let value = if value.as_true_node().is_some() {
            "true"
        } else if value.as_false_node().is_some() {
            "false"
        } else {
            continue;
        };
        let text = context.source_file().at(&optional.location());
        let name = String::from_utf8_lossy(optional.name().as_slice());
        context.report(
            format!("Prefer keyword arguments for arguments with a boolean default value; use `{name}: {value}` instead of `{text}`."),
            optional.location(),
        );
    }
}
