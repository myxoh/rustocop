use ruby_prism::{Node, StatementsNode};

use super::*;

define_rule!(OneLineConditionalRule);

define_cops! {
    OneLineConditional => "Style/OneLineConditional" => node_rule_aliases(
        OneLineConditionalRule,
        on_normal_if_unless => [as_if_node, as_unless_node]
    ),
}

impl OneLineConditionalRule<'_, '_, '_> {
    fn on_normal_if_unless(&mut self, node: &Node<'_>) {
        let Some(parts) = conditional_parts_for_one_line(node) else { return };
        return_if!(!self.source_file().same_line(parts.location.start, parts.location.end.saturating_sub(1)));
        return_if!(parts.elsif || parts.else_branch.is_empty() || parts.if_branch.len() >= 2
            || parts.if_branch.first().is_some_and(|branch| branch.as_begin_node().is_some()));
        let multiline = self.config_bool("AlwaysCorrectToMultiline", false)
            || parts.else_branch.len() >= 2 || parts.has_elsif;
        let keyword = if parts.unless { "unless" } else { "if" };
        let message = if multiline {
            format!("Favor multi-line `{keyword}` over single-line `{keyword}/then/else/end` constructs.")
        } else {
            format!("Favor the ternary operator (`?:`) over single-line `{keyword}/then/else/end` constructs.")
        };
        if self.ancestors().iter().rev().any(|ancestor| {
            (ancestor.as_if_node().is_some() || ancestor.as_unless_node().is_some())
                && self.source_file().same_line(ancestor.location().start_offset(), ancestor.location().end_offset().saturating_sub(1))
        }) {
            if !self.autocorrect_enabled() {
                self.report(message, parts.location);
            }
            return;
        }
        let replacement = if multiline {
            self.multiline_replacement(node, &parts)
        } else {
            self.ternary_replacement(&parts)
        };
        add_offense!(self, parts.location.clone(), message: message, |corrector| {
            corrector.replace(parts.location, replacement);
        });
    }

    fn ternary_replacement(&self, parts: &ConditionalParts<'_>) -> String {
        let condition = expression_replacement(Some(&parts.predicate), self.source_file());
        let if_branch = expression_replacement(parts.if_branch.first(), self.source_file());
        let else_branch = expression_replacement(parts.else_branch.first(), self.source_file());
        let (truthy, falsey) = if parts.unless { (else_branch, if_branch) } else { (if_branch, else_branch) };
        let replacement = format!("{condition} ? {truthy} : {falsey}");
        if self.ancestors().iter().rev().any(operator_parent) {
            format!("({replacement})")
        } else {
            replacement
        }
    }

    fn multiline_replacement(&self, node: &Node<'_>, parts: &ConditionalParts<'_>) -> String {
        let width = self
            .related_config_value("Layout/IndentationWidth", "Width")
            .and_then(|value| value.parse().ok())
            .unwrap_or(2);
        render_multiline_conditional(node, self.source_file(), parts.location.start, width)
    }
}

struct ConditionalParts<'pr> {
    location: std::ops::Range<usize>,
    predicate: Node<'pr>,
    if_branch: Vec<Node<'pr>>,
    else_branch: Vec<Node<'pr>>,
    unless: bool,
    elsif: bool,
    has_elsif: bool,
}

fn conditional_parts_for_one_line<'pr>(node: &Node<'pr>) -> Option<ConditionalParts<'pr>> {
    if let Some(condition) = node.as_if_node() {
        condition.if_keyword_loc()?;
        let subsequent = condition.subsequent()?;
        let has_elsif = subsequent.as_if_node().is_some();
        let elsif = condition.if_keyword_loc().is_some_and(|keyword| keyword.as_slice() == b"elsif");
        let else_branch = if let Some(branch) = subsequent.as_else_node() {
            statement_nodes(branch.statements())
        } else if let Some(branch) = subsequent.as_if_node() {
            vec![branch.as_node()]
        } else {
            return None;
        };
        Some(ConditionalParts {
            location: condition.location().start_offset()..condition.location().end_offset(),
            predicate: condition.predicate(),
            if_branch: statement_nodes(condition.statements()),
            else_branch,
            unless: false,
            elsif,
            has_elsif,
        })
    } else {
        let condition = node.as_unless_node()?;
        let branch = condition.else_clause()?;
        Some(ConditionalParts {
            location: condition.location().start_offset()..condition.location().end_offset(),
            predicate: condition.predicate(),
            if_branch: statement_nodes(condition.statements()),
            else_branch: statement_nodes(branch.statements()),
            unless: true,
            elsif: false,
            has_elsif: false,
        })
    }
}

fn statement_nodes<'pr>(statements: Option<StatementsNode<'pr>>) -> Vec<Node<'pr>> {
    statements.map(|statements| statements.body().iter().collect()).unwrap_or_default()
}

fn expression_replacement(node: Option<&Node<'_>>, file: SourceFile<'_>) -> String {
    let Some(node) = node else { return "nil".to_string() };
    let source = file.node(node);
    if requires_ternary_parentheses(node, source) { format!("({source})") } else { source.to_string() }
}

fn requires_ternary_parentheses(node: &Node<'_>, source: &str) -> bool {
    if node.as_and_node().is_some() || node.as_or_node().is_some() || node.as_if_node().is_some()
        || node.as_local_variable_write_node().is_some() || node.as_instance_variable_write_node().is_some()
        || node.as_class_variable_write_node().is_some() || node.as_global_variable_write_node().is_some()
    {
        return true;
    }
    if let Some(call) = node.as_call_node() {
        let has_arguments = call.arguments().is_some_and(|arguments| !arguments.arguments().is_empty());
        let operator = matches!(call.name().as_slice(), b"+" | b"-" | b"*" | b"/" | b"%" | b"**" | b"<" | b">" | b"<=" | b">=" | b"==" | b"===" | b"!=" | b"=~" | b"!~" | b"<=>" | b"<<" | b">>" | b"|" | b"&" | b"^" | b"~" | b"!");
        if has_arguments && call.opening_loc().is_none() && !operator { return true; }
    }
    (source.starts_with("not ") || source.starts_with("defined? ") || source.starts_with("yield ") || source.starts_with("super "))
        || source.contains(" and ") || source.contains(" or ")
}

fn operator_parent(node: &Node<'_>) -> bool {
    node.as_and_node().is_some() || node.as_or_node().is_some() || node.as_call_node().is_some_and(|call| {
        matches!(call.name().as_slice(), b"+" | b"-" | b"*" | b"/" | b"%" | b"**" | b"<" | b">" | b"<=" | b">=" | b"==" | b"===" | b"!=" | b"=~" | b"!~" | b"<=>" | b"<<" | b">>" | b"|" | b"&" | b"^" | b"~" | b"!")
    })
}

fn render_multiline_conditional(node: &Node<'_>, file: SourceFile<'_>, start: usize, width: usize) -> String {
    let base = " ".repeat(file.column(start));
    let body_indent = format!("{base}{}", " ".repeat(width));
    if let Some(condition) = node.as_if_node() {
        let mut rendered = render_if_chain(&condition, file, &base, &body_indent);
        rendered.push_str(&format!("\n{base}end"));
        rendered
    } else {
        let condition = node.as_unless_node().expect("conditional");
        let predicate = file.node(&condition.predicate());
        let body = render_statements(condition.statements(), file, &body_indent);
        let branch = condition.else_clause().expect("else checked");
        format!("unless {predicate}\n{body}\n{base}else\n{}\n{base}end", render_statements(branch.statements(), file, &body_indent))
    }
}

fn render_if_chain(condition: &ruby_prism::IfNode<'_>, file: SourceFile<'_>, base: &str, body_indent: &str) -> String {
    let keyword = String::from_utf8_lossy(condition.if_keyword_loc().expect("normal if").as_slice());
    let predicate = file.node(&condition.predicate());
    let body = render_statements(condition.statements(), file, body_indent);
    let mut rendered = format!("{keyword} {predicate}\n{body}");
    if let Some(subsequent) = condition.subsequent() {
        if let Some(elsif) = subsequent.as_if_node() {
            rendered.push('\n');
            rendered.push_str(&render_if_chain(&elsif, file, base, body_indent));
        } else if let Some(branch) = subsequent.as_else_node() {
            rendered.push_str(&format!("\n{base}else\n{}", render_statements(branch.statements(), file, body_indent)));
        }
    }
    rendered
}

fn render_statements(statements: Option<StatementsNode<'_>>, file: SourceFile<'_>, indent: &str) -> String {
    let values = statement_nodes(statements);
    if values.is_empty() { return format!("{indent}nil") }
    let source = values.iter().map(|node| file.node(node)).collect::<Vec<_>>().join("; ");
    format!("{indent}{source}")
}
