use ruby_prism::{CallNode, ClassNode, DefNode};

use super::*;

define_cops! {
    Attr => "Style/Attr" => call(attr),
    DataInheritance => "Style/DataInheritance" => node(as_class_node, data_inheritance),
    RedundantInitialize => "Style/RedundantInitialize" => node(as_def_node, redundant_initialize),
    RedundantStructKeywordInit => "Style/RedundantStructKeywordInit" => call(redundant_struct_keyword_init),
    StructInheritance => "Style/StructInheritance" => node(as_class_node, struct_inheritance),
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
    let source = context.source_file().at(&node.location());
    let allow_comments = context.config_bool("AllowComments", true);
    if allow_comments
        && (source.contains('#')
            || comments_before_next_statement(node.location().end_offset(), context.source()))
    {
        return;
    }
    let signature = source.lines().next().unwrap_or_default().trim();
    let body_lines = source
        .lines()
        .skip(1)
        .take(source.lines().count().saturating_sub(2))
        .map(str::trim)
        .filter(|line| !line.is_empty() && (allow_comments || !line.starts_with('#')))
        .collect::<Vec<_>>();
    let empty = node.body().is_none() && node.parameters().is_none();
    let redundant_super =
        body_lines.len() == 1 && super_matches_signature(signature, body_lines[0]);
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

fn comments_before_next_statement(offset: usize, source: &str) -> bool {
    source[offset..]
        .lines()
        .skip(1)
        .map(str::trim)
        .find(|line| !line.is_empty())
        .is_some_and(|line| line.starts_with('#'))
}

fn super_matches_signature(signature: &str, body: &str) -> bool {
    let parameters = signature
        .strip_prefix("def initialize")
        .unwrap_or_default()
        .trim();
    if body == "super" {
        return !parameters.contains('=') && !parameters.contains('*') && !parameters.contains(':');
    }
    if body == "super()" {
        return parameters == "()";
    }
    let Some(super_arguments) = body
        .strip_prefix("super(")
        .and_then(|body| body.strip_suffix(')'))
    else {
        return false;
    };
    parameters
        .strip_prefix('(')
        .and_then(|parameters| parameters.strip_suffix(')'))
        .is_some_and(|parameters| {
            parameters.split_whitespace().collect::<String>()
                == super_arguments.split_whitespace().collect::<String>()
        })
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
