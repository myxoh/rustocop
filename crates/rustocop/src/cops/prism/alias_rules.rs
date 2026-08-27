use super::*;

define_cops! {
    Alias => "Style/Alias" => compatibility_prism_any_node(alias_style),
}

fn alias_style(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let style = context.policy().enforced_style("prefer_alias").to_string();
    if let Some(alias) = node.as_alias_method_node() {
        inspect_alias_keyword(&alias, &style, context);
    } else if let Some(call) = node.as_call_node() {
        inspect_alias_method(&call, &style, context);
    }
}

fn inspect_alias_keyword(
    node: &ruby_prism::AliasMethodNode<'_>,
    style: &str,
    context: &mut CopContext<'_, '_>,
) {
    let scope = alias_scope(context);
    if matches!(scope, AliasScope::InstanceEval)
        || context.ancestors().iter().any(|ancestor| {
            ancestor.as_def_node().is_some_and(|definition| definition.receiver().is_none())
        })
        || node.new_name().as_global_variable_read_node().is_some()
        || node.old_name().as_global_variable_read_node().is_some()
    {
        return;
    }
    let file = context.source_file();
    let new_name = file.node(&node.new_name());
    let old_name = file.node(&node.old_name());
    let interpolated = new_name.contains("#{") || old_name.contains("#{");
    if style == "prefer_alias_method" || matches!(scope, AliasScope::Dynamic) {
        let replacement = format!(
            "alias_method {}, {}",
            as_symbol(new_name),
            as_symbol(old_name)
        );
        context.replace(
            "Use `alias_method` instead of `alias`.",
            node.keyword_loc(),
            node.location(),
            replacement,
        );
    } else if !interpolated && new_name.starts_with(':') && old_name.starts_with(':') {
        let arguments =
            node.new_name().location().start_offset()..node.old_name().location().end_offset();
        let replacement = format!(
            "{} {}",
            new_name.trim_start_matches(':'),
            old_name.trim_start_matches(':')
        );
        context.replace(
            format!("Use `alias {replacement}` instead of `alias {new_name} {old_name}`."),
            arguments.clone(),
            arguments,
            replacement,
        );
    }
}

fn inspect_alias_method(node: &CallNode<'_>, style: &str, context: &mut CopContext<'_, '_>) {
    if style != "prefer_alias"
        || call_name(node) != b"alias_method"
        || node.receiver().is_some()
        || matches!(alias_scope(context), AliasScope::Dynamic)
        || context.ancestors().iter().any(assignment_node)
    {
        return;
    }
    let arguments = node
        .arguments()
        .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
        .unwrap_or_default();
    if arguments.len() != 2 {
        return;
    }
    let file = context.source_file();
    let names = arguments
        .iter()
        .map(|argument| file.node(argument))
        .collect::<Vec<_>>();
    if names
        .iter()
        .any(|name| !name.starts_with(':') || name.contains("#{"))
    {
        return;
    }
    if context.ancestors().iter().any(|ancestor| {
        ancestor.as_call_node().is_some_and(|parent| {
            matches!(
                call_name(&parent),
                b"public" | b"private" | b"protected" | b"module_function"
            )
        })
    }) {
        return;
    }
    let scope = alias_scope(context);
    let location = lexical_scope_location(context);
    let (new_name, old_name) = if matches!(scope, AliasScope::InstanceEval) {
        (names[0], names[1])
    } else {
        (names[0].trim_start_matches(':'), names[1].trim_start_matches(':'))
    };
    context.replace(
        format!("Use `alias` instead of `alias_method` {location}."),
        node.message_loc().expect("alias_method selector"),
        node.location(),
        format!(
            "alias {} {}",
            new_name,
            old_name
        ),
    );
}

#[derive(Clone, Copy)]
enum AliasScope {
    Lexical,
    InstanceEval,
    Dynamic,
}

fn alias_scope(context: &CopContext<'_, '_>) -> AliasScope {
    for (index, ancestor) in context.ancestors().iter().enumerate().rev() {
        if ancestor.as_class_node().is_some() {
            return AliasScope::Lexical;
        }
        if ancestor.as_module_node().is_some() {
            return AliasScope::Lexical;
        }
        if ancestor.as_def_node().is_some() {
            return AliasScope::Dynamic;
        }
        if let Some(block) = ancestor.as_block_node() {
            let instance_eval = context.ancestors()[..index].iter().rev().find_map(|candidate| {
                let call = candidate.as_call_node()?;
                let owned = call.block().and_then(|owned| owned.as_block_node()).is_some_and(|owned| {
                    owned.location().start_offset() == block.location().start_offset()
                        && owned.location().end_offset() == block.location().end_offset()
                });
                owned.then_some(call)
            }).is_some_and(|call| call_name(&call) == b"instance_eval");
            return if instance_eval { AliasScope::InstanceEval } else { AliasScope::Dynamic };
        }
    }
    AliasScope::Lexical
}

fn lexical_scope_location(context: &CopContext<'_, '_>) -> &'static str {
    context.ancestors().iter().rev().find_map(|ancestor| {
        if ancestor.as_class_node().is_some() {
            Some("in a class body")
        } else if ancestor.as_module_node().is_some() {
            Some("in a module body")
        } else {
            None
        }
    }).unwrap_or("at the top level")
}

fn as_symbol(name: &str) -> String {
    if name.starts_with(':') {
        name.to_string()
    } else {
        format!(":{name}")
    }
}

fn assignment_node(node: &Node<'_>) -> bool {
    node.as_constant_write_node().is_some()
        || node.as_constant_path_write_node().is_some()
        || node.as_local_variable_write_node().is_some()
        || node.as_instance_variable_write_node().is_some()
        || node.as_class_variable_write_node().is_some()
        || node.as_global_variable_write_node().is_some()
}
