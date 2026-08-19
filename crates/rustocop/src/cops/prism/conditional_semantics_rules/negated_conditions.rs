use super::*;

pub(super) fn negated_if(node: &ruby_prism::IfNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.subsequent().is_some() {
        return;
    }
    let Some(keyword) = node.if_keyword_loc() else {
        return;
    };
    if context.source_file().at(&keyword) != "if" {
        return;
    }
    check_negated_conditional(
        &node.location(),
        &keyword,
        &node.predicate(),
        "unless",
        "Favor `unless` over `if` for negative conditions.",
        context,
    );
}

pub(super) fn negated_unless(
    node: &ruby_prism::UnlessNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    if node.else_clause().is_some() {
        return;
    }
    let keyword = node.keyword_loc();
    check_negated_conditional(
        &node.location(),
        &keyword,
        &node.predicate(),
        "if",
        "Favor `if` over `unless` for negative conditions.",
        context,
    );
}

pub(super) fn negated_while(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(loop_node) = node.as_while_node() {
        check_negated_loop(
            loop_node.location(),
            loop_node.keyword_loc(),
            loop_node.predicate(),
            "until",
            "Favor `until` over `while` for negative conditions.",
            context,
        );
    } else if let Some(loop_node) = node.as_until_node() {
        check_negated_loop(
            loop_node.location(),
            loop_node.keyword_loc(),
            loop_node.predicate(),
            "while",
            "Favor `while` over `until` for negative conditions.",
            context,
        );
    }
}

fn check_negated_loop(
    location: ruby_prism::Location<'_>,
    keyword: ruby_prism::Location<'_>,
    predicate: Node<'_>,
    replacement_keyword: &str,
    message: &str,
    context: &mut CopContext<'_, '_>,
) {
    let predicate_location = predicate.location();
    let source = context.source_file().at(&predicate_location);
    if source.starts_with("!!") || source.contains(" && ") || source.contains(" or ") {
        return;
    }
    let relative = if source.starts_with("not ") {
        Some((0, 4))
    } else if source.starts_with('!') {
        Some((0, 1))
    } else if source.starts_with("(not ") {
        Some((1, 5))
    } else if source.starts_with("(!") {
        Some((1, 2))
    } else {
        source.rfind("; !").map(|at| (at + 2, at + 3))
    };
    let Some((start, end)) = relative else {
        return;
    };
    context.replace_many(
        message,
        &location,
        vec![
            (
                keyword.start_offset()..keyword.end_offset(),
                replacement_keyword.to_string(),
            ),
            (
                predicate_location.start_offset() + start..predicate_location.start_offset() + end,
                String::new(),
            ),
        ],
    );
}

fn check_negated_conditional(
    location: &ruby_prism::Location<'_>,
    keyword: &ruby_prism::Location<'_>,
    predicate: &Node<'_>,
    replacement_keyword: &str,
    message: &str,
    context: &mut CopContext<'_, '_>,
) {
    let modifier = keyword.start_offset() != location.start_offset();
    match context.policy().enforced_style("both") {
        "prefix" if modifier => return,
        "postfix" if !modifier => return,
        _ => {}
    }
    let predicate_location = predicate.location();
    let predicate_source = context.source_file().at(&predicate_location);
    if predicate_source.starts_with("!!")
        || predicate_source.contains(" && ")
        || predicate_source.contains(" or ")
    {
        return;
    }
    let negation = if predicate_source.starts_with("not ") {
        predicate_location.start_offset()..predicate_location.start_offset() + 4
    } else if predicate_source.starts_with('!') {
        predicate_location.start_offset()..predicate_location.start_offset() + 1
    } else if predicate_source.starts_with("(!") {
        predicate_location.start_offset() + 1..predicate_location.start_offset() + 2
    } else {
        return;
    };
    context.replace_many(
        message,
        location,
        vec![
            (
                keyword.start_offset()..keyword.end_offset(),
                replacement_keyword.to_string(),
            ),
            (negation, String::new()),
        ],
    );
}

pub(super) fn non_local_exit_from_iterator(
    node: &ReturnNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    if node.arguments().is_some() {
        return;
    }
    let ancestors = context.ancestors();
    for (index, ancestor) in ancestors.iter().enumerate().rev() {
        if ancestor.as_def_node().is_some() || ancestor.as_lambda_node().is_some() {
            return;
        }
        let Some(block) = ancestor.as_block_node() else {
            continue;
        };
        let call = index
            .checked_sub(1)
            .and_then(|parent| ancestors.get(parent))
            .and_then(Node::as_call_node);
        if call.as_ref().is_some_and(|call| {
            matches!(
                call.name().as_slice(),
                b"define_method" | b"define_singleton_method" | b"lambda"
            )
        }) {
            return;
        }
        if block.parameters().is_some() && call.is_some_and(|call| call.receiver().is_some()) {
            context.report(
                "Non-local exit from iterator, without return value. `next`, `break`, `Array#find`, `Array#any?`, etc. is preferred.",
                node.keyword_loc(),
            );
            return;
        }
    }
}
