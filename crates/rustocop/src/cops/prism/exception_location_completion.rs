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
            } else if tail.starts_with("\nrescue") {
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
                let begin = source[..call_start].rfind("begin").unwrap_or(call_start);
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
    if !correct_existing_eval_location(context) {
        check_eval_calls(context);
    }
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
                }
                return true;
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
            }
            return true;
        }
    }
    false
}

fn check_eval_calls(context: &mut CopContext<'_, '_>) {
    for (line_number, (offset, line)) in context.source_file().lines().enumerate() {
        let trimmed = line.trim_start();
        let Some(method_at) = [
            "::Kernel.eval",
            "Kernel.eval",
            "class_eval",
            "module_eval",
            "instance_eval",
            "eval",
        ]
        .into_iter()
        .find_map(|name| trimmed.find(name).map(|at| (name, at))) else {
            continue;
        };
        let (method, at) = method_at;
        if method == "eval" && at > 0 && trimmed.as_bytes().get(at.saturating_sub(1)) == Some(&b'.')
        {
            continue;
        }
        let call = &trimmed[at..];
        if call.trim_end().ends_with(" do") || call.contains(" do |") {
            continue;
        }
        if method == "eval" {
            let argument = call["eval".len()..].trim_start_matches([' ', '(']);
            if !argument.starts_with(['\'', '"', '%', '<']) {
                continue;
            }
        }
        if call.contains("__FILE__") && call.contains("__LINE__") {
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
                        format!("Incorrect line number for `{}`; use `{expected}` instead of `{actual}`.", method.trim_start_matches("::Kernel.")),
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
                    && (line_value.starts_with(['\'', '"']) || line_value.starts_with("__LINE__"))
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
        let call_end = offset + line.trim_end().len();
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
