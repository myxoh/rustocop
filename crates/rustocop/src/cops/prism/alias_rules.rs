use super::*;

define_cops! {
    Alias => "Style/Alias" => any_node(alias_style),
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
    if inside_instance_eval(context)
        || node.new_name().as_global_variable_read_node().is_some()
        || node.old_name().as_global_variable_read_node().is_some()
    {
        return;
    }
    let file = context.source_file();
    let new_name = file.node(&node.new_name());
    let old_name = file.node(&node.old_name());
    let interpolated = new_name.contains("#{") || old_name.contains("#{");
    let inside_instance_method = context.ancestors().iter().rev().any(|ancestor| {
        ancestor
            .as_def_node()
            .is_some_and(|definition| definition.receiver().is_none())
    });
    if style == "prefer_alias" && inside_instance_method {
        return;
    }
    let scope = alias_scope(context);

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
        || !matches!(alias_scope(context), AliasScope::Lexical(_))
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
    let AliasScope::Lexical(location) = alias_scope(context) else {
        return;
    };
    context.replace(
        format!("Use `alias` instead of `alias_method` {location}."),
        node.message_loc().expect("alias_method selector"),
        node.location(),
        format!(
            "alias {} {}",
            names[0].trim_start_matches(':'),
            names[1].trim_start_matches(':')
        ),
    );
}

#[derive(Clone, Copy)]
enum AliasScope {
    Lexical(&'static str),
    Dynamic,
}

fn alias_scope(context: &CopContext<'_, '_>) -> AliasScope {
    for ancestor in context.ancestors().iter().rev() {
        if ancestor.as_class_node().is_some() {
            return AliasScope::Lexical("in a class body");
        }
        if ancestor.as_module_node().is_some() {
            return AliasScope::Lexical("in a module body");
        }
        if ancestor.as_def_node().is_some() || ancestor.as_block_node().is_some() {
            return AliasScope::Dynamic;
        }
    }
    AliasScope::Lexical("at the top level")
}

fn as_symbol(name: &str) -> String {
    if name.starts_with(':') {
        name.to_string()
    } else {
        format!(":{name}")
    }
}

fn inside_instance_eval(context: &CopContext<'_, '_>) -> bool {
    context.ancestors().iter().any(|ancestor| {
        ancestor
            .as_call_node()
            .is_some_and(|call| call_name(&call) == b"instance_eval")
    })
}

fn assignment_node(node: &Node<'_>) -> bool {
    node.as_constant_write_node().is_some()
        || node.as_constant_path_write_node().is_some()
        || node.as_local_variable_write_node().is_some()
        || node.as_instance_variable_write_node().is_some()
        || node.as_class_variable_write_node().is_some()
        || node.as_global_variable_write_node().is_some()
}
