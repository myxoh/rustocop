use ruby_prism::{ElseNode, InNode, WhenNode};

use super::*;

define_rule!(WhenThenRule);

const WHEN_THEN_MSG: &str =
    "Do not use `when {expression};`. Use `when {expression} then` instead.";

define_cops!(
    EmptyWhen => "Lint/EmptyWhen" => node(as_when_node, empty_when),
    ElseLayout => "Lint/ElseLayout" => node(as_else_node, else_layout),
    MultilineInPatternThen => "Style/MultilineInPatternThen" => node(as_in_node, multiline_in_pattern_then),
    MultilineIfModifier => "Style/MultilineIfModifier" => any_node(multiline_if_modifier),
    MultilineWhenThen => "Style/MultilineWhenThen" => node(as_when_node, multiline_when_then),
    WhenThen => "Style/WhenThen" => node_rule(as_when_node, WhenThenRule, on_when),
);

fn empty_when(node: &WhenNode<'_>, context: &mut CopContext<'_, '_>) {
    if node
        .statements()
        .is_some_and(|statements| !statements.body().is_empty())
    {
        return;
    }
    let Some(last_condition) = node.conditions().last() else {
        return;
    };
    if context.config_bool("AllowComments", false) {
        let tail = &context.source()[last_condition.location().end_offset()..];
        let mut has_comment = false;
        for (index, line) in tail.lines().enumerate() {
            let trimmed = line.trim_start();
            if index > 0
                && matches!(
                    trimmed.split_whitespace().next(),
                    Some("when" | "else" | "end")
                )
            {
                break;
            }
            if trimmed.starts_with('#') || line.contains('#') {
                has_comment = true;
            }
        }
        if has_comment {
            return;
        }
    }
    context.report(
        "Avoid `when` branches without a body.",
        node.keyword_loc().start_offset()..last_condition.location().end_offset(),
    );
}

fn else_layout(node: &ElseNode<'_>, context: &mut CopContext<'_, '_>) {
    if context.parent().is_none_or(|parent| {
        parent.as_if_node().is_none() && parent.as_unless_node().is_none()
    }) {
        return;
    }
    let Some(statements) = node.statements() else {
        return;
    };
    let Some(first) = statements.body().first() else {
        return;
    };
    let keyword = node.else_keyword_loc();
    if &context.source()[keyword.start_offset()..keyword.end_offset()] != "else" {
        return;
    }
    let first_location = first.location();
    let file = context.source_file();
    if !file.same_line(keyword.start_offset(), first_location.start_offset()) {
        return;
    }
    let single_line_then_form = context.ancestors().iter().rev().any(|ancestor| {
        ancestor.as_if_node().is_some_and(|conditional| {
            conditional.then_keyword_loc().is_some()
                && statements.body().len() == 1
                && file.same_line(keyword.start_offset(), first_location.end_offset())
        })
    });
    if single_line_then_form {
        return;
    }
    let indentation = file.indentation(keyword.start_offset()).len() + 2;
    context.replace(
        "Odd `else` layout detected. Did you mean to use `elsif`?",
        &first_location,
        keyword.end_offset()..first_location.start_offset(),
        format!("\n{}", " ".repeat(indentation)),
    );
}

fn multiline_in_pattern_then(node: &InNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(then_keyword) = node.then_loc() else {
        return;
    };
    check_multiline_then(
        &node.pattern(),
        node.statements().and_then(|body| body.body().first()),
        then_keyword,
        "Do not use `then` for multiline `in` statement.",
        context,
    );
}

fn multiline_if_modifier(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let parts = conditional_parts(node);
    let Some((keyword, body, predicate, name, location)) = parts else {
        return;
    };
    let nested_in_same_modifier = context.ancestors().iter().rev().any(|ancestor| {
        conditional_parts(ancestor).is_some_and(|(outer_keyword, _, _, _, outer_location)| {
            outer_keyword.start_offset() != outer_location.start_offset()
        })
    });
    let file = context.source_file();
    let body_is_multiline = body
        .as_call_node()
        .and_then(|call| call.block())
        .and_then(|block| block.as_block_node())
        .is_none_or(|block| {
            !file.same_line(
                block.opening_loc().start_offset(),
                block.closing_loc().end_offset(),
            )
        })
        && !file.same_line(body.location().start_offset(), body.location().end_offset());
    if keyword.start_offset() == location.start_offset()
        || nested_in_same_modifier
        || !body_is_multiline
    {
        return;
    }
    let base = context
        .source_file()
        .indentation(location.start_offset())
        .len();
    let rendered = render_conditional(node, base, file);
    let replacement = rendered
        .strip_prefix(&" ".repeat(base))
        .unwrap_or(&rendered);
    context.replace(
        format!("Favor a normal {name}-statement over a modifier clause in a multiline statement."),
        &location,
        &location,
        replacement,
    );
    let _ = predicate;
}

fn conditional_parts<'pr>(
    node: &Node<'pr>,
) -> Option<(
    ruby_prism::Location<'pr>,
    Node<'pr>,
    Node<'pr>,
    &'static str,
    ruby_prism::Location<'pr>,
)> {
    if let Some(conditional) = node.as_if_node() {
        Some((
            conditional.if_keyword_loc()?,
            conditional.statements()?.body().first()?,
            conditional.predicate(),
            "if",
            conditional.location(),
        ))
    } else {
        let conditional = node.as_unless_node()?;
        Some((
            conditional.keyword_loc(),
            conditional.statements()?.body().first()?,
            conditional.predicate(),
            "unless",
            conditional.location(),
        ))
    }
}

fn render_conditional(node: &Node<'_>, indentation: usize, file: SourceFile<'_>) -> String {
    if let Some((keyword, body, predicate, name, location)) = conditional_parts(node) {
        if keyword.start_offset() != location.start_offset() {
            return format!(
                "{}{name} {}\n{}\n{}end",
                " ".repeat(indentation),
                file.node(&predicate),
                render_conditional(&body, indentation + 2, file),
                " ".repeat(indentation),
            );
        }
    }
    indent_node_source(node, indentation, file)
}

fn indent_node_source(node: &Node<'_>, indentation: usize, file: SourceFile<'_>) -> String {
    let original_indent = file.indentation(node.location().start_offset()).len();
    file.node(node)
        .lines()
        .map(|line| {
            let removable = line
                .bytes()
                .take_while(|byte| *byte == b' ')
                .count()
                .min(original_indent);
            format!("{}{}", " ".repeat(indentation), &line[removable..])
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn multiline_when_then(node: &WhenNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(then_keyword) = node.then_keyword_loc() else {
        return;
    };
    let Some(last_condition) = node.conditions().last() else {
        return;
    };
    let Some(first_condition) = node.conditions().first() else {
        return;
    };
    if !context.source_file().same_line(
        first_condition.location().start_offset(),
        last_condition.location().end_offset(),
    ) {
        return;
    }
    check_multiline_then(
        &last_condition,
        node.statements().and_then(|body| body.body().first()),
        then_keyword,
        "Do not use `then` for multiline `when` statement.",
        context,
    );
}

impl WhenThenRule<'_, '_, '_> {
    fn on_when(&mut self, node: &WhenNode<'_>) {
        let Some(last_condition) = node.conditions().last() else {
            return;
        };
        let Some(statements) = node.statements() else {
            return;
        };
        let Some(first_statement) = statements.body().first() else {
            return;
        };
        let Some(last_statement) = statements.body().last() else {
            return;
        };
        let separator_gap = &self.source()
            [last_condition.location().end_offset()..first_statement.location().start_offset()];
        let Some(relative_separator) = separator_gap.find(';') else {
            return;
        };
        return_if!(
            node.then_keyword_loc().is_some()
                || !self.source_file().same_line(
                    node.keyword_loc().start_offset(),
                    last_statement.location().end_offset(),
                )
        );
        let separator_start = last_condition.location().end_offset() + relative_separator;
        let separator = separator_start..separator_start + 1;
        let expression = node
            .conditions()
            .iter()
            .map(|condition| self.source_of(&condition))
            .collect::<Vec<_>>()
            .join(", ");
        let message = WHEN_THEN_MSG.replace("{expression}", &expression);
        add_offense!(self, separator.clone(), message: message, |corrector| {
            corrector.replace(separator, " then");
        });
    }
}

fn check_multiline_then(
    header: &Node<'_>,
    first_statement: Option<Node<'_>>,
    then_keyword: ruby_prism::Location<'_>,
    message: &str,
    context: &mut CopContext<'_, '_>,
) {
    let file = context.source_file();
    let same_header_line =
        file.same_line(header.location().end_offset(), then_keyword.start_offset());
    let same_body_line = first_statement.is_some_and(|statement| {
        file.same_line(
            then_keyword.end_offset(),
            statement.location().start_offset(),
        )
    });
    if same_header_line && same_body_line {
        return;
    }
    let edit_start = then_keyword.start_offset().saturating_sub(1);
    let edit_end = then_keyword.end_offset()
        + usize::from(context.source().as_bytes().get(then_keyword.end_offset()) == Some(&b' '));
    context.remove(message, &then_keyword, edit_start..edit_end);
}
