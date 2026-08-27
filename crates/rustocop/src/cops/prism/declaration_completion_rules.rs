use ruby_prism::{CallNode, ClassNode, DefNode, Node, ParametersNode, StatementsNode};

use super::*;

define_cops! {
    Attr => "Style/Attr" => call(attr),
    DataInheritance => "Style/DataInheritance" => node(as_class_node, data_inheritance),
    RedundantInitialize => "Style/RedundantInitialize" => node(as_def_node, redundant_initialize),
    RedundantStructKeywordInit => "Style/RedundantStructKeywordInit" => call(redundant_struct_keyword_init),
}

fn data_inheritance(node: &ClassNode<'_>, context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(3, 2) {
        return;
    }
    class_factory_inheritance(
        node,
        b"Data",
        "Don't extend an instance initialized by `Data.define`. Use a block to customize the class.",
        context,
    );
}

fn struct_inheritance(node: &ClassNode<'_>, context: &mut CopContext<'_, '_>) {
    class_factory_inheritance(
        node,
        b"Struct",
        "Don't extend an instance initialized by `Struct.new`. Use a block to customize the struct.",
        context,
    );
}

fn class_factory_inheritance(
    node: &ClassNode<'_>,
    factory: &[u8],
    message: &str,
    context: &mut CopContext<'_, '_>,
) {
    let Some(superclass) = node.superclass() else {
        return;
    };
    let Some(call) = superclass.as_call_node() else {
        return;
    };
    let method = if factory == b"Data" {
        b"define".as_slice()
    } else {
        b"new".as_slice()
    };
    if call.name().as_slice() != method || !root_constant(call.receiver(), factory) {
        return;
    }
    let superclass_location = superclass.location();
    let file = context.source_file();
    let mut edits = vec![
        (
            node.class_keyword_loc().start_offset()..node.constant_path().location().start_offset(),
            String::new(),
        ),
        (
            node.inheritance_operator_loc()
                .expect("class with a superclass has an inheritance operator")
                .start_offset()
                ..node
                    .inheritance_operator_loc()
                    .expect("class with a superclass has an inheritance operator")
                    .end_offset(),
            "=".to_string(),
        ),
    ];

    if let Some(block) = call.block().and_then(|block| block.as_block_node()) {
        let closing = block.closing_loc();
        let line_start = file.line_start(closing.start_offset());
        let removal_start = context.source()[line_start..closing.start_offset()]
            .trim_end_matches([' ', '\t'])
            .len()
            + line_start;
        edits.push((removal_start..closing.end_offset(), String::new()));
    } else if node.body().is_none() {
        let class_end = node.end_keyword_loc();
        let removal = if file.line_start(node.class_keyword_loc().start_offset())
            == file.line_start(class_end.start_offset())
        {
            superclass_location.end_offset()..node.location().end_offset()
        } else {
            file.line_range(class_end.start_offset())
        };
        edits.push((removal, String::new()));
    } else if call.opening_loc().is_none() {
        let selector = call
            .message_loc()
            .expect("Struct.new and Data.define have a selector");
        let arguments = call
            .arguments()
            .map(|arguments| {
                arguments
                    .arguments()
                    .iter()
                    .map(|argument| file.node(&argument))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        edits.push((
            selector.end_offset()..superclass_location.end_offset(),
            format!("({arguments}) do"),
        ));
    } else {
        edits.push((
            superclass_location.end_offset()..superclass_location.end_offset(),
            " do".to_string(),
        ));
    }
    context.replace_many(message, &superclass_location, edits);
}

fn attr(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.name().as_slice() != b"attr"
        || node.receiver().is_some()
        || node.arguments().is_none()
        || context.source().contains("def attr")
    {
        return;
    }
    let arguments = node.arguments().expect("checked above").arguments();
    let last = arguments.last();
    let accessor = last
        .as_ref()
        .is_some_and(|argument| argument.as_true_node().is_some());
    let boolean = last.as_ref().is_some_and(|argument| {
        argument.as_true_node().is_some() || argument.as_false_node().is_some()
    });
    let replacement = if accessor {
        "attr_accessor"
    } else {
        "attr_reader"
    };
    let selector = node.message_loc().expect("attr has a selector");
    let mut edits = vec![(
        selector.start_offset()..selector.end_offset(),
        replacement.to_string(),
    )];
    if boolean {
        let boolean = last.expect("checked above").location();
        let comma = context.source()[..boolean.start_offset()]
            .rfind(',')
            .unwrap_or(boolean.start_offset());
        edits.push((comma..boolean.end_offset(), String::new()));
    }
    context.replace_many(
        format!("Do not use `attr`. Use `{replacement}` instead."),
        &selector,
        edits,
    );
}

fn redundant_initialize(node: &DefNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.name().as_slice() != b"initialize" {
        return;
    }
    let allow_comments = context.config_bool("AllowComments", true);
    if allow_comments && definition_contains_allowed_comments(node, context) {
        return;
    }
    let Some(parameters) = initialize_parameters(node) else { return };
    let empty = node.body().is_none() && parameters.is_empty();
    let redundant_super = node.body()
        .and_then(|body| body.as_statements_node())
        .and_then(|body| (body.body().len() == 1).then(|| body.body().first()).flatten())
        .is_some_and(|body| initialize_forwards_same_arguments(&body, &parameters));
    if !empty && !redundant_super {
        return;
    }
    let message = if empty {
        "Remove unnecessary empty `initialize` method."
    } else {
        "Remove unnecessary `initialize` method."
    };
    let location = node.location();
    let line_start = context.source_file().line_start(location.start_offset());
    let start = if context.source()[line_start..location.start_offset()]
        .trim()
        .is_empty()
    {
        line_start
    } else {
        location.start_offset()
    };
    let end = location.end_offset()
        + usize::from(context.source().as_bytes().get(location.end_offset()) == Some(&b'\n'));
    context.remove(message, &location, start..end);
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum InitializeArgument {
    Positional(Vec<u8>),
    Keyword(Vec<u8>),
    Block(Vec<u8>),
}

fn initialize_parameters(node: &DefNode<'_>) -> Option<Vec<InitializeArgument>> {
    let Some(parameters) = node.parameters() else {
        return Some(Vec::new());
    };
    simple_parameter_names(&parameters)
}

fn simple_parameter_names(parameters: &ParametersNode<'_>) -> Option<Vec<InitializeArgument>> {
    if !parameters.optionals().is_empty()
        || parameters.rest().is_some()
        || parameters.keyword_rest().is_some()
    {
        return None;
    }
    let mut names = Vec::new();
    for parameter in parameters.requireds().iter().chain(parameters.posts().iter()) {
        names.push(InitializeArgument::Positional(
            parameter.as_required_parameter_node()?.name().as_slice().to_vec(),
        ));
    }
    for parameter in parameters.keywords().iter() {
        names.push(InitializeArgument::Keyword(
            parameter.as_required_keyword_parameter_node()?.name().as_slice().to_vec(),
        ));
    }
    if let Some(block) = parameters.block() {
        names.push(InitializeArgument::Block(block.name()?.as_slice().to_vec()));
    }
    Some(names)
}

fn initialize_forwards_same_arguments(body: &Node<'_>, parameters: &[InitializeArgument]) -> bool {
    if body.as_forwarding_super_node().is_some() {
        return true;
    }
    let Some(super_node) = body.as_super_node() else { return false };
    let mut forwarded = Vec::new();
    if let Some(arguments) = super_node.arguments() {
        for argument in arguments.arguments().iter() {
            if let Some(read) = argument.as_local_variable_read_node() {
                forwarded.push(InitializeArgument::Positional(read.name().as_slice().to_vec()));
            } else if let Some(hash) = argument.as_keyword_hash_node() {
                for element in hash.elements().iter() {
                    let Some(pair) = element.as_assoc_node() else { return false };
                    let Some(key) = pair.key().as_symbol_node() else { return false };
                    let Some(value) = pair.value().as_local_variable_read_node() else { return false };
                    if key.unescaped() != value.name().as_slice() {
                        return false;
                    }
                    forwarded.push(InitializeArgument::Keyword(value.name().as_slice().to_vec()));
                }
            } else {
                return false;
            }
        }
    }
    if let Some(block) = super_node.block().and_then(|block| block.as_block_argument_node()) {
        let Some(read) = block.expression().and_then(|expression| expression.as_local_variable_read_node()) else {
            return false;
        };
        forwarded.push(InitializeArgument::Block(read.name().as_slice().to_vec()));
    }
    forwarded == parameters
}

fn definition_contains_allowed_comments(node: &DefNode<'_>, context: &CopContext<'_, '_>) -> bool {
    let Some(statements) = containing_statements(node, context.ancestors()) else {
        return false;
    };
    let body = statements.body().iter().collect::<Vec<_>>();
    let Some(index) = body.iter().position(|statement| same_node(statement, &node.as_node())) else {
        return false;
    };
    let end = if let Some(next) = body.get(index + 1) {
        next.location().start_offset()
    } else if body.len() == 1 {
        node.location().end_offset()
    } else {
        node.location().start_offset()
    };
    let comments = &context.source()[node.location().start_offset()..end];
    comments.lines().any(|line| {
        let Some((_, comment)) = line.split_once('#') else { return false };
        let directive = comment.trim_start();
        if let Some(cops) = directive
            .strip_prefix("rubocop:disable")
            .or_else(|| directive.strip_prefix("rubocop:todo"))
        {
            !cops.split(',').map(str::trim).any(|cop| {
                matches!(cop, "all" | "Style/RedundantInitialize")
            })
        } else {
            true
        }
    })
}

fn containing_statements<'pr>(node: &DefNode<'pr>, ancestors: &[Node<'pr>]) -> Option<StatementsNode<'pr>> {
    ancestors.iter().rev().find_map(|ancestor| {
        let statements = if let Some(program) = ancestor.as_program_node() {
            Some(program.statements())
        } else if let Some(definition) = ancestor.as_def_node() {
            definition.body().and_then(|body| body.as_statements_node())
        } else if let Some(class) = ancestor.as_class_node() {
            class.body().and_then(|body| body.as_statements_node())
        } else if let Some(module) = ancestor.as_module_node() {
            module.body().and_then(|body| body.as_statements_node())
        } else if let Some(singleton) = ancestor.as_singleton_class_node() {
            singleton.body().and_then(|body| body.as_statements_node())
        } else if let Some(block) = ancestor.as_block_node() {
            block.body().and_then(|body| body.as_statements_node())
        } else if let Some(begin) = ancestor.as_begin_node() {
            begin.statements()
        } else {
            None
        }?;
        statements.body().iter().any(|statement| same_node(&statement, &node.as_node())).then_some(statements)
    })
}

fn same_node(left: &Node<'_>, right: &Node<'_>) -> bool {
    left.location().start_offset() == right.location().start_offset()
        && left.location().end_offset() == right.location().end_offset()
}

fn redundant_struct_keyword_init(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(3, 2)
        || node.name().as_slice() != b"new"
        || !root_constant(node.receiver(), b"Struct")
    {
        return;
    }
    let Some(arguments) = node.arguments() else {
        return;
    };
    let Some(last_argument) = arguments.arguments().iter().last() else {
        return;
    };
    let explicit_hash = last_argument.as_hash_node().is_some();
    let pairs = if let Some(keyword_hash) = last_argument.as_keyword_hash_node() {
        keyword_hash
            .elements()
            .iter()
            .filter_map(|element| element.as_assoc_node())
            .collect::<Vec<_>>()
    } else if let Some(hash) = last_argument.as_hash_node() {
        hash.elements()
            .iter()
            .filter_map(|element| element.as_assoc_node())
            .collect::<Vec<_>>()
    } else {
        return;
    };
    let keyword_init = |pair: &ruby_prism::AssocNode<'_>| {
        pair.key()
            .as_symbol_node()
            .is_some_and(|key| key.unescaped() == b"keyword_init")
    };
    if pairs
        .iter()
        .any(|pair| keyword_init(pair) && pair.value().as_false_node().is_some())
    {
        return;
    }
    for (index, pair) in pairs.iter().enumerate().filter(|(_, pair)| {
        keyword_init(pair)
            && (pair.value().as_true_node().is_some() || pair.value().as_nil_node().is_some())
    }) {
        let value = context.source_file().node(&pair.value());
        let offense = pair.location();
        let edit_start = index
            .checked_sub(1)
            .and_then(|previous| pairs.get(previous))
            .map_or(offense.start_offset(), |previous| {
                previous.location().end_offset()
            });
        let edit_start = if index == 0 {
            context.source()[..offense.start_offset()]
                .rfind(',')
                .filter(|comma| {
                    explicit_hash
                        || !context.source()[*comma + 1..offense.start_offset()]
                        .chars()
                        .any(|character| matches!(character, '(' | '{'))
                })
                .unwrap_or(edit_start)
        } else {
            edit_start
        };
        context.remove(
            format!("Remove the redundant `keyword_init: {value}`."),
            &offense,
            edit_start..offense.end_offset(),
        );
    }
}
