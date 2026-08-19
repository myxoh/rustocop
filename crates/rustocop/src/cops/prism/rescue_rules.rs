use super::*;

define_cops! {
    SuppressedException => "Lint/SuppressedException" => any_node(suppressed_exception),
    UselessRescue => "Lint/UselessRescue" => node(as_rescue_node, useless_rescue),
}

fn suppressed_exception(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(rescue) = node.as_rescue_node() {
        suppressed_rescue(&rescue, context);
    } else if let Some(rescue) = node.as_rescue_modifier_node() {
        if rescue.rescue_expression().as_nil_node().is_some()
            && !context.config_bool("AllowNil", true)
        {
            context.report(
                "Do not suppress exceptions.",
                rescue.keyword_loc().start_offset()
                    ..rescue.rescue_expression().location().end_offset(),
            );
        }
    }
}

fn suppressed_rescue(node: &ruby_prism::RescueNode<'_>, context: &mut CopContext<'_, '_>) {
    let body = node
        .statements()
        .map(|statements| statements.body().iter().collect::<Vec<_>>())
        .unwrap_or_default();
    let only_nil = body.len() == 1 && body[0].as_nil_node().is_some();
    if !body.is_empty() && (!only_nil || context.config_bool("AllowNil", true)) {
        return;
    }
    if context.config_bool("AllowComments", true) && comments_follow_rescue(node, context) {
        return;
    }
    let keyword = node.keyword_loc();
    let offense = if body.is_empty() {
        let mut end = node.location().end_offset().max(keyword.end_offset());
        if context.source().as_bytes().get(end) == Some(&b';') {
            end += 1;
        }
        keyword.start_offset()..end
    } else {
        keyword.start_offset()..body[0].location().end_offset()
    };
    context.report("Do not suppress exceptions.", offense);
}

fn comments_follow_rescue(node: &ruby_prism::RescueNode<'_>, context: &CopContext<'_, '_>) -> bool {
    let start = node.keyword_loc().end_offset();
    let end = context
        .ancestors()
        .iter()
        .rev()
        .map(|ancestor| ancestor.location().end_offset())
        .find(|end| *end > start)
        .unwrap_or(context.source().len());
    context.source()[start..end]
        .lines()
        .take_while(|line| !matches!(line.trim(), "end" | "else" | "ensure"))
        .any(|line| line.trim_start().starts_with('#'))
}

fn useless_rescue(node: &ruby_prism::RescueNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.subsequent().is_some() {
        return;
    }
    let Some(statements) = node.statements() else {
        return;
    };
    let body = statements.body();
    if body.len() != 1 {
        return;
    }
    let Some(raise) = body.first().and_then(|statement| statement.as_call_node()) else {
        return;
    };
    if call_name(&raise) != b"raise" || raise.receiver().is_some() || raise.block().is_some() {
        return;
    }
    let reference_source = node
        .reference()
        .map(|reference| context.source_file().node(&reference));
    let reraises_current = match argument_count(&raise) {
        0 => true,
        1 => only_argument(&raise).is_some_and(|argument| {
            let argument = context.source_file().node(&argument);
            reference_source.map_or(matches!(argument, "$!" | "$ERROR_INFO"), |reference| {
                argument == reference
            })
        }),
        _ => false,
    };
    if !reraises_current {
        return;
    }
    if let Some(reference) = reference_source {
        let used_in_ensure = context.ancestors().iter().rev().any(|ancestor| {
            ancestor.as_begin_node().is_some_and(|begin| {
                begin.ensure_clause().is_some_and(|ensure_clause| {
                    context
                        .source_file()
                        .at(&ensure_clause.location())
                        .contains(reference)
                })
            })
        });
        if used_in_ensure {
            return;
        }
    }
    context.report("Useless `rescue` detected.", node.location());
}
