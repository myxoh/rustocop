use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(BinaryOperatorWithIdenticalOperands),
        Box::new(HashCompareByIdentity),
        Box::new(RandOne),
    ]
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
        if source_at(source, &left.location()) != source_at(source, &right.location()) {
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

struct HashCompareByIdentity;

impl Cop for HashCompareByIdentity {
    fn name(&self) -> &'static str {
        "Lint/HashCompareByIdentity"
    }

    fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
        if !matches!(
            call_name(node),
            b"key?" | b"has_key?" | b"fetch" | b"[]" | b"[]="
        ) {
            return;
        }
        let Some(key_call) = first_argument(node).and_then(|argument| argument.as_call_node())
        else {
            return;
        };
        if call_name(&key_call) != b"object_id" || key_call.arguments().is_some() {
            return;
        }
        context.report(
            self.name(),
            "Use `Hash#compare_by_identity` instead of using `object_id` for keys.",
            node.location(),
        );
    }
}

struct RandOne;

impl Cop for RandOne {
    fn name(&self) -> &'static str {
        "Lint/RandOne"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        if call_name(&call) != b"rand"
            || !(call.receiver().is_none() || root_constant(call.receiver(), b"Kernel"))
            || call
                .arguments()
                .is_none_or(|arguments| arguments.arguments().len() != 1)
        {
            return;
        }
        let Some(argument) = first_argument(&call) else {
            return;
        };
        let one = argument
            .as_integer_node()
            .and_then(|integer| TryInto::<i32>::try_into(integer.value()).ok())
            .is_some_and(|value| value.abs() == 1)
            || argument
                .as_float_node()
                .is_some_and(|float| float.value().abs() == 1.0);
        if !one {
            return;
        }
        let location = call.location();
        let method = source_at(source, &location);
        context.report(
            self.name(),
            format!("`{method}` always returns `0`. Perhaps you meant `rand(2)` or `rand`?"),
            location,
        );
    }
}
