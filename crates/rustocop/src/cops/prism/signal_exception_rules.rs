use super::*;

define_cops! {
    SignalException => "Style/SignalException" => call(signal_exception),
}

fn signal_exception(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let method = call_name(node);
    if !matches!(method, b"raise" | b"fail")
        || node
            .receiver()
            .is_some_and(|receiver| !node_is_root_constant(&receiver, b"Kernel"))
    {
        return;
    }
    let style = context.policy().enforced_style("only_raise");
    if style == "only_raise" && method == b"fail" && custom_fail_defined(context.source()) {
        return;
    }
    let in_rescue = context
        .ancestors()
        .iter()
        .any(|ancestor| ancestor.as_rescue_node().is_some());
    let (replacement, message) = match style {
        "semantic" if method == b"raise" && !in_rescue => (
            "fail",
            "Use `fail` instead of `raise` to signal exceptions.",
        ),
        "semantic" if method == b"fail" && in_rescue => (
            "raise",
            "Use `raise` instead of `fail` to rethrow exceptions.",
        ),
        "only_raise" if method == b"fail" => ("raise", "Always use `raise` to signal exceptions."),
        "only_fail" if method == b"raise" => ("fail", "Always use `fail` to signal exceptions."),
        _ => return,
    };
    context.replace_selector(node, message, replacement);
}

fn custom_fail_defined(source: &str) -> bool {
    #[derive(Default)]
    struct FailDefinition {
        found: bool,
    }

    impl<'pr> ruby_prism::Visit<'pr> for FailDefinition {
        fn visit_def_node(&mut self, node: &ruby_prism::DefNode<'pr>) {
            if node.name().as_slice() == b"fail" {
                self.found = true;
            } else {
                ruby_prism::visit_def_node(self, node);
            }
        }
    }

    let parsed = ruby_prism::parse(source.as_bytes());
    let mut definition = FailDefinition::default();
    ruby_prism::Visit::visit(&mut definition, &parsed.node());
    definition.found
}
