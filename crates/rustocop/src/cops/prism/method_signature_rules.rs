use super::*;

define_cops! {
    MultilineMethodSignature => "Style/MultilineMethodSignature" => node(as_def_node, multiline_method_signature),
}

fn multiline_method_signature(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let (Some(left), Some(right)) = (node.lparen_loc(), node.rparen_loc()) else {
        return;
    };
    let keyword = node.def_keyword_loc();
    let signature = &context.source()[keyword.start_offset()..right.end_offset()];
    if !signature.contains('\n') {
        return;
    }
    let identity_start = node.receiver().map_or_else(
        || node.name_loc().start_offset(),
        |receiver| receiver.location().start_offset(),
    );
    let identity = context.source()[identity_start..left.start_offset()].trim();
    let gap = &context.source()[keyword.end_offset()..identity_start];
    let parameters = context.source()[left.end_offset()..right.start_offset()]
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let replacement = if gap.contains('\n') {
        format!("def {identity}{gap}({parameters})")
    } else {
        format!("def {identity}({parameters})")
    };
    if context
        .related_config_value("Layout/LineLength", "Max")
        .and_then(|maximum| maximum.parse::<usize>().ok())
        .is_some_and(|maximum| replacement.lines().any(|line| line.len() > maximum))
    {
        return;
    }
    context.replace(
        "Avoid multi-line method signatures.",
        node.location(),
        keyword.start_offset()..right.end_offset(),
        replacement,
    );
}
