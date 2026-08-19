use super::*;

declare_cops!(EmptyBlock, MissingSuper);
define_any_node_cop!(EmptyBlock => "Lint/EmptyBlock" => empty_node);
define_node_cop!(MissingSuper => "Lint/MissingSuper" => as_def_node => missing_super);

fn empty_node(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(lambda) = node.as_lambda_node() {
        let empty = lambda.body().is_none_or(|body| {
            body.as_statements_node()
                .is_some_and(|statements| statements.body().is_empty())
        });
        if empty && !context.config_bool("AllowEmptyLambdas", true) {
            context.report("Empty block detected.", lambda.location());
        }
        return;
    }
    let Some(node) = node.as_block_node() else {
        return;
    };
    if node.body().is_some_and(|body| {
        body.as_statements_node()
            .is_none_or(|statements| !statements.body().is_empty())
    }) {
        return;
    }
    let parent = context.parent();
    let parent_source = parent
        .map(|parent| context.source_file().at(&parent.location()))
        .unwrap_or_default();
    let lambda_or_proc = parent_source.trim_start().starts_with("->")
        || parent_source.trim_start().starts_with("lambda")
        || parent_source.trim_start().starts_with("proc")
        || parent_source.trim_start().starts_with("Proc.new")
        || parent_source.trim_start().starts_with("::Proc.new");
    if lambda_or_proc && context.config_bool("AllowEmptyLambdas", true) {
        return;
    }
    let location = parent.map_or_else(|| node.location(), Node::location);
    let line_start = context.source_file().line_start(location.start_offset());
    let line_end = context.source()[line_start..]
        .find('\n')
        .map_or(context.source().len(), |at| line_start + at);
    let line = &context.source()[line_start..line_end];
    let block_source = context.source_file().at(&node.location());
    let inline_comment = node.location().end_offset() <= line_end
        && line[node.location().end_offset().saturating_sub(line_start)..].contains('#');
    if context.config_bool("AllowComments", true) && (block_source.contains('#') || inline_comment)
    {
        return;
    }
    let start = location.start_offset();
    let end = if lambda_or_proc && block_source.contains('\n')
        || block_source.contains('#') && !context.config_bool("AllowComments", true)
    {
        location.end_offset()
    } else if block_source.contains('\n') {
        line_end
    } else {
        node.location().end_offset()
    };
    context.report("Empty block detected.", start..end);
}

fn missing_super(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let name = node.name().as_slice();
    let callback = matches!(
        name,
        b"inherited"
            | b"method_added"
            | b"method_removed"
            | b"method_undefined"
            | b"singleton_method_added"
            | b"singleton_method_removed"
            | b"singleton_method_undefined"
    );
    if name != b"initialize" && !callback {
        return;
    }
    let method_source = context.source_file().at(&node.location());
    if method_source
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .any(|word| word == "super")
    {
        return;
    }
    let allowed_parent = |parent: &str| {
        matches!(parent.trim_start_matches("::"), "Object" | "BasicObject")
            || context
                .config_values("AllowedParentClasses")
                .iter()
                .any(|allowed| allowed == parent.trim_start_matches("::"))
    };
    let in_class = context
        .ancestors()
        .iter()
        .rev()
        .find_map(Node::as_class_node);
    let class_new = context.ancestors().iter().rev().find_map(|ancestor| {
        let call = ancestor.as_call_node()?;
        (call_name(&call) == b"new" && root_constant(call.receiver(), b"Class")).then_some(call)
    });
    let applies = if callback {
        in_class.is_some()
    } else if let Some(call) = class_new {
        first_argument(&call)
            .is_some_and(|parent| !allowed_parent(node_source(context.source(), &parent)))
    } else if let Some(class) = in_class {
        let nested_block = context
            .ancestors()
            .iter()
            .rev()
            .take_while(|ancestor| ancestor.as_class_node().is_none())
            .any(|ancestor| ancestor.as_block_node().is_some());
        !nested_block
            && class
                .superclass()
                .is_some_and(|parent| !allowed_parent(node_source(context.source(), &parent)))
    } else {
        false
    };
    if !applies {
        return;
    }
    let message = if name == b"initialize" {
        "Call `super` to initialize state of the parent class."
    } else {
        "Call `super` to invoke callback defined in the parent class."
    };
    context.report(message, node.location());
}
