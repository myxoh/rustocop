use ruby_prism::{IfNode, LocalVariableWriteNode, Node};

use super::*;

define_rule!(RedundantSelfAssignmentBranchRule);

const MSG: &str = "Remove the self-assignment branch.";

define_cops! {
    RedundantSelfAssignmentBranch => "Style/RedundantSelfAssignmentBranch" => compatibility_prism_node_rule(as_local_variable_write_node, RedundantSelfAssignmentBranchRule, on_lvasgn),
}

impl RedundantSelfAssignmentBranchRule<'_, '_, '_> {
    fn on_lvasgn(&mut self, node: &LocalVariableWriteNode<'_>) {
        let Some(expression) = node.value().as_if_node() else {
            return;
        };
        let Some((if_branch, else_branch)) = branches(&expression) else {
            return;
        };
        return_if!(if_branch.as_ref().is_some_and(grouped_branch)
            || else_branch.as_ref().is_some_and(grouped_branch));
        return_if!(expression
            .subsequent()
            .is_some_and(|branch| branch.as_if_node().is_some()));

        let variable = String::from_utf8_lossy(node.name().as_slice());
        if self_assign(&variable, if_branch.as_ref(), self.source_file()) {
            self.register_offense(&expression, if_branch.as_ref(), else_branch.as_ref(), "unless");
        } else if self_assign(&variable, else_branch.as_ref(), self.source_file()) {
            self.register_offense(&expression, else_branch.as_ref(), if_branch.as_ref(), "if");
        }
    }

    fn register_offense(
        &mut self,
        if_node: &IfNode<'_>,
        offense_branch: Option<&Node<'_>>,
        opposite_branch: Option<&Node<'_>>,
        keyword: &str,
    ) {
        let Some(offense_branch) = offense_branch else {
            return;
        };
        let assignment_value = opposite_branch
            .map(|branch| self.source_of(branch))
            .unwrap_or("nil");
        let condition = self.source_of(&if_node.predicate());
        let mut replacement = format!("{assignment_value} {keyword} {condition}");
        if let Some(branch) = opposite_branch {
            replacement.push_str(self.heredoc_tail(branch));
        }
        add_offense!(self, offense_branch.location(), message: MSG, |corrector| {
            corrector.replace(if_node.location(), replacement);
        });
    }

    fn heredoc_tail<'a>(&'a self, node: &Node<'_>) -> &'a str {
        if !self.source_of(node).starts_with("<<") {
            return "";
        }
        let closing_end = node
            .as_string_node()
            .and_then(|string| string.closing_loc())
            .or_else(|| {
                node.as_interpolated_string_node()
                    .and_then(|string| string.closing_loc())
            })
            .map_or(node.location().end_offset(), |closing| closing.end_offset());
        let tail = self
            .source()
            .get(node.location().end_offset()..closing_end)
            .unwrap_or_default();
        tail.strip_suffix('\n').unwrap_or(tail)
    }
}

fn grouped_branch(node: &Node<'_>) -> bool {
    node.as_parentheses_node()
        .and_then(|parentheses| parentheses.body())
        .and_then(|body| body.as_statements_node())
        .is_some_and(|statements| !statements.body().is_empty())
}

fn branches<'pr>(node: &IfNode<'pr>) -> Option<(Option<Node<'pr>>, Option<Node<'pr>>)> {
    let if_branch = optional_single_statement(node.statements())?;
    let else_branch = match node.subsequent() {
        None => None,
        Some(subsequent) => {
            let else_node = subsequent.as_else_node()?;
            optional_single_statement(else_node.statements())?
        }
    };
    Some((if_branch, else_branch))
}

fn optional_single_statement<'pr>(
    statements: Option<ruby_prism::StatementsNode<'pr>>,
) -> Option<Option<Node<'pr>>> {
    let Some(statements) = statements else {
        return Some(None);
    };
    (statements.body().len() <= 1).then(|| statements.body().first())
}

fn self_assign(variable: &str, branch: Option<&Node<'_>>, file: SourceFile<'_>) -> bool {
    branch.is_some_and(|branch| file.node(branch) == variable)
}
