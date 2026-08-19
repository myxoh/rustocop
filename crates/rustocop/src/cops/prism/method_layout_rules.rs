use ruby_prism::DefNode;

use super::*;

define_cops! {
    SingleLineMethods => "Style/SingleLineMethods" => node(as_def_node, single_line_methods),
    TrailingMethodEndStatement => "Style/TrailingMethodEndStatement" => node(as_def_node, trailing_method_end_statement),
}

fn trailing_method_end_statement(node: &DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(end_keyword) = node.end_keyword_loc() else {
        return;
    };
    let file = context.source_file();
    if file.same_line(node.location().start_offset(), end_keyword.start_offset()) {
        return;
    }
    let line_start = file.line_start(end_keyword.start_offset());
    if context.source()[line_start..end_keyword.start_offset()]
        .trim()
        .is_empty()
    {
        return;
    }
    let indentation = file.indentation(node.location().start_offset());
    let padding = &context.source()[indentation];
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

    let indentation = file.indentation(location.start_offset()).len();
    let padding = " ".repeat(indentation + 2);
    let end_padding = " ".repeat(indentation);
    let Some(end_keyword) = node.end_keyword_loc() else {
        return;
    };
    let end_start = end_keyword.start_offset();
    let header_end = statements
        .first()
        .map_or(end_start, |statement| statement.location().start_offset());
    let header = context.source()[location.start_offset()..header_end].trim_end();
    let mut replacement = header.to_string();

    for (index, statement) in statements.iter().enumerate() {
        let statement_location = statement.location();
        let following = statements
            .get(index + 1)
            .map_or(end_start, |next| next.location().start_offset());
        let separator = &context.source()[statement_location.end_offset()..following];
        let semicolon = if separator.contains(';') { ";" } else { "" };
        replacement.push('\n');
        replacement.push_str(&padding);
        replacement.push_str(file.node(statement));
        replacement.push_str(semicolon);
    }
    replacement.push('\n');
    replacement.push_str(&end_padding);
    replacement.push_str("end");

    context.replace(
        "Avoid single-line method definitions.",
        &location,
        &location,
        replacement,
    );
}
