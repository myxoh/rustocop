use super::*;

define_cops! {
    EmptyLambdaParameter => "Style/EmptyLambdaParameter" => node(as_lambda_node, on_lambda),
}

fn on_lambda(node: &ruby_prism::LambdaNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(parameters) = node
        .parameters()
        .and_then(|parameters| parameters.as_block_parameters_node())
    else {
        return;
    };
    if parameters.parameters().is_some() || parameters.locals().iter().next().is_some() {
        return;
    }
    let (Some(opening), Some(closing)) = (parameters.opening_loc(), parameters.closing_loc()) else {
        return;
    };

    context.replace(
        "Omit parentheses for the empty lambda parameters.",
        opening.start_offset()..closing.end_offset(),
        node.operator_loc().end_offset()..closing.end_offset(),
        "",
    );
}
