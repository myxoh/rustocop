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
    let class_name = context.source_file().node(&node.constant_path());
    let superclass_location = superclass.location();
    let base_end = call.closing_loc().map_or_else(
        || {
            call.block()
                .map_or(superclass_location.end_offset(), |block| {
                    block.location().start_offset()
                })
        },
        |closing| closing.end_offset(),
    );
    let raw_base = context.source()[superclass_location.start_offset()..base_end].trim_end();
    let base = if call.opening_loc().is_none() {
        let selector_end = call
            .message_loc()
            .map_or(superclass_location.start_offset(), |selector| {
                selector.end_offset()
            });
        let prefix = &context.source()[superclass_location.start_offset()..selector_end];
        let arguments = context.source()[selector_end..base_end].trim();
        format!("{prefix}({arguments})")
    } else {
        raw_base.to_string()
    };
    let file = context.source_file();
    let body = node
        .body()
        .and_then(|body| body.as_statements_node())
        .and_then(|statements| statements.body().first())
        .map(|first| {
            context.source()[file.line_start(first.location().start_offset())
                ..file.line_start(node.end_keyword_loc().start_offset())]
                .trim_end_matches('\n')
                .to_string()
        })
        .unwrap_or_default();
    let replacement = if call.block().is_some() {
        format!("{class_name} = {base} do\nend")
    } else if body.trim().is_empty() {
        format!("{class_name} = {base}")
    } else {
        format!("{class_name} = {base} do\n{body}\nend")
    };
    context.replace(message, &superclass_location, node.location(), replacement);
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
    if allow_comments && source.contains('#') {
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
    let empty = body_lines.is_empty() && matches!(signature, "def initialize" | "def initialize()");
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
    let end = location.end_offset()
        + usize::from(context.source().as_bytes().get(location.end_offset()) == Some(&b'\n'));
    context.remove(message, &location, location.start_offset()..end);
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
    let Some(opening) = node.opening_loc() else {
        return;
    };
    let Some(closing) = node.closing_loc() else {
        return;
    };
    let source = context.source();
    let arguments = super::source_syntax::top_level_elements(
        source,
        opening.end_offset(),
        closing.start_offset(),
    );
    if arguments
        .iter()
        .any(|range| source[range.clone()].trim() == "keyword_init: false")
    {
        return;
    }
    for argument in arguments.iter().filter(|range| {
        matches!(
            source[(*range).clone()].trim(),
            "keyword_init: true" | "keyword_init: nil"
        )
    }) {
        let value = source[argument.clone()]
            .split_once(':')
            .map_or("", |(_, value)| value.trim());
        let edit_start = source[..argument.start]
            .rfind(',')
            .filter(|comma| *comma >= opening.end_offset())
            .unwrap_or(argument.start);
        context.remove(
            format!("Remove the redundant `keyword_init: {value}`."),
            argument.clone(),
            edit_start..argument.end,
        );
    }
}
