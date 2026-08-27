use super::*;

define_cops! {
    UselessRuby2Keywords => "Lint/UselessRuby2Keywords" => compatibility_prism_call(useless_ruby2_keywords),
}

fn useless_ruby2_keywords(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) != b"ruby2_keywords" || node.receiver().is_some() {
        return;
    }
    let Some(argument) = only_argument(node) else {
        return;
    };
    if let Some(definition) = argument.as_def_node() {
        if unnecessary_parameters(definition.parameters()) {
            report(node, definition.name().as_slice(), true, context);
        }
        return;
    }
    let Some(name) = argument
        .as_symbol_node()
        .map(|symbol| symbol.unescaped().to_vec())
    else {
        return;
    };
    if scope_has_unnecessary_definition(&name, node.location().start_offset(), context.ancestors())
    {
        report(node, &name, false, context);
    }
}

fn report(node: &CallNode<'_>, name: &[u8], selector_only: bool, context: &mut CopContext<'_, '_>) {
    let offense = if selector_only {
        node.message_loc().unwrap_or_else(|| node.location())
    } else {
        node.location()
    };
    context.report(
        format!(
            "`ruby2_keywords` is unnecessary for method `{}`.",
            String::from_utf8_lossy(name)
        ),
        &offense,
    );
}

fn scope_has_unnecessary_definition(name: &[u8], before: usize, ancestors: &[Node<'_>]) -> bool {
    for ancestor in ancestors.iter().rev() {
        if let Some(class) = ancestor.as_class_node() {
            return class
                .body()
                .and_then(|body| body.as_statements_node())
                .is_some_and(|statements| definitions_include(&statements, name, before));
        }
        if let Some(module) = ancestor.as_module_node() {
            return module
                .body()
                .and_then(|body| body.as_statements_node())
                .is_some_and(|statements| definitions_include(&statements, name, before));
        }
        if let Some(singleton) = ancestor.as_singleton_class_node() {
            return singleton
                .body()
                .and_then(|body| body.as_statements_node())
                .is_some_and(|statements| definitions_include(&statements, name, before));
        }
        if let Some(program) = ancestor.as_program_node() {
            return definitions_include(&program.statements(), name, before);
        }
    }
    false
}

fn definitions_include(
    statements: &ruby_prism::StatementsNode<'_>,
    name: &[u8],
    before: usize,
) -> bool {
    statements.body().iter().any(|statement| {
        statement.location().start_offset() < before && unnecessary_definition(&statement, name)
    })
}

fn unnecessary_definition(node: &Node<'_>, name: &[u8]) -> bool {
    if let Some(definition) = node.as_def_node() {
        return definition.name().as_slice() == name
            && unnecessary_parameters(definition.parameters());
    }
    let Some(call) = node.as_call_node() else {
        return false;
    };
    if call_name(&call) != b"define_method"
        || first_argument(&call)
            .and_then(|argument| argument.as_symbol_node())
            .is_none_or(|symbol| symbol.unescaped() != name)
    {
        return false;
    }
    let parameters = call
        .block()
        .and_then(|block| block.as_block_node())
        .and_then(|block| block.parameters())
        .and_then(|parameters| parameters.as_block_parameters_node())
        .and_then(|parameters| parameters.parameters());
    unnecessary_parameters(parameters)
}

fn unnecessary_parameters(parameters: Option<ruby_prism::ParametersNode<'_>>) -> bool {
    parameters.is_none_or(|parameters| {
        parameters.rest().is_none()
            || !parameters.keywords().is_empty()
            || parameters.keyword_rest().is_some()
    })
}
