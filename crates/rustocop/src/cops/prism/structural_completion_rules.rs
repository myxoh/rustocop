use ruby_prism::{CallNode, ClassNode, ModuleNode, SingletonClassNode};

use super::*;

define_cops!(
    MultilineMemoization => "Style/MultilineMemoization" => source(multiline_memoization),
    StaticClass => "Style/StaticClass" => node(as_class_node, static_class),
    TrailingBodyOnClass => "Style/TrailingBodyOnClass" => any_node(trailing_body_on_class),
    TrailingBodyOnModule => "Style/TrailingBodyOnModule" => node(as_module_node, trailing_body_on_module),
    YodaExpression => "Style/YodaExpression" => call(yoda_expression),
);

fn multiline_memoization(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let Some(operator) = source.find("||=") else {
        return;
    };
    if !source[operator + 3..].contains('\n') {
        return;
    }
    let style = context.policy().enforced_style("keyword");
    if style == "keyword" {
        let Some(open) = source[operator + 3..].find('(').map(|at| operator + 3 + at) else {
            return;
        };
        let Some(close) = super::source_syntax::matching_delimiter(source, open, b'(', b')') else {
            return;
        };
        let content = &source[open + 1..close];
        if !content.contains('\n') {
            return;
        }
        let (opening, closing) = if content.starts_with('\n') {
            ("begin".to_string(), "end".to_string())
        } else {
            let continuation = content
                .split_once('\n')
                .map_or(0, |(_, rest)| rest.len() - rest.trim_start().len());
            (
                format!("begin\n{}", " ".repeat(continuation)),
                format!("\n{}end", " ".repeat(continuation.saturating_sub(2))),
            )
        };
        context.replace_many(
            "Wrap multiline memoization blocks in `begin` and `end`.",
            0..close + 1,
            vec![(open..open + 1, opening), (close..close + 1, closing)],
        );
    } else if style == "braces" {
        let Some(begin) = source[operator + 3..]
            .find("begin")
            .map(|at| operator + 3 + at)
        else {
            return;
        };
        let Some(end) = source.rfind("end") else {
            return;
        };
        if end <= begin || !source[begin..end].contains('\n') {
            return;
        }
        context.replace_many(
            "Wrap multiline memoization blocks in `(` and `)`.",
            0..end + 3,
            vec![
                (begin..begin + 5, "(".to_string()),
                (end..end + 3, ")".to_string()),
            ],
        );
    }
}

fn static_class(node: &ClassNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.superclass().is_some() {
        return;
    }
    let Some(body) = node.body().and_then(|body| body.as_statements_node()) else {
        return;
    };
    let mut singleton_methods = Vec::new();
    let mut singleton_classes = Vec::new();
    for statement in &body.body() {
        if let Some(definition) = statement.as_def_node() {
            if definition.receiver().is_none() {
                return;
            }
            singleton_methods.push(definition);
        } else if let Some(singleton) = statement.as_singleton_class_node() {
            if singleton
                .body()
                .and_then(|body| body.as_statements_node())
                .is_some_and(|statements| {
                    statements
                        .body()
                        .iter()
                        .any(|entry| entry.as_def_node().is_none() && !is_assignment(&entry))
                })
            {
                return;
            }
            singleton_classes.push(singleton);
        } else if let Some(call) = statement.as_call_node() {
            if call.name().as_slice() != b"extend" {
                return;
            }
        } else if !is_assignment(&statement) {
            return;
        }
    }
    let location = node.location();
    let header_end = node.constant_path().location().end_offset();
    let indentation = context
        .source_file()
        .indentation(location.start_offset())
        .len();
    let mut edits = vec![
        (
            node.class_keyword_loc().start_offset()..node.class_keyword_loc().end_offset(),
            "module".to_string(),
        ),
        (
            header_end..header_end,
            format!("\n{}module_function\n", " ".repeat(indentation)),
        ),
    ];
    for definition in singleton_methods {
        let receiver = definition.receiver().expect("singleton method receiver");
        edits.push((
            receiver.location().start_offset()..definition.name_loc().start_offset(),
            String::new(),
        ));
    }
    for singleton in singleton_classes {
        edits.push((
            singleton.class_keyword_loc().start_offset()
                ..singleton.expression().location().end_offset(),
            String::new(),
        ));
        edits.push((
            singleton.end_keyword_loc().start_offset()..singleton.end_keyword_loc().end_offset(),
            String::new(),
        ));
    }
    context.replace_many(
        "Prefer modules to classes with only class methods.",
        &location,
        edits,
    );
}

fn is_assignment(node: &Node<'_>) -> bool {
    node.as_constant_write_node().is_some()
        || node.as_constant_path_write_node().is_some()
        || node.as_local_variable_write_node().is_some()
        || node.as_instance_variable_write_node().is_some()
        || node.as_class_variable_write_node().is_some()
        || node.as_global_variable_write_node().is_some()
        || node.as_multi_write_node().is_some()
}

fn trailing_body_on_class(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let location = node.location();
    if context.source_file().same_line(
        location.start_offset(),
        location.end_offset().saturating_sub(1),
    ) {
        return;
    }
    let parts = if let Some(class) = node.as_class_node() {
        class_parts(&class)
    } else if let Some(class) = node.as_singleton_class_node() {
        singleton_class_parts(&class)
    } else {
        return;
    };
    let Some((header_end, body)) = parts else {
        return;
    };
    report_trailing_body(
        node.location().start_offset(),
        header_end,
        body,
        "Place the first line of class body on its own line.",
        context,
    );
}

fn trailing_body_on_module(node: &ModuleNode<'_>, context: &mut CopContext<'_, '_>) {
    let location = node.location();
    if context.source_file().same_line(
        location.start_offset(),
        location.end_offset().saturating_sub(1),
    ) {
        return;
    }
    let Some(body) = node.body() else {
        return;
    };
    report_trailing_body(
        node.location().start_offset(),
        node.constant_path().location().end_offset(),
        body,
        "Place the first line of module body on its own line.",
        context,
    );
}

fn yoda_expression(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let operator = node.name().as_slice();
    if !context
        .config_values("SupportedOperators")
        .iter()
        .any(|configured| configured.as_bytes() == operator)
    {
        return;
    }
    let Some(left) = node.receiver() else {
        return;
    };
    let Some(right) = only_argument(node) else {
        return;
    };
    if !yoda_literal(&left) || yoda_literal(&right) {
        return;
    }
    let nested_in_yoda = context.ancestors().iter().rev().any(|ancestor| {
        ancestor.as_call_node().is_some_and(|call| {
            call.receiver().as_ref().is_some_and(yoda_literal)
                && only_argument(&call).is_some_and(|argument| !yoda_literal(&argument))
        })
    });
    if nested_in_yoda {
        return;
    }
    let right_source = context.source_file().node(&right);
    let replacement = render_yoda(node, context.source_file());
    context.replace_call(
        node,
        format!("Non-literal operand (`{right_source}`) should be first."),
        replacement,
    );
}

fn yoda_literal(node: &Node<'_>) -> bool {
    node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_rational_node().is_some()
        || node.as_imaginary_node().is_some()
        || node.as_constant_read_node().is_some()
        || node.as_constant_path_node().is_some()
}

fn render_yoda(node: &CallNode<'_>, file: SourceFile<'_>) -> String {
    let Some(left) = node.receiver() else {
        return file.node(&node.as_node()).to_string();
    };
    let Some(right) = only_argument(node) else {
        return file.node(&node.as_node()).to_string();
    };
    let left = render_yoda_node(&left, file);
    let right = render_yoda_node(&right, file);
    let operator = String::from_utf8_lossy(node.name().as_slice());
    if node
        .call_operator_loc()
        .is_some_and(|operator| operator.as_slice() == b".")
        && node.opening_loc().is_some()
    {
        format!("{right}.{operator}({left})")
    } else {
        format!("{right} {operator} {left}")
    }
}

fn render_yoda_node(node: &Node<'_>, file: SourceFile<'_>) -> String {
    if let Some(parentheses) = node.as_parentheses_node() {
        if let Some(body) = parentheses.body() {
            let expression = body
                .as_statements_node()
                .and_then(|statements| statements.body().first())
                .unwrap_or(body);
            if let Some(call) = expression.as_call_node() {
                if call.receiver().as_ref().is_some_and(yoda_literal)
                    && only_argument(&call).is_some_and(|argument| !yoda_literal(&argument))
                {
                    return format!("({})", render_yoda(&call, file));
                }
            }
        }
    }
    if let Some(call) = node.as_call_node() {
        if call.receiver().as_ref().is_some_and(yoda_literal)
            && only_argument(&call).is_some_and(|argument| !yoda_literal(&argument))
        {
            return render_yoda(&call, file);
        }
    }
    file.node(node).to_string()
}

pub(super) fn report_trailing_body(
    definition_start: usize,
    header_end: usize,
    body: Node<'_>,
    message: &str,
    context: &mut CopContext<'_, '_>,
) {
    let Some(statements) = body.as_statements_node() else {
        return;
    };
    let Some(first) = statements.body().first() else {
        return;
    };
    let first_location = first.location();
    let file = context.source_file();
    if !file.same_line(definition_start, first_location.start_offset()) {
        return;
    }
    let line_end = file.line_end(first_location.end_offset());
    let indentation = file.column(definition_start);
    let indentation_width = context
        .related_config_value("Layout/IndentationWidth", "Width")
        .and_then(|width| width.parse::<usize>().ok())
        .unwrap_or(2);
    let mut replacement = context.source()[header_end..first_location.start_offset()].to_string();
    if let Some(semicolon) = replacement.find(';') {
        replacement.remove(semicolon);
    }
    replacement.push('\n');
    replacement.push_str(&" ".repeat(indentation + indentation_width));
    let mut edits = vec![(header_end..first_location.start_offset(), replacement)];
    let parsed = ruby_prism::parse(context.source().as_bytes());
    if let Some(comment_start) = parsed
        .comments()
        .map(|comment| comment.location().start_offset())
        .find(|start| first_location.end_offset() <= *start && *start < line_end)
    {
        let comment_text = context.source()[comment_start..line_end].trim_end();
        edits.push((
            definition_start..definition_start,
            format!("{comment_text}\n{}", " ".repeat(indentation)),
        ));
        edits.push((comment_start..line_end, String::new()));
    }
    context.replace_many(message, &first_location, edits);
}

fn class_parts<'pr>(node: &ClassNode<'pr>) -> Option<(usize, Node<'pr>)> {
    let header_end = node.superclass().map_or_else(
        || node.constant_path().location().end_offset(),
        |superclass| superclass.location().end_offset(),
    );
    Some((header_end, node.body()?))
}

fn singleton_class_parts<'pr>(node: &SingletonClassNode<'pr>) -> Option<(usize, Node<'pr>)> {
    Some((node.expression().location().end_offset(), node.body()?))
}
