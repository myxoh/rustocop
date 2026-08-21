use super::*;

define_cops! {
    Syntax => "Lint/Syntax" => parse_error(syntax),
    FormatParameterMismatch => "Lint/FormatParameterMismatch" => source(format_parameter_mismatch),
    UnusedBlockArgument => "Lint/UnusedBlockArgument" => source(unused_block_argument),
    AmbiguousRange => "Lint/AmbiguousRange" => source(ambiguous_range),
    NonAtomicFileOperation => "Lint/NonAtomicFileOperation" => source(non_atomic_file_operation),
    UnmodifiedReduceAccumulator => "Lint/UnmodifiedReduceAccumulator" => source(unmodified_reduce_accumulator),
    DocumentationMethod => "Style/DocumentationMethod" => source(documentation_method),
    RedundantSplatExpansion => "Lint/RedundantSplatExpansion" => source(redundant_splat_expansion),
}

fn syntax(error: &Diagnostic<'_>, context: &mut CopContext<'_, '_>) {
    let location = error.location();
    let start = location.start_offset().min(context.source().len());
    let end = location.end_offset().max(start).min(context.source().len());
    context.report(error.message(), start..end);
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
        let Some(method) = line
            .find(".reduce")
            .or_else(|| line.find(".inject"))
        else {
            continue;
        };
        let pipes = line[method..]
            .match_indices('|')
            .map(|(at, _)| method + at)
            .collect::<Vec<_>>();
        let candidates = pipes
            .windows(2)
            .filter_map(|pair| {
                let prefix = line[..pair[0]].trim_end();
                let brace_block = prefix.ends_with('{');
                let do_block = prefix
                    .strip_suffix("do")
                    .is_some_and(|before| before.is_empty() || before.ends_with(char::is_whitespace));
                let parameters = &line[pair[0] + 1..pair[1]];
                ((brace_block || do_block)
                    && parameters.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric()
                            || matches!(byte, b'_' | b',' | b' ' | b'*' | b';')
                    }))
                .then_some((pair[0], pair[1], do_block))
            })
            .collect::<Vec<_>>();
        let selected = candidates
            .iter()
            .rev()
            .find(|(_, _, do_block)| *do_block)
            .or_else(|| candidates.first());
        let Some(&(pipe, close, _)) = selected else {
            continue;
        };
        let mut parameters = line[pipe + 1..close].split(',').map(str::trim);
        let accumulator = parameters.next().unwrap_or("");
        let element = parameters.next().unwrap_or("");
        let body = &line[close + 1..];
        if accumulator.is_empty() {
            continue;
        }
        let expression_start = body.rfind(';').map_or(0, |at| at + 1);
        let expression = &body[expression_start..];
        let leading = expression.len() - expression.trim_start().len();
        let returned = expression.trim().trim_end_matches('}').trim_end();
        if let Some(relative_index) = returned
            .strip_prefix(accumulator)
            .filter(|suffix| suffix.starts_with('['))
            .map(|_| expression_start + leading)
        {
            let Some(end) = body[relative_index..]
                .find(']')
                .map(|at| relative_index + at + 1)
            else {
                continue;
            };
            let index_argument = &body[relative_index + accumulator.len() + 1..end - 1];
            let tail = returned[end - relative_index..].trim();
            let assignment = tail.starts_with('=') && !tail.starts_with("==");
            if !tail.is_empty() && !assignment {
                continue;
            }
            if !assignment && index_argument.trim() == element {
                continue;
            }
            let method_name = if line[method..].starts_with(".inject") {
                "inject"
            } else {
                "reduce"
            };
            context.report(
                format!("Do not return an element of the accumulator in `{method_name}`."),
                offset + close + 1 + relative_index..offset + close + 1 + end,
            );
        } else if body.contains('}')
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
