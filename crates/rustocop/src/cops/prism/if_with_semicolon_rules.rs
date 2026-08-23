use ruby_prism::{IfNode, Node, UnlessNode};

use super::*;

define_cops! {
    IfWithSemicolon => "Style/IfWithSemicolon" => rubocop_callbacks(IfWithSemicolonRule, [on_if, on_unless]),
}

impl IfWithSemicolonRule<'_, '_, '_> {
    fn on_if(&mut self, node: &IfNode<'_>) {
        let boundary = node
            .statements()
            .map(|body| body.location().start_offset())
            .or_else(|| node.subsequent().map(|branch| branch.location().start_offset()))
            .unwrap_or_else(|| node.end_keyword_loc().map_or(node.location().end_offset(), |end| end.start_offset()));
        let Some(beginning) = semicolon_between(
            node.predicate().location().end_offset(),
            boundary,
            self.source(),
        ) else { return };
        return_if!(self.ancestors().iter().any(|parent| parent.as_if_node().is_some() || parent.as_unless_node().is_some()));
        let condition = self.source_file().node(&node.predicate());
        let branches = if_parts(node, self.source_file());
        let elsif = self.source_file().at(&node.location()).contains(" elsif ");
        self.register(node.location(), beginning, "if", condition, branches, elsif);
    }

    fn on_unless(&mut self, node: &UnlessNode<'_>) {
        let boundary = node
            .statements()
            .map(|body| body.location().start_offset())
            .or_else(|| node.else_clause().map(|branch| branch.location().start_offset()))
            .unwrap_or_else(|| node.end_keyword_loc().map_or(node.location().end_offset(), |end| end.start_offset()));
        let Some(beginning) = semicolon_between(
            node.predicate().location().end_offset(),
            boundary,
            self.source(),
        ) else { return };
        return_if!(self.ancestors().iter().any(|parent| parent.as_if_node().is_some() || parent.as_unless_node().is_some()));
        let condition = self.source_file().node(&node.predicate());
        let mut branches = unless_parts(node, self.source_file());
        std::mem::swap(&mut branches.0, &mut branches.1);
        self.register(node.location(), beginning, "unless", condition, branches, false);
    }

    fn register(&mut self, location: ruby_prism::Location<'_>, beginning: std::ops::Range<usize>, keyword: &str, condition: &str, branches: (Option<String>, Option<String>, bool), elsif: bool) {
        let (truthy, falsey, complex) = branches;
        let require_newline = complex || truthy.as_deref().is_some_and(complex_branch) || (!elsif && falsey.as_deref().is_some_and(complex_branch));
        let if_else = elsif || truthy.as_deref().is_some_and(block_or_assignment) || falsey.as_deref().is_some_and(block_or_assignment);
        let choice = if require_newline { "a newline" } else if if_else { "`if/else`" } else { "a ternary operator" };
        let message = format!("Do not use `{keyword} {condition};` - use {choice} instead.");
        let offense = location.start_offset()..location.end_offset();
        if require_newline || (if_else && !elsif) {
            add_offense!(self, offense, message: message, |corrector| { corrector.replace(beginning, "\n"); });
        } else if elsif {
            let replacement = multiline_elsif(self.source_file().at(&location));
            add_offense!(self, offense.clone(), message: message, |corrector| { corrector.replace(offense, replacement); });
        } else {
            let replacement = format!("{condition} ? {} : {}", truthy.as_deref().map(command_parentheses).unwrap_or_else(|| "nil".to_string()), falsey.as_deref().map(command_parentheses).unwrap_or_else(|| "nil".to_string()));
            add_offense!(self, offense.clone(), message: message, |corrector| { corrector.replace(offense, replacement); });
        }
    }
}

fn semicolon_between(start: usize, end: usize, source: &str) -> Option<std::ops::Range<usize>> {
    let between = source.get(start..end)?;
    let whitespace = between.len() - between.trim_start().len();
    (between.as_bytes().get(whitespace) == Some(&b';'))
        .then_some(start + whitespace..start + whitespace + 1)
}

fn if_parts(node: &IfNode<'_>, file: SourceFile<'_>) -> (Option<String>, Option<String>, bool) {
    let truthy = body_source(node.statements(), file);
    if let Some(elsif) = elsif_branch(node) {
        return (truthy, Some(file.node(&elsif.as_node()).to_string()), false);
    }
    let falsey = node.subsequent().and_then(|branch| branch.as_else_node()).and_then(|branch| body_source(branch.statements(), file));
    let complex = node.statements().is_some_and(|body| body.body().len() > 1)
        || node.subsequent().and_then(|branch| branch.as_else_node()).and_then(|branch| branch.statements()).is_some_and(|body| body.body().len() > 1);
    (truthy, falsey, complex)
}

fn elsif_branch<'pr>(node: &IfNode<'pr>) -> Option<IfNode<'pr>> {
    let branch = node.subsequent()?.as_else_node()?;
    let conditional = only_statement(branch.statements())?.as_if_node()?;
    conditional.if_keyword_loc().is_some_and(|keyword| keyword.as_slice() == b"elsif").then_some(conditional)
}

fn unless_parts(node: &UnlessNode<'_>, file: SourceFile<'_>) -> (Option<String>, Option<String>, bool) {
    let truthy = body_source(node.statements(), file);
    let falsey = node.else_clause().and_then(|branch| body_source(branch.statements(), file));
    let complex = node.statements().is_some_and(|body| body.body().len() > 1)
        || node.else_clause().and_then(|branch| branch.statements()).is_some_and(|body| body.body().len() > 1);
    (truthy, falsey, complex)
}

fn body_source(body: Option<ruby_prism::StatementsNode<'_>>, file: SourceFile<'_>) -> Option<String> {
    body.and_then(|body| (!body.body().is_empty()).then(|| file.at(&body.location()).trim().to_string()))
}

fn complex_branch(source: &str) -> bool { source.starts_with("return ") }
fn block_or_assignment(source: &str) -> bool { source.contains(" { ") || source.contains(", ") && source.contains(" = ") }

fn command_parentheses(source: &str) -> String {
    let source = source.trim();
    if [" + ", " - ", " * ", " / ", " % "].iter().any(|operator| source.contains(operator)) || source.contains('(') || source.starts_with('[') || source.contains(" = ") || source == "return" { return source.to_string(); }
    if let Some((method, argument)) = source.split_once(' ') {
        return format!("{method}({argument})");
    }
    source.to_string()
}

fn multiline_elsif(source: &str) -> String {
    let mut output = if source.contains("; ") { source.replacen("; ", "\n  ", 1) } else { source.replacen(';', "\n  ", 1) };
    while let Some(at) = output.find("; ") { output.replace_range(at..at + 2, "\n  "); }
    output = output.replace("\n  elsif ", "\n  \nelsif ");
    output = output.replace(" elsif ", "\nelsif ").replace(" else ", "\nelse\n  ");
    if output.ends_with("\n  end") {
        output.truncate(output.len() - 3);
        output.push_str("\nend");
    } else if output.ends_with(" end") {
        output.truncate(output.len() - 4);
        output.push_str("\nend");
    }
    output
}
