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
    let source = context.source().to_string();
    let ignore_empty = context.config_bool("IgnoreEmptyBlocks", true);
    let allow_keywords = context.config_bool("AllowUnusedKeywordArguments", false);
    let mut groups = block_parameter_groups(&source);
    groups.extend(lambda_parameter_groups(&source));
    for group in groups {
        let body_end = block_body_end(&source, group.body_start);
        let body = &source[group.body_start..body_end];
        if ignore_empty && body.trim_matches([' ', '\t', '\r', '\n', '}', ';']).is_empty() {
            continue;
        }
        if body.contains("binding") && !body.contains("binding(") && !body.contains("def ") {
            continue;
        }
        let unused = group
            .parameters
            .iter()
            .filter(|parameter| {
                !parameter.name.starts_with('_')
                    && !(allow_keywords && parameter.keyword)
                    && !(if parameter.local {
                        body.split(|character: char| {
                            !character.is_ascii_alphanumeric() && character != '_'
                        })
                        .any(|word| word == parameter.name)
                    } else {
                        block_variable_read(body, &parameter.name)
                    })
            })
            .collect::<Vec<_>>();
        for parameter in &unused {
            let all_unused = unused.len() == group.parameters.len();
            let message = unused_block_message(parameter, &group, all_unused);
            if parameter.keyword {
                context.report(message, parameter.range.clone());
            } else {
                context.replace(
                    message,
                    parameter.range.clone(),
                    parameter.range.clone(),
                    format!("_{}", parameter.name),
                );
            }
        }
    }
}

struct BlockParameterGroup {
    parameters: Vec<BlockParameterInfo>,
    body_start: usize,
    lambda: bool,
    define_method: bool,
}

struct BlockParameterInfo {
    name: String,
    range: std::ops::Range<usize>,
    keyword: bool,
    local: bool,
}

fn block_parameter_groups(source: &str) -> Vec<BlockParameterGroup> {
    let mut groups = Vec::new();
    let mut cursor = 0;
    while let Some(open) = source[cursor..].find('|').map(|at| cursor + at) {
        let Some(close) = source[open + 1..].find('|').map(|at| open + 1 + at) else {
            break;
        };
        if source.as_bytes().get(open.wrapping_sub(1)) == Some(&b'|') {
            cursor = close + 1;
            continue;
        }
        let prefix = &source[source[..open].rfind('\n').map_or(0, |at| at + 1)..open];
        if prefix.contains(" do") || prefix.contains('{') {
            groups.push(BlockParameterGroup {
                parameters: parse_block_parameters(source, open + 1, close),
                body_start: close + 1,
                lambda: false,
                define_method: prefix.contains("define_method"),
            });
        }
        cursor = close + 1;
    }
    groups
}

fn lambda_parameter_groups(source: &str) -> Vec<BlockParameterGroup> {
    source
        .match_indices("->")
        .filter_map(|(arrow, _)| {
            let open = source[arrow + 2..].find('(').map(|at| arrow + 2 + at)?;
            let close = source[open + 1..].find(')').map(|at| open + 1 + at)?;
            let body_start = source[close + 1..].find('{').map(|at| close + 2 + at)?;
            Some(BlockParameterGroup {
                parameters: parse_block_parameters(source, open + 1, close),
                body_start,
                lambda: true,
                define_method: false,
            })
        })
        .collect()
}

fn parse_block_parameters(source: &str, start: usize, end: usize) -> Vec<BlockParameterInfo> {
    let local_start = source[start..end].find(';').map(|at| start + at);
    let mut search = start;
    source[start..end]
        .split([',', ';'])
        .filter_map(|raw| {
            let token = raw.trim();
            let name = token
                .trim_start_matches('*')
                .split(['=', ':'])
                .next()
                .unwrap_or("")
                .trim();
            let relative = source[search..end].find(name)? + search;
            search = relative + name.len();
            (!name.is_empty()).then(|| BlockParameterInfo {
                name: name.to_string(),
                range: relative..relative + name.len(),
                keyword: token.contains(':'),
                local: local_start.is_some_and(|separator| relative > separator),
            })
        })
        .collect()
}

fn block_body_end(source: &str, start: usize) -> usize {
    let brace = source[start..].find('}').map(|at| start + at);
    let ending = source[start..]
        .match_indices("\nend")
        .map(|(at, _)| start + at)
        .next();
    brace.into_iter().chain(ending).min().unwrap_or(source.len())
}

fn block_variable_read(body: &str, name: &str) -> bool {
    body.lines().any(|line| {
        let inspected = line.split_once('=').map_or(line, |(_, right)| right);
        inspected.match_indices(name).any(|(at, _)| {
            let before = inspected.as_bytes().get(at.wrapping_sub(1)).copied();
            let after = inspected.as_bytes().get(at + name.len()).copied();
            let boundary = |byte: Option<u8>| {
                byte.is_none_or(|byte| !byte.is_ascii_alphanumeric() && byte != b'_')
            };
            boundary(before)
                && boundary(after)
                && inspected[..at].matches('\'').count() % 2 == 0
                && inspected[..at].matches('"').count() % 2 == 0
        })
    })
}

fn unused_block_message(
    parameter: &BlockParameterInfo,
    group: &BlockParameterGroup,
    all_unused: bool,
) -> String {
    if parameter.local {
        return format!("Unused block local variable - `{}`.", parameter.name);
    }
    let prefix = format!("Unused block argument - `{}`.", parameter.name);
    if group.lambda && all_unused {
        return format!("{prefix} If it's necessary, use `_` or `_{}` as an argument name to indicate that it won't be used. Also consider using a proc without arguments instead of a lambda if you want it to accept any arguments but don't care about them.", parameter.name);
    }
    if group.define_method || !all_unused {
        return format!("{prefix} If it's necessary, use `_` or `_{}` as an argument name to indicate that it won't be used.", parameter.name);
    }
    if group.parameters.len() == 1 {
        format!("{prefix} You can omit the argument if you don't care about it.")
    } else {
        format!("{prefix} You can omit all the arguments if you don't care about them.")
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
