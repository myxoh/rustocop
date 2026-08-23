use ruby_prism::{CallNode, Node};

use super::*;

define_cops! {
    RequireParentheses => "Lint/RequireParentheses" => call(require_parentheses),
    FileOpen => "Style/FileOpen" => call(file_open),
    KeywordArgumentsMerging => "Style/KeywordArgumentsMerging" => call(keyword_arguments_merging),
    MethodCalledOnDoEndBlock => "Style/MethodCalledOnDoEndBlock" => call(method_called_on_do_end_block),
}

fn require_parentheses(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.opening_loc().is_some() || argument_count(node) == 0 {
        return;
    }

    let Some(arguments) = node.arguments() else {
        return;
    };
    let arguments = arguments.arguments();
    let Some(first_argument) = arguments.iter().next() else {
        return;
    };
    let message = "Use parentheses in the method call to avoid confusion about precedence.";

    if let Some(conditional) = first_argument.as_if_node().filter(is_ternary) {
        if matches!(call_name(node), b"[]" | b"[]=") || assignment_method(node) {
            return;
        }
        let predicate = conditional.predicate();
        if is_boolean_operator(&predicate) {
            context.report(
                message,
                node.location().start_offset()..predicate.location().end_offset(),
            );
        }
        return;
    }

    if call_name(node).ends_with(b"?")
        && arguments
            .iter()
            .last()
            .is_some_and(|argument| is_boolean_operator(&argument))
    {
        context.report_call(node, message);
    }
}

fn is_ternary(node: &ruby_prism::IfNode<'_>) -> bool {
    node.if_keyword_loc().is_none()
        && node.then_keyword_loc().is_some()
        && node.end_keyword_loc().is_none()
}

fn assignment_method(node: &CallNode<'_>) -> bool {
    let name = call_name(node);
    name.ends_with(b"=") && !matches!(name, b"==" | b"===" | b"!=" | b"<=" | b">=")
}

fn is_boolean_operator(node: &Node<'_>) -> bool {
    node.as_and_node().is_some() || node.as_or_node().is_some()
}

fn file_open(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !match_call(node)
        .named(b"open")
        .on_root_constant(b"File")
        .with_operator(b".")
        .without_block()
        .matches()
    {
        return;
    }

    let used_unsafely = context.parent().is_some_and(|parent| {
        parent.as_local_variable_write_node().is_some()
            || parent
                .as_call_node()
                .and_then(|call| call.receiver())
                .is_some_and(|receiver| {
                    receiver.location().start_offset() == node.location().start_offset()
                        && receiver.location().end_offset() == node.location().end_offset()
                })
    });
    let discarded_statement = context
        .parent()
        .and_then(|parent| parent.as_statements_node())
        .is_some_and(|statements| {
            statements.body().iter().last().is_some_and(|last| {
                last.location().start_offset() != node.location().start_offset()
                    || last.location().end_offset() != node.location().end_offset()
            })
        });
    let discarded_at_top_level = !context.inside_method()
        && context.ancestors().iter().all(|ancestor| {
            ancestor.as_program_node().is_some() || ancestor.as_statements_node().is_some()
        });

    if used_unsafely || discarded_statement || discarded_at_top_level {
        context.report_call(
            node,
            "`File.open` without a block may leak a file descriptor; use the block form.",
        );
    }
}

fn keyword_arguments_merging(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let keyword_hash = context
        .ancestors()
        .iter()
        .rev()
        .nth(1)
        .and_then(Node::as_keyword_hash_node);
    let splat_is_first = keyword_hash.is_some_and(|hash| {
        hash.elements().iter().next().is_some_and(|element| {
            element.as_assoc_splat_node().is_some_and(|splat| {
                context.parent().is_some_and(|parent| {
                    splat.location().start_offset() == parent.location().start_offset()
                })
            })
        })
    });
    let outer_call = context
        .ancestors()
        .iter()
        .rev()
        .nth(2)
        .and_then(Node::as_call_node);

    if call_name(node) != b"merge"
        || node.arguments().is_none()
        || context
            .parent()
            .is_none_or(|parent| parent.as_assoc_splat_node().is_none())
        || !splat_is_first
        || outer_call.is_none()
        || outer_call.is_some_and(|call| {
            call.block()
                .is_some_and(|block| block.as_block_argument_node().is_some())
        })
    {
        return;
    }

    let Some(replacement) = flattened_keyword_merge(node, context) else {
        return;
    };
    context.replace_call(
        node,
        "Provide additional arguments directly rather than using `merge`.",
        replacement,
    );
}

fn flattened_keyword_merge(node: &CallNode<'_>, context: &CopContext<'_, '_>) -> Option<String> {
    let receiver = node.receiver()?;
    let mut pieces = if let Some(call) = receiver
        .as_call_node()
        .filter(|call| call_name(call) == b"merge" && call.arguments().is_some())
    {
        vec![flattened_keyword_merge(&call, context)?]
    } else {
        vec![context.source_file().node(&receiver).to_string()]
    };

    for argument in node.arguments()?.arguments().iter() {
        if argument.as_keyword_hash_node().is_some() {
            pieces.push(context.source_file().node(&argument).to_string());
        } else if let Some(hash) = argument.as_hash_node() {
            let opening = hash.opening_loc();
            let closing = hash.closing_loc();
            pieces.push(
                context
                    .source_file()
                    .slice(opening.end_offset()..closing.start_offset())?
                    .to_string(),
            );
        } else {
            pieces.push(format!("**{}", context.source_file().node(&argument)));
        }
    }
    Some(pieces.join(", "))
}

fn method_called_on_do_end_block(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node
        .block()
        .and_then(|block| block.as_block_node())
        .is_some()
    {
        return;
    }
    let Some(receiver) = node.receiver() else {
        return;
    };
    let closing = if let Some(block) = receiver
        .as_call_node()
        .and_then(|receiver| receiver.block())
        .and_then(|block| block.as_block_node())
    {
        block.closing_loc()
    } else if let Some(lambda) = receiver.as_lambda_node() {
        lambda.closing_loc()
    } else {
        return;
    };
    if closing.as_slice() != b"end" {
        return;
    }
    context.report(
        "Avoid chaining a method call on a do...end block.",
        closing.start_offset()..node.location().end_offset(),
    );
}
