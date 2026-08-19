use super::*;

define_cops! {
    RedundantRequireStatement => "Lint/RedundantRequireStatement" => call(redundant_require_statement),
}

fn redundant_require_statement(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) != b"require" || node.receiver().is_some() {
        return;
    }
    let Some(feature) = only_argument(node).and_then(|argument| static_string(&argument)) else {
        return;
    };
    if !redundant_feature(&feature, context.target_ruby_version()) {
        return;
    }
    let message = "Remove unnecessary `require` statement.";
    if let Some(modifier) = modifier_condition(node, context) {
        let file = context.source_file();
        let line = file.line_range(modifier.location().start_offset());
        let indentation = &context.source()[line.start..modifier.location().start_offset()];
        let predicate = file.node(&modifier.predicate());
        context.replace(
            message,
            node.location(),
            modifier.location(),
            format!("if {predicate}\n{indentation}end"),
        );
    } else {
        context.remove_statement(&node.as_node(), message);
    }
}

fn redundant_feature(feature: &[u8], version: RubyVersion) -> bool {
    feature == b"enumerator"
        || feature == b"thread" && version.at_least(2, 1)
        || matches!(feature, b"rational" | b"complex") && version.at_least(2, 2)
        || feature == b"ruby2_keywords" && version.at_least(2, 7)
        || feature == b"fiber" && version.at_least(3, 1)
        || feature == b"set" && version.at_least(3, 2)
        || feature == b"pathname" && version.at_least(4, 0)
}

fn modifier_condition<'pr>(
    node: &CallNode<'pr>,
    context: &CopContext<'_, 'pr>,
) -> Option<ruby_prism::IfNode<'pr>> {
    context.ancestors().iter().rev().find_map(|ancestor| {
        let condition = ancestor.as_if_node()?;
        condition
            .if_keyword_loc()
            .is_some_and(|keyword| keyword.start_offset() > node.location().start_offset())
            .then_some(condition)
    })
}
