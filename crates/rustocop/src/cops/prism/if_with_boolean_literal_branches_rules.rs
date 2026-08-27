use ruby_prism::{IfNode, Node, UnlessNode};

use super::*;

define_cops! {
    IfWithBooleanLiteralBranches => "Style/IfWithBooleanLiteralBranches" => compatibility_prism_callbacks(
        IfWithBooleanLiteralBranchesRule,
        [on_if, on_unless]
    ),
}

impl IfWithBooleanLiteralBranchesRule<'_, '_, '_> {
    fn on_if(&mut self, node: &IfNode<'_>) {
        let Some((truthy, falsey)) = if_branches(node) else { return };
        let condition = node.predicate();
        return_unless!(self.boolean_condition(&condition));
        let Some(opposite) = boolean_branches(&truthy, &falsey) else { return };
        let keyword = node.if_keyword_loc();
        let ternary = keyword.is_none();
        let elsif = keyword.as_ref().is_some_and(|keyword| keyword.as_slice() == b"elsif");
        return_if!(elsif && self.parent().and_then(Node::as_if_node).is_some_and(|parent| {
            parent.if_keyword_loc().is_some_and(|keyword| keyword.as_slice() == b"elsif")
        }));
        let condition_source = self.source_file().node(&condition);
        let replacement = replacement_condition(condition_source, opposite, &condition);
        let offense = if ternary {
            condition.location().end_offset()..node.location().end_offset()
        } else {
            let keyword = keyword.expect("block conditional");
            keyword.start_offset()..keyword.end_offset()
        };
        let message = if elsif {
            "Use `else` instead of redundant `elsif` with boolean literal branches.".to_string()
        } else if ternary {
            "Remove redundant ternary operator with boolean literal branches.".to_string()
        } else {
            "Remove redundant `if` with boolean literal branches.".to_string()
        };
        if elsif {
            let base = " ".repeat(self.source_file().column(node.location().start_offset()));
            let indent = format!("{base}  ");
            let edit = node.location();
            add_offense!(self, offense, message: message, |corrector| {
                corrector.replace(edit, format!("else\n{indent}{replacement}\n{base}end"));
            });
        } else {
            add_offense!(self, offense, message: message, |corrector| {
                corrector.replace(node.location(), replacement);
            });
        }
    }

    fn on_unless(&mut self, node: &UnlessNode<'_>) {
        let Some((truthy, falsey)) = unless_branches(node) else { return };
        let condition = node.predicate();
        return_unless!(self.boolean_condition(&condition));
        let Some(branches_opposite) = boolean_branches(&truthy, &falsey) else { return };
        let opposite = !branches_opposite;
        let condition_source = self.source_file().node(&condition);
        let replacement = replacement_condition(condition_source, opposite, &condition);
        let keyword = node.keyword_loc();
        let offense = keyword.start_offset()..keyword.end_offset();
        add_offense!(self, offense, message: "Remove redundant `unless` with boolean literal branches.", |corrector| {
            corrector.replace(node.location(), replacement);
        });
    }

    fn boolean_condition(&self, condition: &Node<'_>) -> bool {
        if let Some(parentheses) = condition.as_parentheses_node() {
            return parentheses.body().and_then(single_expression).is_some_and(|body| self.boolean_condition(&body));
        }
        if let Some(or) = condition.as_or_node() {
            return self.boolean_condition(&or.left()) && self.boolean_condition(&or.right());
        }
        if let Some(and) = condition.as_and_node() {
            return self.boolean_condition(&and.right());
        }
        let Some(call) = condition.as_call_node() else { return false };
        return_if!(
            call.call_operator_loc()
                .is_some_and(|operator| operator.as_slice() == b"&."),
            false
        );
        let method = call.name().as_slice();
        return_if!(self.config_values("AllowedMethods").iter().any(|allowed| allowed.as_bytes() == method), false);
        matches!(method, b"==" | b"!=" | b"===" | b"<" | b"<=" | b">" | b">=" | b"eql?" | b"equal?")
            || method.ends_with(b"?")
            || (method == b"!" && call.receiver().and_then(|receiver| receiver.as_call_node()).is_some_and(|inner| inner.name().as_slice() == b"!"))
    }
}

fn if_branches<'pr>(node: &IfNode<'pr>) -> Option<(Node<'pr>, Node<'pr>)> {
    let truthy = only_statement(node.statements())?;
    let falsey = node.subsequent()?.as_else_node().and_then(|branch| only_statement(branch.statements()))?;
    Some((truthy, falsey))
}

fn unless_branches<'pr>(node: &UnlessNode<'pr>) -> Option<(Node<'pr>, Node<'pr>)> {
    let truthy = only_statement(node.statements())?;
    let falsey = only_statement(node.else_clause()?.statements())?;
    Some((truthy, falsey))
}

fn boolean_branches(truthy: &Node<'_>, falsey: &Node<'_>) -> Option<bool> {
    if truthy.as_true_node().is_some() && falsey.as_false_node().is_some() {
        Some(false)
    } else if truthy.as_false_node().is_some() && falsey.as_true_node().is_some() {
        Some(true)
    } else {
        None
    }
}

fn replacement_condition(source: &str, opposite: bool, condition: &Node<'_>) -> String {
    if !opposite {
        return source.to_string();
    }
    let parenthesize = source.contains(" && ")
        || source.contains(" || ")
        || [" == ", " != ", " === ", " < ", " <= ", " > ", " >= ", " =~ ", " !~ "]
            .iter()
            .any(|operator| source.contains(operator));
    if condition.as_parentheses_node().is_some() || !parenthesize {
        format!("!{source}")
    } else {
        format!("!({source})")
    }
}
