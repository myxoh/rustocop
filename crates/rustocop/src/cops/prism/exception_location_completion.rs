use super::*;

define_cops! {
    SuppressedExceptionInNumberConversion => "Lint/SuppressedExceptionInNumberConversion" => source(suppressed_exception_in_number_conversion),
    EvalWithLocation => "Style/EvalWithLocation" => source(eval_with_location),
}

fn suppressed_exception_in_number_conversion(context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(2, 6) {
        return;
    }
    let source = context.source();
    for method in ["Integer", "Float", "BigDecimal", "Complex", "Rational"] {
        let mut search = 0;
        while let Some(relative) = source[search..].find(&format!("{method}(")) {
            let call_start = search + relative;
            let constructor_start = ["::Kernel&.", "::Kernel::", "Kernel::"]
                .into_iter()
                .find_map(|prefix| {
                    source[..call_start]
                        .ends_with(prefix)
                        .then(|| call_start - prefix.len())
                })
                .unwrap_or(call_start);
            let Some(close_relative) = source[call_start..].find(')') else {
                break;
            };
            let close = call_start + close_relative;
            let call = &source[call_start..=close];
            if call.contains("exception:") {
                search = close + 1;
                continue;
            }
            if method == "Float" && call[..call.len() - 1].contains(',') {
                search = close + 1;
                continue;
            }
            let tail = &source[close + 1..];
            let (offense_start, offense_end) = if tail.starts_with(" rescue nil") {
                (constructor_start, close + 1 + " rescue nil".len())
            } else if tail.starts_with('\n') {
                let tail_lines = tail.lines().collect::<Vec<_>>();
                let rescue = tail_lines.get(1).map_or("", |line| line.trim());
                let allowed_rescue = rescue == "rescue"
                    || rescue.strip_prefix("rescue ").is_some_and(|exceptions| {
                        exceptions.split(',').all(|exception| {
                            matches!(
                                exception.trim().trim_start_matches("::"),
                                "ArgumentError" | "TypeError"
                            )
                        })
                    });
                let body = tail_lines.get(2).map_or("", |line| line.trim());
                let end_index = if body == "nil"
                    && tail_lines.get(3).is_some_and(|line| line.trim() == "end")
                {
                    3
                } else if body == "end" {
                    2
                } else {
                    search = close + 1;
                    continue;
                };
                if !allowed_rescue {
                    search = close + 1;
                    continue;
                }
                let Some(begin) = source[..call_start]
                    .rfind("begin")
                    .filter(|begin| begin + "begin".len() <= call_start)
                else {
                    search = close + 1;
                    continue;
                };
                if !source[begin + "begin".len()..call_start].trim().is_empty() {
                    search = close + 1;
                    continue;
                }
                let relative_end = tail
                    .match_indices('\n')
                    .nth(end_index)
                    .map_or(tail.len(), |(at, _)| at);
                (begin, close + 1 + relative_end)
            } else {
                search = close + 1;
                continue;
            };
            let preferred = format!("{}, exception: false)", &source[constructor_start..close]);
            let indent_start = context.source_file().line_start(offense_start);
            let indent = &source[indent_start..offense_start];
            context.replace(
                format!("Use `{preferred}` instead."),
                offense_start..offense_end,
                offense_start..offense_end,
                format!("{indent}{preferred}").trim_start_matches(indent),
            );
            search = offense_end;
        }
    }
}

fn eval_with_location(context: &mut CopContext<'_, '_>) {
    correct_existing_eval_location(context);
    check_eval_calls(context);
}

fn correct_existing_eval_location(context: &mut CopContext<'_, '_>) -> bool {
    let source = context.source();
    if let Some(eval_start) = source.find("eval(") {
        if let Some(close_relative) = source[eval_start..].find(')') {
            let close = eval_start + close_relative;
            let call = &source[eval_start..=close];
            if call.contains('\n') && call.contains("__FILE__") && call.contains("__LINE__") {
                let line_at = eval_start + call.find("__LINE__").unwrap_or(0);
                let delta = source[eval_start..line_at].matches('\n').count();
                let actual_end = source[line_at..]
                    .find([',', ')'])
                    .map_or(source.len(), |at| line_at + at);
                let actual = source[line_at..actual_end].trim_end();
                let expected = if delta == 0 {
                    "__LINE__".to_string()
                } else {
                    format!("__LINE__ - {delta}")
                };
                if actual != expected {
                    context.replace(
                        format!("Incorrect line number for `eval`; use `{expected}` instead of `{actual}`."),
                        line_at..line_at + actual.len(),
                        line_at..line_at + actual.len(),
                        expected,
                    );
                    return true;
                }
            }
        }
    }
    if let (Some(eval_start), Some(line_at)) = (source.find("eval"), source.find("__LINE__")) {
        if line_at > eval_start
            && source[eval_start..line_at].contains("binding")
            && source[eval_start..line_at].contains("__FILE__")
        {
            let delta = source[eval_start..line_at].matches('\n').count();
            let actual_end = source[line_at..]
                .find([',', ')', '\n'])
                .map_or(source.len(), |at| line_at + at);
            let actual = source[line_at..actual_end].trim_end();
            let expected = if source[eval_start..line_at].contains("<<") {
                "__LINE__ + 1".to_string()
            } else if delta == 0 {
                "__LINE__".to_string()
            } else {
                format!("__LINE__ - {delta}")
            };
            if actual != expected {
                context.replace(
                    format!(
                        "Incorrect line number for `eval`; use `{expected}` instead of `{actual}`."
                    ),
                    line_at..line_at + actual.len(),
                    line_at..line_at + actual.len(),
                    expected,
                );
                return true;
            }
        }
    }
    false
}

fn check_eval_calls(context: &mut CopContext<'_, '_>) {
    let multiline_starts = check_multiline_eval_locations(context);
    let excluded = context
        .source_file()
        .literal_ranges()
        .into_iter()
        .chain(context.source_file().comment_ranges())
        .collect::<Vec<_>>();
    for (line_number, (offset, line)) in context.source_file().lines().enumerate() {
        if multiline_starts.contains(&line_number) {
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        let Some((method, at)) = eval_method(trimmed) else {
            continue;
        };
        let method_absolute = offset + line.len() - trimmed.len() + at;
        if excluded
            .iter()
            .any(|range| range.start <= method_absolute && method_absolute < range.end)
        {
            continue;
        }
        let call = &trimmed[at..];
        if at == 0 && call.trim_end().ends_with(',') && !call.contains(')') {
            continue;
        }
        if at == 0 && (call.trim_end().ends_with(" do") || call.contains(" do |")) {
            continue;
        }
        let display_method = method.rsplit('.').next().unwrap_or(method);
        let method_end = call
            .find(display_method)
            .map_or(display_method.len(), |at| at + display_method.len());
        let argument = call[method_end..].trim_start_matches([' ', '(']);
        if argument.starts_with("<<") && argument.contains(".lines") {
            continue;
        }
        if !argument.starts_with(['\'', '"', '%', '<'])
            || argument.starts_with('%')
                && argument
                    .as_bytes()
                    .get(1)
                    .is_some_and(|byte| matches!(byte, b'w' | b'W' | b'i' | b'I'))
        {
            continue;
        }
        if call.contains("__FILE__") && call.contains("__LINE__") {
            if display_method == "eval" {
                continue;
            }
            if let Some(line_at) = call.find("__LINE__") {
                let expected = if call.contains("<<") {
                    "__LINE__ + 1"
                } else {
                    "__LINE__"
                };
                let actual = call[line_at..]
                    .split([',', ')'])
                    .next()
                    .unwrap_or("__LINE__")
                    .trim();
                if actual != expected {
                    let start = offset + trimmed.len() - line.trim_start().len() + at + line_at;
                    context.replace(
                        format!("Incorrect line number for `{display_method}`; use `{expected}` instead of `{actual}`."),
                        start..start + actual.len(),
                        start..start + actual.len(),
                        expected,
                    );
                }
            }
            continue;
        }
        if call.contains(',') {
            let arguments = call.split(',').collect::<Vec<_>>();
            let required = if method == "eval" { 4 } else { 3 };
            if arguments.len() >= required {
                let file = arguments[arguments.len() - 2].trim();
                let line_value = arguments[arguments.len() - 1].trim().trim_end_matches(')');
                let expected_line = if call.contains("<<") {
                    "__LINE__ + 1"
                } else {
                    "__LINE__"
                };
                let display = method.trim_start_matches("::Kernel.");
                let file_at = offset + line.find(file).unwrap_or(0);
                let line_at = offset + line.rfind(line_value).unwrap_or(0);
                if file != "__FILE__" {
                    context.replace(
                        format!(
                            "Incorrect file for `{display}`; use `__FILE__` instead of `{file}`."
                        ),
                        file_at..file_at + file.len(),
                        file_at..file_at + file.len(),
                        "__FILE__",
                    );
                }
                if line_value != expected_line
                    && (line_value.starts_with(['\'', '"'])
                        || line_value.starts_with("__LINE__")
                        || line_value.bytes().all(|byte| byte.is_ascii_digit()))
                {
                    context.replace(
                        format!("Incorrect line number for `{display}`; use `{expected_line}` instead of `{line_value}`."),
                        line_at..line_at + line_value.len(),
                        line_at..line_at + line_value.len(),
                        expected_line,
                    );
                }
                continue;
            }
        }
        let receiver_at = if method == "eval" || method.contains("Kernel") {
            at
        } else {
            0
        };
        let call_start = offset + line.len() - trimmed.len() + receiver_at;
        let comment_at = context
            .source_file()
            .comment_ranges()
            .into_iter()
            .filter(|range| offset <= range.start && range.start < offset + line.len())
            .map(|range| range.start - offset)
            .min();
        let code = line[..comment_at.unwrap_or(line.len())].trim_end();
        let mut call_end = offset + code.len();
        let display = if method.ends_with("eval") && method.contains("Kernel") {
            "eval"
        } else {
            method
        };
        let message = if display == "eval" {
            "Pass a binding, `__FILE__`, and `__LINE__` to `eval`.".to_string()
        } else {
            format!("Pass `__FILE__` and `__LINE__` to `{display}`.")
        };
        if display == "eval" && !call.contains("binding") {
            if let Some(quote_relative) = call[method_end..].find(['\'', '"']) {
                let quote = method_absolute + method_end + quote_relative;
                if let Some(close) = closing_quote(context.source(), quote) {
                    call_end = close
                        + 1
                        + usize::from(context.source().as_bytes().get(close + 1) == Some(&b')'));
                }
            }
            context.report(message, call_start..call_end);
            continue;
        }
        let line_expression = if call.contains("<<") {
            "__LINE__ + 1"
        } else {
            "__LINE__"
        };
        let addition = if display == "eval" && call.contains("__FILE__") {
            format!(", {line_expression}")
        } else if display == "eval" {
            format!(", __FILE__, {line_expression}")
        } else {
            let _ = line_number;
            format!(", __FILE__, {line_expression}")
        };
        let insert = if call.ends_with(')') {
            call_end - 1
        } else {
            call_end
        };
        context.insert(message, call_start..call_end, insert, addition);
    }
}

fn check_multiline_eval_locations(
    context: &mut CopContext<'_, '_>,
) -> std::collections::HashSet<usize> {
    let source = context.source();
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut handled = std::collections::HashSet::new();
    for (line_number, (offset, line)) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        let Some((method, at)) = eval_method(trimmed) else {
            continue;
        };
        if !matches!(method.rsplit('.').next(), Some("class_eval" | "module_eval" | "instance_eval")) {
            continue;
        }
        let method_absolute = *offset + line.len() - trimmed.len() + at;
        let method_end = method_absolute + method.rsplit('.').next().unwrap_or(method).len();
        let Some(quote_relative) = source[method_end..].find(['\'', '"']) else {
            continue;
        };
        let quote = method_end + quote_relative;
        let Some(close) = closing_quote(source, quote) else {
            continue;
        };
        if !source[quote..close].contains('\n') {
            continue;
        }
        let closing_line_end = context.source_file().line_end(close);
        let tail = &source[close + 1..closing_line_end];
        let Some(arguments) = tail.strip_prefix(',') else {
            continue;
        };
        let arguments = arguments.split(',').map(str::trim).collect::<Vec<_>>();
        if arguments.len() < 2 {
            continue;
        }
        let file = arguments[0];
        let line_value = arguments[1].trim_end_matches(')');
        let file_at = close + 1 + tail.find(file).unwrap_or(0);
        let line_at = close + 1 + tail.rfind(line_value).unwrap_or(0);
        let delta = source[*offset..line_at].matches('\n').count();
        let expected_line = format!("__LINE__ - {delta}");
        let display = method.rsplit('.').next().unwrap_or(method);
        if file != "__FILE__" {
            context.replace(
                format!("Incorrect file for `{display}`; use `__FILE__` instead of `{file}`."),
                file_at..file_at + file.len(),
                file_at..file_at + file.len(),
                "__FILE__",
            );
        }
        if line_value != expected_line {
            context.replace(
                format!("Incorrect line number for `{display}`; use `{expected_line}` instead of `{line_value}`."),
                line_at..line_at + line_value.len(),
                line_at..line_at + line_value.len(),
                expected_line,
            );
        }
        handled.insert(line_number);
    }
    handled
}

fn closing_quote(source: &str, opening: usize) -> Option<usize> {
    let quote = *source.as_bytes().get(opening)?;
    let mut escaped = false;
    for (relative, byte) in source.as_bytes()[opening + 1..].iter().copied().enumerate() {
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == quote {
            return Some(opening + 1 + relative);
        }
    }
    None
}

fn eval_method(line: &str) -> Option<(&str, usize)> {
    let (method, at) = [
        "::Kernel.eval",
        "Kernel.eval",
        "class_eval",
        "module_eval",
        "instance_eval",
        "eval",
    ]
    .into_iter()
    .find_map(|name| line.find(name).map(|at| (name, at)))?;
    if method == "eval" && at > 0 && line.as_bytes().get(at - 1) == Some(&b'.') {
        None
    } else {
        Some((method, at))
    }
}
