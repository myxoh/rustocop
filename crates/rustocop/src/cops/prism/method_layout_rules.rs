use ruby_prism::DefNode;

use super::*;

define_cops! {
    SingleLineMethods => "Style/SingleLineMethods" => node(as_def_node, single_line_methods),
}

fn trailing_method_end_statement(node: &DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(end_keyword) = node.end_keyword_loc() else {
        return;
    };
    if node.body().is_none() {
        return;
    }
    let file = context.source_file();
    if file.same_line(node.location().start_offset(), end_keyword.start_offset()) {
        return;
    }
    let before_end = &context.source()
        [file.line_start(end_keyword.start_offset())..end_keyword.start_offset()];
    if before_end.trim().is_empty() {
        return;
    }
    let padding = " ".repeat(file.column(node.def_keyword_loc().start_offset()));
    context.insert(
        "Place the end statement of a multi-line method on its own line.",
        &end_keyword,
        end_keyword.start_offset(),
        format!("\n{padding}"),
    );
}

fn single_line_methods(node: &DefNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.equal_loc().is_some() {
        return;
    }
    let location = node.location();
    let file = context.source_file();
    if !file.same_line(
        location.start_offset(),
        location.end_offset().saturating_sub(1),
    ) {
        return;
    }

    let statements = node
        .body()
        .and_then(|body| body.as_statements_node())
        .map(|body| body.body().iter().collect::<Vec<_>>())
        .unwrap_or_default();
    if statements.is_empty() && context.config_bool("AllowIfMethodIsEmpty", true) {
        return;
    }

    if let Some(body) = endless_body(node, context) {
        let replacement = endless_method_source(node, &body, context);
        context.replace(
            "Avoid single-line method definitions.",
            &location,
            &location,
            replacement,
        );
        return;
    }

    let indentation = file.indentation(location.start_offset()).len();
    let width = context
        .related_config_value("Layout/IndentationWidth", "Width")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(2);
    let padding = " ".repeat(indentation + width);
    let end_padding = " ".repeat(indentation);
    let Some(end_keyword) = node.end_keyword_loc() else {
        return;
    };

    // LineBreakCorrector inserts before each syntax range instead of rebuilding
    // the definition. This intentionally retains semicolons and spaces that
    // precede those ranges (including RuboCop's trailing whitespace).
    let mut edits = statements
        .iter()
        .map(|statement| {
            (
                statement.location().start_offset()..statement.location().start_offset(),
                format!("\n{padding}"),
            )
        })
        .collect::<Vec<_>>();
    edits.push((
        end_keyword.start_offset()..end_keyword.start_offset(),
        format!("\n{end_padding}"),
    ));

    if let Some((comment_range, comment)) = trailing_comment(node, context.source()) {
        edits.push((comment_range, String::new()));
        edits.push((
            location.start_offset()..location.start_offset(),
            format!("{comment}\n{end_padding}"),
        ));
    }

    context.replace_many("Avoid single-line method definitions.", &location, edits);
}

fn endless_body<'pr>(
    node: &DefNode<'pr>,
    context: &CopContext<'_, 'pr>,
) -> Option<ruby_prism::Node<'pr>> {
    if !context.target_ruby_version().at_least(3, 0)
        || context.related_config_value("Style/EndlessMethod", "Enabled") == Some("false")
        || context.related_config_value("Style/EndlessMethod", "EnforcedStyle") == Some("disallow")
        || assignment_method_name(node.name().as_slice())
    {
        return None;
    }

    let statements = node.body()?.as_statements_node()?;
    if statements.body().len() != 1 {
        return None;
    }
    let body = statements.body().first()?;
    if body.as_if_node().is_some()
        || body.as_unless_node().is_some()
        || body.as_while_node().is_some()
        || body.as_until_node().is_some()
        || body.as_return_node().is_some()
        || body.as_break_node().is_some()
        || body.as_next_node().is_some()
    {
        return None;
    }
    Some(body)
}

fn assignment_method_name(name: &[u8]) -> bool {
    name.ends_with(b"=") && !matches!(name, b"==" | b"===" | b"!=" | b"<=" | b">=")
}

fn endless_method_source(
    node: &DefNode<'_>,
    body: &ruby_prism::Node<'_>,
    context: &CopContext<'_, '_>,
) -> String {
    let file = context.source_file();
    let receiver = node
        .receiver()
        .map(|receiver| format!("{}.", file.node(&receiver)))
        .unwrap_or_default();
    let name_loc = node.name_loc();
    let name = &context.source()[name_loc.start_offset()..name_loc.end_offset()];
    let arguments = match (node.lparen_loc(), node.rparen_loc(), node.parameters()) {
        (Some(left), Some(right), _) => {
            context.source()[left.start_offset()..right.end_offset()].to_string()
        }
        (_, _, Some(parameters)) => {
            let location = parameters.location();
            context.source()[location.start_offset()..location.end_offset()].to_string()
        }
        _ => "()".to_string(),
    };
    let body = endless_body_source(body, context);
    format!("def {receiver}{name}{arguments} = {body}")
}

fn endless_body_source(node: &ruby_prism::Node<'_>, context: &CopContext<'_, '_>) -> String {
    let Some(call) = node.as_call_node() else {
        return context.source_file().node(node).to_string();
    };
    let Some(arguments) = call.arguments() else {
        return context.source_file().node(node).to_string();
    };
    if arguments.arguments().is_empty()
        || matches!(
            call.name().as_slice(),
            b"+" | b"-"
                | b"*"
                | b"/"
                | b"%"
                | b"**"
                | b"|"
                | b"^"
                | b"&"
                | b"<<"
                | b">>"
                | b"<=>"
                | b"=="
                | b"!="
                | b"==="
                | b"=~"
                | b"!~"
                | b"<"
                | b">"
                | b"<="
                | b">="
        )
    {
        return context.source_file().node(node).to_string();
    }

    let receiver = call
        .receiver()
        .map(|receiver| format!("{}.", context.source_file().node(&receiver)))
        .unwrap_or_default();
    let method = String::from_utf8_lossy(call.name().as_slice());
    let arguments = arguments
        .arguments()
        .iter()
        .map(|argument| context.source_file().node(&argument).to_string())
        .collect::<Vec<_>>()
        .join(", ");
    format!("{receiver}{method}({arguments})")
}

fn trailing_comment(node: &DefNode<'_>, source: &str) -> Option<(std::ops::Range<usize>, String)> {
    let end = node.location().end_offset();
    let line_end = source[end..]
        .find('\n')
        .map_or(source.len(), |offset| end + offset);
    let tail = &source[end..line_end];
    let hash = tail.find('#')?;
    let start = end + hash;
    Some((start..line_end, source[start..line_end].to_string()))
}
