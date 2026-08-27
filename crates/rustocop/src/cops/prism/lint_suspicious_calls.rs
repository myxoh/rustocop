use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    Vec::new()
}

struct BinaryOperatorWithIdenticalOperands;

impl Cop for BinaryOperatorWithIdenticalOperands {
    fn name(&self) -> &'static str {
        "Lint/BinaryOperatorWithIdenticalOperands"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        if let Some(call) = node.as_call_node() {
            self.check_call(&call, source, context);
        } else if let Some(and_node) = node.as_and_node() {
            self.check_operands(
                node,
                and_node.left(),
                and_node.right(),
                and_node.operator_loc().as_slice(),
                source,
                context,
            );
        } else if let Some(or_node) = node.as_or_node() {
            self.check_operands(
                node,
                or_node.left(),
                or_node.right(),
                or_node.operator_loc().as_slice(),
                source,
                context,
            );
        }
    }
}

impl BinaryOperatorWithIdenticalOperands {
    fn check_call(&self, call: &CallNode<'_>, source: &str, context: &mut Context) {
        let operator = call_name(call);
        if !matches!(
            operator,
            b"==" | b"!=" | b"===" | b"<=>" | b"=~" | b">" | b">=" | b"<" | b"<=" | b"|" | b"^"
        ) {
            return;
        }
        let (Some(left), Some(right)) = (call.receiver(), first_argument(call)) else {
            return;
        };
        if call
            .arguments()
            .is_none_or(|arguments| arguments.arguments().len() != 1)
        {
            return;
        }
        self.check_operands(&call.as_node(), left, right, operator, source, context);
    }

    fn check_operands(
        &self,
        node: &Node<'_>,
        left: Node<'_>,
        right: Node<'_>,
        operator: &[u8],
        source: &str,
        context: &mut Context,
    ) {
        if !equivalent_operands(&left, &right, source) {
            return;
        }
        context.report(
            self.name(),
            format!(
                "Binary operator `{}` has identical operands.",
                String::from_utf8_lossy(operator)
            ),
            node.location(),
        );
    }
}

fn equivalent_operands(left: &Node<'_>, right: &Node<'_>, source: &str) -> bool {
    if same_source(source, left, right) {
        return true;
    }
    if let (Some(left), Some(right)) = (left.as_string_node(), right.as_string_node()) {
        return left.unescaped() == right.unescaped();
    }
    if let (Some(left), Some(right)) = (left.as_symbol_node(), right.as_symbol_node()) {
        return left.unescaped() == right.unescaped();
    }
    if let (Some(left), Some(right)) = (left.as_float_node(), right.as_float_node()) {
        return left.value() == right.value();
    }
    if let (Some(left), Some(right)) = (left.as_array_node(), right.as_array_node()) {
        return equivalent_lists(left.elements().iter(), right.elements().iter(), source);
    }
    if let (Some(left), Some(right)) = (left.as_hash_node(), right.as_hash_node()) {
        return equivalent_lists(left.elements().iter(), right.elements().iter(), source);
    }
    if let (Some(left), Some(right)) = (left.as_keyword_hash_node(), right.as_keyword_hash_node()) {
        return equivalent_lists(left.elements().iter(), right.elements().iter(), source);
    }
    if let (Some(left), Some(right)) = (left.as_assoc_node(), right.as_assoc_node()) {
        return equivalent_operands(&left.key(), &right.key(), source)
            && equivalent_operands(&left.value(), &right.value(), source);
    }
    left.as_true_node().is_some() && right.as_true_node().is_some()
        || left.as_false_node().is_some() && right.as_false_node().is_some()
        || left.as_nil_node().is_some() && right.as_nil_node().is_some()
}

fn equivalent_lists<'pr>(
    left: impl Iterator<Item = Node<'pr>>,
    right: impl Iterator<Item = Node<'pr>>,
    source: &str,
) -> bool {
    let left = left.collect::<Vec<_>>();
    let right = right.collect::<Vec<_>>();
    left.len() == right.len()
        && left
            .iter()
            .zip(&right)
            .all(|(left, right)| equivalent_operands(left, right, source))
}
