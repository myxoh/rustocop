use super::*;

define_cops! {
    Syntax => "Lint/Syntax" => source(syntax),
    FormatParameterMismatch => "Lint/FormatParameterMismatch" => source(format_parameter_mismatch),
    UnusedBlockArgument => "Lint/UnusedBlockArgument" => source(unused_block_argument),
    AmbiguousRange => "Lint/AmbiguousRange" => source(ambiguous_range),
    NonAtomicFileOperation => "Lint/NonAtomicFileOperation" => source(non_atomic_file_operation),
    UnmodifiedReduceAccumulator => "Lint/UnmodifiedReduceAccumulator" => source(unmodified_reduce_accumulator),
    DocumentationMethod => "Style/DocumentationMethod" => source(documentation_method),
    RedundantSplatExpansion => "Lint/RedundantSplatExpansion" => source(redundant_splat_expansion),
    MethodCallWithArgsParentheses => "Style/MethodCallWithArgsParentheses" => source(method_call_parentheses),
    ModuleMemberExistenceCheck => "Style/ModuleMemberExistenceCheck" => source(module_member_existence_check),
}

fn syntax(context: &mut CopContext<'_, '_>) {
    let result = parse(context.source().as_bytes());
    for error in result.errors() {
        let location = error.location();
        let start = location.start_offset().min(context.source().len());
        let end = location.end_offset().max(start).min(context.source().len());
        context.report(error.message(), start..end);
    }
}

fn format_parameter_mismatch(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let unescaped = line.replace("%%", "");
        let placeholders = unescaped.matches("%s").count()
            + unescaped.matches("%d").count()
            + unescaped.matches("%f").count();
        if placeholders == 0 {
            continue;
        }
        let Some(percent) = line.find(" % ") else {
            continue;
        };
        let arguments = line[percent + 3..].trim();
        let supplied = if arguments.starts_with('[') {
            arguments.split(',').count()
        } else {
            1
        };
        if placeholders != supplied {
            context.report(format!("Number of arguments ({supplied}) to format string differs from number of fields ({placeholders})."), offset + percent..offset + line.len());
        }
    }
}

fn unused_block_argument(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let Some(first_pipe) = line.find('|') else {
            continue;
        };
        let Some(second_pipe) = line[first_pipe + 1..]
            .find('|')
            .map(|at| first_pipe + 1 + at)
        else {
            continue;
        };
        let body = &line[second_pipe + 1..];
        if body.contains('=') {
            continue;
        }
        for argument in line[first_pipe + 1..second_pipe].split(',').map(str::trim) {
            if argument.is_empty()
                || argument.starts_with('_')
                || body
                    .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .any(|word| word == argument)
            {
                continue;
            }
            let start = offset
                + first_pipe
                + 1
                + line[first_pipe + 1..second_pipe]
                    .find(argument)
                    .unwrap_or(0);
            context.replace(
                "Unused block argument.",
                start..start + argument.len(),
                start..start + argument.len(),
                format!("_{argument}"),
            );
        }
    }
}

fn ambiguous_range(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let indent = line.len() - line.trim_start().len();
        let code = line.trim_start();
        for operator in ["...", ".."] {
            if !code.contains(").each do") {
                if let Some(at) = line.find(operator) {
                    let rhs = &line[at + operator.len()..];
                    let left_parenthesized = line[..at].trim_end().starts_with('(');
                    let right_parenthesized = rhs.trim_start().starts_with('(');
                    if rhs.contains('.') && left_parenthesized != right_parenthesized {
                        context.report(
                            "Wrap complex range boundaries with parentheses to avoid ambiguity.",
                            offset + at..offset + line.len(),
                        );
                    }
                }
                continue;
            }
            let Some(at) = code.find(operator) else {
                continue;
            };
            let Some(open) = code[..at].rfind('(') else {
                continue;
            };
            let Some(close) = code[at + operator.len()..]
                .rfind(").")
                .map(|relative| at + operator.len() + relative)
                .or_else(|| code.rfind(')'))
            else {
                continue;
            };
            let left = &code[open + 1..at];
            let right = &code[at + operator.len()..close];
            let boundary = if is_unparenthesized_range_arithmetic(left) {
                open + 1..at
            } else if is_unparenthesized_range_arithmetic(right) {
                at + operator.len()..close
            } else {
                continue;
            };
            let absolute = offset + indent + boundary.start..offset + indent + boundary.end;
            context.replace_many(
                "Wrap complex range boundaries with parentheses to avoid ambiguity.",
                absolute.clone(),
                vec![
                    (absolute.start..absolute.start, "(".to_string()),
                    (absolute.end..absolute.end, ")".to_string()),
                ],
            );
        }
    }
}

fn is_unparenthesized_range_arithmetic(boundary: &str) -> bool {
    !boundary.starts_with('(')
        && !boundary.ends_with(')')
        && [" + ", " - ", " * ", " / ", " % "]
            .iter()
            .any(|operator| boundary.contains(operator))
}

fn non_atomic_file_operation(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if (window[0].1.contains("File.exist?") || window[0].1.contains("File.exists?"))
            && ["File.delete", "File.rename", "FileUtils.rm", "FileUtils.mv"]
                .iter()
                .any(|operation| window[1].1.contains(operation))
        {
            context.report(
                "File operation is not atomic.",
                window[0].0..window[1].0 + window[1].1.len(),
            );
        }
    }
}

fn unmodified_reduce_accumulator(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        if !(line.contains(".reduce") || line.contains(".inject")) {
            continue;
        }
        let Some(pipe) = line.find('|') else { continue };
        let Some(close) = line[pipe + 1..].find('|').map(|at| pipe + 1 + at) else {
            continue;
        };
        let accumulator = line[pipe + 1..close].split(',').next().unwrap_or("").trim();
        let body = &line[close + 1..];
        if !accumulator.is_empty()
            && body.contains('}')
            && !body.trim_start().starts_with('}')
            && !body.contains(accumulator)
        {
            let start = offset + pipe + 1;
            context.report(
                "Ensure the reduce accumulator is modified in each iteration.",
                start..start + accumulator.len(),
            );
        }
    }
}

fn documentation_method(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut public = true;
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        let trimmed = line.trim_start();
        if matches!(trimmed.trim(), "private" | "protected") {
            public = false;
            continue;
        }
        if trimmed.trim() == "public" {
            public = true;
            continue;
        }
        if !public || !trimmed.starts_with("def ") || trimmed.starts_with("def _") {
            continue;
        }
        let documented = index > 0 && lines[index - 1].1.trim_start().starts_with('#');
        if !documented {
            let indent = line.len() - trimmed.len();
            let end = lines[index + 1..]
                .iter()
                .find(|(_, candidate)| candidate.trim() == "end")
                .map_or(offset + line.len(), |(end, candidate)| {
                    end + candidate.len()
                });
            context.report(
                "Missing method documentation comment.",
                offset + indent..end,
            );
        }
    }
}

fn redundant_splat_expansion(context: &mut CopContext<'_, '_>) {
    let source = context.source().to_string();
    let mut search = 0;
    while let Some(relative) = source[search..].find("[*") {
        let start = search + relative;
        let Some(close) = source[start + 2..].find(']').map(|at| start + 2 + at) else {
            break;
        };
        let value = source[start + 2..close].trim();
        if value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            context.replace(
                "Redundant splat expansion.",
                start..close + 1,
                start..close + 1,
                value,
            );
        }
        search = close + 1;
    }
}

fn method_call_parentheses(context: &mut CopContext<'_, '_>) {
    if context.policy().enforced_style("require_parentheses") != "require_parentheses" {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(['#', ':'])
            || [
                "class ", "module ", "def ", "super ", "raise ", "return ", "yield ", "alias ",
                "undef ",
            ]
            .iter()
            .any(|keyword| trimmed.starts_with(keyword))
            || trimmed.contains('=')
            || trimmed.contains('(')
        {
            continue;
        }
        let Some((method, argument)) = trimmed.split_once(char::is_whitespace) else {
            continue;
        };
        let argument = argument.trim();
        if argument.is_empty()
            || argument.starts_with(['+', '-', '*', '/', '%', '<', '>', '=', '&', '|'])
            || method.ends_with(['+', '-', '*', '/', '%', '<', '>', '='])
        {
            continue;
        }
        if !method
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.'))
        {
            continue;
        }
        let start = offset + line.find(trimmed).unwrap_or(0);
        context.replace(
            "Use parentheses for method calls with arguments.",
            start..start + trimmed.len(),
            start..start + trimmed.len(),
            format!("{method}({argument})"),
        );
    }
}

fn module_member_existence_check(context: &mut CopContext<'_, '_>) {
    for (old, new) in [
        (".constants.include?(", ".const_defined?("),
        (".methods.include?(", ".respond_to?("),
        (".instance_methods.include?(", ".method_defined?("),
    ] {
        for start in context.source_file().code_offsets(old) {
            let Some(close) = context.source()[start + old.len()..]
                .find(')')
                .map(|at| start + old.len() + at)
            else {
                continue;
            };
            let argument = &context.source()[start + old.len()..close];
            if argument.contains(',') || argument.trim_start().starts_with(['*', '&']) {
                continue;
            }
            context.replace(
                "Use the dedicated module member existence predicate.",
                start..start + old.len(),
                start..start + old.len(),
                new,
            );
        }
    }
}
