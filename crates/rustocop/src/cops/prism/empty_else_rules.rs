use super::*;

define_cops! {
    EmptyElse => "Style/EmptyElse" => compatibility_prism_any_node(empty_else),
}

fn empty_else(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (else_clause, kind) = if let Some(node) = node.as_if_node() {
        (node.subsequent().and_then(|node| node.as_else_node()), "if")
    } else if let Some(node) = node.as_unless_node() {
        (node.else_clause(), "if")
    } else if let Some(node) = node.as_case_node() {
        (node.else_clause(), "case")
    } else {
        return;
    };
    let Some(else_clause) = else_clause else {
        return;
    };
    if else_clause.else_keyword_loc().as_slice() != b"else" {
        return;
    }
    let statements = else_clause.statements();
    let empty = statements.is_none();
    let nil_only = statements.as_ref().is_some_and(|statements| {
        statements.body().len() == 1
            && statements
                .body()
                .first()
                .is_some_and(|node| node.as_nil_node().is_some())
    });
    let style = context.policy().enforced_style("empty");
    let redundant = match style {
        "nil" => nil_only,
        "both" => empty || nil_only,
        _ => empty,
    };
    if !redundant {
        return;
    }

    let keyword = else_clause.else_keyword_loc();
    let comments = ruby_prism::parse(context.source().as_bytes())
        .comments()
        .any(|comment| {
            keyword.end_offset() <= comment.location().start_offset()
                && comment.location().start_offset() < else_clause.location().end_offset()
        });
    if comments && context.config_bool("AllowComments", false) {
        return;
    }
    let missing_enabled =
        context.related_config_value("Style/MissingElse", "Enabled") != Some("false");
    let missing_style = context
        .related_config_value("Style/MissingElse", "EnforcedStyle")
        .unwrap_or_default();
    let conflicts = missing_enabled && (missing_style == "both" || missing_style == kind);
    if comments || conflicts {
        context.report("Redundant `else`-clause.", keyword);
        return;
    }

    let file = context.source_file();
    let location = else_clause.location();
    let end_start = context.source()[keyword.end_offset()..node.location().end_offset()]
        .rfind("end")
        .map_or(location.end_offset(), |relative| {
            keyword.end_offset() + relative
        });
    let multiline = context.source()[keyword.end_offset()..end_start].contains('\n')
        || context.source().as_bytes().get(keyword.end_offset()) == Some(&b'\n');
    let (start, end) = if multiline {
        (
            file.line_start(keyword.start_offset()),
            file.line_start(end_start),
        )
    } else {
        (keyword.start_offset(), end_start)
    };
    context.remove("Redundant `else`-clause.", keyword, start..end);
}
