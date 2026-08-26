use super::*;

pub(super) fn variable_interpolation(
    node: &ruby_prism::EmbeddedVariableNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let variable = node.variable();
    let variable_source = context.source_file().node(&variable);
    context.replace(
        format!(
            "Replace interpolated variable `{variable_source}` with expression `#{{{variable_source}}}`."
        ),
        variable.location(),
        variable.location(),
        format!("{{{variable_source}}}"),
    );
}
