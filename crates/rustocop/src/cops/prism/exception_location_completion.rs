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
    let mut non_code_ranges = context.source_file().literal_ranges();
    non_code_ranges.extend(context.source_file().heredoc_ranges());
    non_code_ranges.extend(context.source_file().comment_ranges());
    for method in ["Integer", "Float", "BigDecimal", "Complex", "Rational"] {
        let mut search = 0;
        while let Some(relative) = source[search..].find(&format!("{method}(")) {
            let call_start = search + relative;
            if non_code_ranges
                .iter()
                .any(|range| range.start <= call_start && call_start < range.end)
            {
                search = call_start + method.len();
                continue;
            }
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
    eval_with_location_ast(context);
}

fn eval_with_location_ast(context: &mut CopContext<'_, '_>) {
    #[derive(Default)]
    struct EvalCalls<'pr>(Vec<Node<'pr>>);
    impl<'pr> ruby_prism::Visit<'pr> for EvalCalls<'pr> {
        fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
            if matches!(node.name().as_slice(), b"eval" | b"class_eval" | b"module_eval" | b"instance_eval") {
                self.0.push(node.as_node());
            }
            ruby_prism::visit_call_node(self, node);
        }
    }

    let parsed = ruby_prism::parse(context.source().as_bytes());
    let mut calls = EvalCalls::default();
    ruby_prism::Visit::visit(&mut calls, &parsed.node());
    for call in calls.0 {
        if let Some(call) = call.as_call_node() {
            check_eval_call_ast(context, &call);
        }
    }
}

fn check_eval_call_ast(context: &mut CopContext<'_, '_>, node: &ruby_prism::CallNode<'_>) {
    let method = String::from_utf8_lossy(node.name().as_slice());
    if method == "eval" && node.receiver().is_some_and(|receiver| {
        !matches!(context.source_file().node(&receiver).trim_start_matches("::"), "Kernel")
    }) {
        return;
    }
    let arguments = node
        .arguments()
        .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
        .unwrap_or_default();
    let Some(code) = arguments.first() else { return };
    if code.as_string_node().is_none() && code.as_interpolated_string_node().is_none() {
        return;
    }
    let base = usize::from(method == "eval") + 1;
    let file = arguments.get(base);
    let line = arguments.get(base + 1);
    if let Some(line) = line {
        if let Some(file) = file {
            check_eval_file_ast(context, &method, file);
        }
        check_eval_line_ast(context, &method, code, line);
    } else if let Some(file) = file {
        check_eval_file_ast(context, &method, file);
        add_missing_eval_line_ast(context, node, &method, code, arguments.last().unwrap());
    } else {
        add_missing_eval_location_ast(context, node, &method, code, &arguments);
    }
}

fn check_eval_file_ast(
    context: &mut CopContext<'_, '_>,
    method: &str,
    file: &Node<'_>,
) {
    let actual = context.source_file().node(file);
    if actual == "__FILE__" {
        return;
    }
    context.replace(
        format!("Incorrect file for `{method}`; use `__FILE__` instead of `{actual}`."),
        file.location(),
        file.location(),
        "__FILE__",
    );
}

fn check_eval_line_ast(
    context: &mut CopContext<'_, '_>,
    method: &str,
    code: &Node<'_>,
    line: &Node<'_>,
) {
    if eval_variable(line)
        || line.as_call_node().is_some_and(|call| call.name().as_slice() != b"+")
    {
        return;
    }
    let difference = eval_code_first_line(context.source(), code) as isize
        - eval_line_at(context.source(), line.location().start_offset()) as isize;
    let expected = eval_expected_line(difference);
    if eval_line_matches(context.source_file().node(line), difference) {
        return;
    }
    let actual = context.source_file().node(line);
    context.replace(
        format!("Incorrect line number for `{method}`; use `{expected}` instead of `{actual}`."),
        line.location(),
        line.location(),
        expected,
    );
}

fn add_missing_eval_line_ast(
    context: &mut CopContext<'_, '_>,
    node: &ruby_prism::CallNode<'_>,
    method: &str,
    code: &Node<'_>,
    last: &Node<'_>,
) {
    let difference = eval_code_first_line(context.source(), code) as isize
        - eval_line_at(context.source(), last.location().start_offset()) as isize;
    let message = eval_missing_message(method);
    let offense = eval_call_range(node, code);
    context.insert(
        message,
        offense,
        last.location().end_offset(),
        format!(", {}", eval_expected_line(difference)),
    );
}

fn add_missing_eval_location_ast(
    context: &mut CopContext<'_, '_>,
    node: &ruby_prism::CallNode<'_>,
    method: &str,
    code: &Node<'_>,
    arguments: &[Node<'_>],
) {
    let message = eval_missing_message(method);
    let offense = eval_call_range(node, code);
    if method == "eval" && arguments.len() < 2 {
        context.report(message, offense);
        return;
    }
    let last = arguments.last().expect("string argument checked");
    let difference = eval_code_first_line(context.source(), code) as isize
        - eval_line_at(context.source(), last.location().start_offset()) as isize;
    context.insert(
        message,
        offense,
        last.location().end_offset(),
        format!(", __FILE__, {}", eval_expected_line(difference)),
    );
}

fn eval_missing_message(method: &str) -> String {
    if method == "eval" {
        "Pass a binding, `__FILE__`, and `__LINE__` to `eval`.".to_string()
    } else {
        format!("Pass `__FILE__` and `__LINE__` to `{method}`.")
    }
}

fn eval_call_range(node: &ruby_prism::CallNode<'_>, code: &Node<'_>) -> std::ops::Range<usize> {
    let location = node.location();
    let _ = code;
    location.start_offset()..location.end_offset()
}

fn eval_code_first_line(source: &str, code: &Node<'_>) -> usize {
    let opening = code
        .as_string_node()
        .and_then(|string| string.opening_loc())
        .or_else(|| code.as_interpolated_string_node().and_then(|string| string.opening_loc()));
    if opening.as_ref().is_some_and(|opening| opening.as_slice().starts_with(b"<<")) {
        let opening = opening.unwrap();
        return source[opening.end_offset()..]
            .find('\n')
            .map_or_else(
                || eval_line_at(source, opening.start_offset()),
                |newline| eval_line_at(source, opening.end_offset() + newline + 1),
            );
    }
    eval_line_at(source, code.location().start_offset())
}

fn eval_line_at(source: &str, offset: usize) -> usize {
    source[..offset.min(source.len())].bytes().filter(|byte| *byte == b'\n').count() + 1
}

fn eval_expected_line(difference: isize) -> String {
    match difference.cmp(&0) {
        std::cmp::Ordering::Equal => "__LINE__".to_string(),
        std::cmp::Ordering::Greater => format!("__LINE__ + {difference}"),
        std::cmp::Ordering::Less => format!("__LINE__ - {}", difference.unsigned_abs()),
    }
}

fn eval_line_matches(source: &str, difference: isize) -> bool {
    let compact = source.split_whitespace().collect::<String>();
    match difference.cmp(&0) {
        std::cmp::Ordering::Equal => compact == "__LINE__",
        std::cmp::Ordering::Greater => {
            compact == format!("__LINE__+{difference}")
                || compact == format!("{difference}+__LINE__")
        }
        std::cmp::Ordering::Less => compact == format!("__LINE__-{}", difference.unsigned_abs()),
    }
}

fn eval_variable(node: &Node<'_>) -> bool {
    node.as_local_variable_read_node().is_some()
        || node.as_instance_variable_read_node().is_some()
        || node.as_class_variable_read_node().is_some()
        || node.as_global_variable_read_node().is_some()
}
