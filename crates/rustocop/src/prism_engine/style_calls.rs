use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![Box::new(ColonMethodCall)]
}

struct ColonMethodCall;

impl Cop for ColonMethodCall {
    fn name(&self) -> &'static str {
        "Style/ColonMethodCall"
    }

    fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
        let Some(operator) = node.call_operator_loc() else {
            return;
        };
        let method = call_name(node);
        if operator.as_slice() != b"::"
            || method.first().is_some_and(u8::is_ascii_uppercase)
            || root_constant(node.receiver(), b"Java")
        {
            return;
        }

        context.replace(
            self.name(),
            "Do not use `::` for method calls.",
            &operator,
            &operator,
            ".",
        );
    }
}
