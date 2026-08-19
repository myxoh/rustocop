use ruby_prism::ReturnNode;

use super::*;

define_cops!(
    IfUnlessModifierOfIfUnless => "Style/IfUnlessModifierOfIfUnless" => any_node(if_unless_modifier_of_if_unless),
    NegatedIf => "Style/NegatedIf" => node(as_if_node, negated_if),
    NegatedUnless => "Style/NegatedUnless" => node(as_unless_node, negated_unless),
    NegatedWhile => "Style/NegatedWhile" => any_node(negated_while),
    NonNilCheck => "Style/NonNilCheck" => any_node(non_nil_check),
    NonLocalExitFromIterator => "Lint/NonLocalExitFromIterator" => node(as_return_node, non_local_exit_from_iterator),
);

fn non_nil_check(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let semantic = context.config_bool("IncludeSemanticChanges", false);
    let Some(call) = node.as_call_node() else {
        return;
    };

    if call_name(&call) == b"!" && argument_count(&call) == 0 {
        if !semantic {
            return;
        }
        let Some(nil_call) = call.receiver().and_then(|receiver| receiver.as_call_node()) else {
            return;
        };
        if call_name(&nil_call) != b"nil?" || argument_count(&nil_call) != 0 {
            return;
        }
        let replacement = nil_call.receiver().map_or_else(
            || "self".to_string(),
            |receiver| context.source_file().node(&receiver).to_string(),
        );
        context.replace_call(
            &call,
            "Explicit non-nil checks are usually redundant.",
            replacement,
        );
        return;
    }

    if call_name(&call) == b"nil?" && argument_count(&call) == 0 && semantic {
        let Some(unless_node) = context.parent().and_then(Node::as_unless_node) else {
            return;
        };
        if unless_node.end_keyword_loc().is_some() {
            return;
        }
        let Some(receiver) = call.receiver() else {
            return;
        };
        let Some(body) = unless_node
            .statements()
            .and_then(|statements| statements.body().first())
        else {
            return;
        };
        let replacement = format!(
            "{} if {}",
            context.source_file().node(&body),
            context.source_file().node(&receiver)
        );
        context.replace(
            "Explicit non-nil checks are usually redundant.",
            call.location(),
            unless_node.location(),
            replacement,
        );
        return;
    }

    if call_name(&call) != b"!=" {
        return;
    }
    let (Some(left), Some(right)) = (call.receiver(), only_argument(&call)) else {
        return;
    };
    let receiver = if right.as_nil_node().is_some() {
        left
    } else if left.as_nil_node().is_some() {
        right
    } else {
        return;
    };
    if !semantic
        && (context.related_config_value("Style/NilComparison", "EnforcedStyle")
            == Some("comparison")
            || final_predicate_expression(&call, context.ancestors()))
    {
        return;
    }
    let receiver_source = context.source_file().node(&receiver);
    let original = context.source_file().node(&call.as_node());
    if semantic {
        context.replace_call(
            &call,
            "Explicit non-nil checks are usually redundant.",
            receiver_source,
        );
    } else {
        let replacement = format!("!{receiver_source}.nil?");
        context.replace_call(
            &call,
            format!("Prefer `{replacement}` over `{original}`."),
            replacement,
        );
    }
}

fn final_predicate_expression(call: &CallNode<'_>, ancestors: &[Node<'_>]) -> bool {
    let Some(definition) = ancestors.iter().rev().find_map(Node::as_def_node) else {
        return false;
    };
    if !definition.name().as_slice().ends_with(b"?") {
        return false;
    }
    definition
        .body()
        .and_then(|body| body.as_statements_node())
        .and_then(|statements| statements.body().last())
        .is_some_and(|last| {
            last.location().start_offset() == call.location().start_offset()
                && last.location().end_offset() == call.location().end_offset()
        })
}

fn if_unless_modifier_of_if_unless(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let Some(modifier) = ModifierConditional::from_node(node) else {
        return;
    };
    if matches!(modifier.kind, ModifierKind::If | ModifierKind::Unless) {
        check_conditional_modifier(&modifier, context);
    }
}

fn check_conditional_modifier(
    modifier: &ModifierConditional<'_>,
    context: &mut CopContext<'_, '_>,
) {
    if !is_conditional(&modifier.body) && modifier.body.as_unless_node().is_none() {
        return;
    }
    let keyword_text = match modifier.kind {
        ModifierKind::If => "if",
        ModifierKind::Unless => "unless",
        ModifierKind::While | ModifierKind::Until => return,
    };
    let replacement = format!(
        "{keyword_text} {}\n{}\nend",
        context.source_file().node(&modifier.predicate),
        expand_modifier_conditionals(&modifier.body, context.source_file())
    );
    context.replace(
        format!("Avoid modifier `{keyword_text}` after another conditional."),
        &modifier.keyword,
        &modifier.location,
        replacement,
    );
}

fn is_conditional(node: &Node<'_>) -> bool {
    node.as_if_node().is_some() || node.as_unless_node().is_some()
}

fn expand_modifier_conditionals(node: &Node<'_>, file: SourceFile<'_>) -> String {
    if let Some(modifier) = ModifierConditional::from_node(node) {
        let keyword = match modifier.kind {
            ModifierKind::If => "if",
            ModifierKind::Unless => "unless",
            ModifierKind::While | ModifierKind::Until => return file.node(node).to_string(),
        };
        return format!(
            "{keyword} {}\n{}\nend",
            file.node(&modifier.predicate),
            expand_modifier_conditionals(&modifier.body, file)
        );
    }
    file.node(node).to_string()
}

fn negated_if(node: &ruby_prism::IfNode<'_>, context: &mut CopContext<'_, '_>) {
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
        "if",
        "unless",
        "Favor `unless` over `if` for negative conditions.",
        context,
    );
}

fn negated_unless(node: &ruby_prism::UnlessNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.else_clause().is_some() {
        return;
    }
    let keyword = node.keyword_loc();
    check_negated_conditional(
        &node.location(),
        &keyword,
        &node.predicate(),
        "unless",
        "if",
        "Favor `if` over `unless` for negative conditions.",
        context,
    );
}

fn negated_while(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
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
    from: &str,
    to: &str,
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
            (keyword.start_offset()..keyword.end_offset(), to.to_string()),
            (negation, String::new()),
        ],
    );
    let _ = from;
}

fn non_local_exit_from_iterator(node: &ReturnNode<'_>, context: &mut CopContext<'_, '_>) {
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
