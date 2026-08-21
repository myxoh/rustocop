use super::*;

define_cops! {
    MultipleComparison => "Lint/MultipleComparison" => call(multiple_comparison),
    UnlessLogicalOperators => "Style/UnlessLogicalOperators" => node(as_unless_node, unless_logical_operators),
}

fn multiple_comparison(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !comparison_operator(call_name(node)) {
        return;
    }
    let (Some(inner), Some(right)) = (
        node.receiver().and_then(|receiver| receiver.as_call_node()),
        only_argument(node),
    ) else {
        return;
    };
    if !comparison_operator(call_name(&inner)) || argument_count(&inner) != 1 {
        return;
    }
    let (Some(left), Some(middle)) = (inner.receiver(), only_argument(&inner)) else {
        return;
    };
    if middle
        .as_call_node()
        .is_some_and(|call| matches!(call_name(&call), b"&" | b"|" | b"^"))
    {
        return;
    }
    let file = context.source_file();
    let replacement = format!(
        "{} {} {} && {} {} {}",
        file.node(&left),
        String::from_utf8_lossy(call_name(&inner)),
        file.node(&middle),
        file.node(&middle),
        String::from_utf8_lossy(call_name(node)),
        file.node(&right)
    );
    context.replace_call(
        node,
        "Use the `&&` operator to compare multiple values.",
        replacement,
    );
}

fn comparison_operator(name: &[u8]) -> bool {
    matches!(name, b"<" | b">" | b"<=" | b">=")
}

fn unless_logical_operators(node: &ruby_prism::UnlessNode<'_>, context: &mut CopContext<'_, '_>) {
    let mut operators = LogicalOperatorVisitor::default();
    operators.visit(&node.predicate());
    let style = context
        .policy()
        .enforced_style("forbid_mixed_logical_operators")
        .to_string();
    let predicate = node.predicate();
    let root_logical = predicate.as_and_node().is_some() || predicate.as_or_node().is_some();
    let message = if style == "forbid_logical_operators" && root_logical {
        "Do not use any logical operator in an `unless`."
    } else if style == "forbid_mixed_logical_operators" && operators.tokens.len() > 1 {
        "Do not use mixed logical operators in an `unless`."
    } else {
        return;
    };
    context.report(message, node.location());
}

#[derive(Default)]
struct LogicalOperatorVisitor {
    tokens: std::collections::HashSet<Vec<u8>>,
}

impl<'pr> Visit<'pr> for LogicalOperatorVisitor {
    fn visit_block_node(&mut self, _node: &ruby_prism::BlockNode<'pr>) {}

    fn visit_and_node(&mut self, node: &ruby_prism::AndNode<'pr>) {
        self.tokens.insert(node.operator_loc().as_slice().to_vec());
        ruby_prism::visit_and_node(self, node);
    }

    fn visit_or_node(&mut self, node: &ruby_prism::OrNode<'pr>) {
        self.tokens.insert(node.operator_loc().as_slice().to_vec());
        ruby_prism::visit_or_node(self, node);
    }
}
