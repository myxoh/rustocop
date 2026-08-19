use super::*;

declare_cops!(BeginBlock, StringMethods);

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

define_call_cop!(StringMethods => "Style/StringMethods" => string_methods);

fn string_methods(node: &CallNode<'_>, reporter: &mut CopContext<'_, '_>) {
    if !match_call(node).without_arguments().matches() {
        return;
    }
    let Ok(method) = std::str::from_utf8(call_name(node)) else {
        return;
    };
    let preferred = reporter
        .config_map("PreferredMethods")
        .and_then(|methods| methods.get(method))
        .map(String::as_str)
        .or_else(|| (method == "intern").then_some("to_sym"))
        .map(str::to_string);
    let Some(preferred) = preferred else { return };
    reporter.replace_selector(
        node,
        format!("Prefer `{preferred}` over `{method}`."),
        &preferred,
    );
}
