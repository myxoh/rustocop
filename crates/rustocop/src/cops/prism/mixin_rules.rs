use super::*;

define_cops! {
    MixinUsage => "Style/MixinUsage" => call(mixin_usage),
}

fn mixin_usage(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let method = call_name(node);
    if !matches!(method, b"include" | b"extend" | b"prepend") || node.receiver().is_some() {
        return;
    }
    if context.ancestors().iter().any(|ancestor| {
        ancestor.as_class_node().is_some()
            || ancestor.as_module_node().is_some()
            || ancestor.as_block_node().is_some()
            || ancestor.as_arguments_node().is_some()
            || ancestor.as_call_node().is_some()
            || ancestor.as_rescue_node().is_some()
            || ancestor
                .as_begin_node()
                .is_some_and(|begin| begin.rescue_clause().is_some())
    }) {
        return;
    }
    let method = String::from_utf8_lossy(method);
    context.report(
        format!("`{method}` is used at the top level. Use inside `class` or `module`."),
        node.location(),
    );
}
