use ruby_prism::ReturnNode;

use super::*;

mod negated_conditions;
use negated_conditions::*;

define_cops!(
    NegatedIf => "Style/NegatedIf" => node(as_if_node, negated_if),
    NegatedUnless => "Style/NegatedUnless" => node(as_unless_node, negated_unless),
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
