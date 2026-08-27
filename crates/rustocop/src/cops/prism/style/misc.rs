use super::*;

declare_cops!(BeginBlock);

struct BeginBlock;

impl Cop for BeginBlock {
    fn name(&self) -> &'static str {
        "Style/BeginBlock"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        _source: &str,
        context: &mut Context,
    ) {
        let Some(pre_execution) = node.as_pre_execution_node() else {
            return;
        };
        context.report(
            self.name(),
            "Avoid the use of `BEGIN` blocks.",
            pre_execution.keyword_loc(),
        );
    }
}


fn string_methods(node: &CallNode<'_>, reporter: &mut CopContext<'_, '_>) {
    let Ok(method) = std::str::from_utf8(call_name(node)) else {
        return;
    };
    let configured = reporter.config_map("PreferredMethods");
    let reverses_default = configured.is_some_and(|methods| {
        methods
            .values()
            .any(|preferred| preferred.as_str() == "intern")
    });
    let preferred = configured
        .and_then(|methods| methods.get(method))
        .map(String::as_str)
        .or_else(|| (method == "intern" && !reverses_default).then_some("to_sym"))
        .map(str::to_string);
    let Some(preferred) = preferred else { return };
    reporter.replace_selector(
        node,
        format!("Prefer `{preferred}` over `{method}`."),
        &preferred,
    );
}
