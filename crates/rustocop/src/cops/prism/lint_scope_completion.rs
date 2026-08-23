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
    let parent_source = parent_source.trim_start();
    let named_proc = ["lambda", "proc", "Proc.new", "::Proc.new"]
        .iter()
        .any(|name| {
            parent_source.strip_prefix(name).is_some_and(|rest| {
                rest.starts_with(|character: char| {
                    character.is_whitespace() || matches!(character, '(' | '{')
                })
            })
        });
    let lambda_or_proc = parent_source.starts_with("->") || named_proc;
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
    let end = location.end_offset();
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
    if contains_super(node) {
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
    let applies = if callback {
        context.ancestors().iter().rev().any(|ancestor| {
            ancestor.as_class_node().is_some()
                || ancestor.as_singleton_class_node().is_some()
                || ancestor.as_module_node().is_some()
        })
    } else if let Some(block) = context
        .ancestors()
        .iter()
        .rev()
        .find_map(Node::as_block_node)
    {
        class_new_call_for_block(&block, context).is_some_and(|call| {
            first_argument(&call)
                .is_some_and(|parent| !allowed_parent(node_source(context.source(), &parent)))
        })
    } else if let Some(class) = in_class {
        class
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

fn class_new_call_for_block<'pr>(
    block: &ruby_prism::BlockNode<'pr>,
    context: &CopContext<'_, 'pr>,
) -> Option<ruby_prism::CallNode<'pr>> {
    context.ancestors().iter().rev().find_map(|ancestor| {
        let call = ancestor.as_call_node()?;
        let call_block = call.block()?;
        let same_block = call_block.location().start_offset() == block.location().start_offset()
            && call_block.location().end_offset() == block.location().end_offset();
        (same_block && call_name(&call) == b"new" && root_constant(call.receiver(), b"Class"))
            .then_some(call)
    })
}

fn contains_super(node: &ruby_prism::DefNode<'_>) -> bool {
    struct SuperFinder(bool);

    impl<'pr> Visit<'pr> for SuperFinder {
        fn visit_super_node(&mut self, _node: &ruby_prism::SuperNode<'pr>) {
            self.0 = true;
        }

        fn visit_forwarding_super_node(&mut self, _node: &ruby_prism::ForwardingSuperNode<'pr>) {
            self.0 = true;
        }
    }

    let Some(body) = node.body() else {
        return false;
    };
    let mut finder = SuperFinder(false);
    finder.visit(&body);
    finder.0
}
