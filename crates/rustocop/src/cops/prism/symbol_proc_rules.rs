use ruby_prism::{BlockNode, CallNode, Node};

use super::*;

define_cops! {
    SymbolProc => "Style/SymbolProc" => any_node(symbol_proc),
}

fn symbol_proc(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(call) = node.as_call_node() {
        inspect_call(&call, context);
    } else if let Some(lambda) = node.as_lambda_node() {
        let Some(method) = lambda
            .parameters()
            .zip(lambda.body())
            .and_then(|(parameters, body)| symbol_proc_method(parameters, body))
        else {
            return;
        };
        if active_support_extensions_enabled(context) {
            return;
        }
        let opening = lambda.opening_loc();
        let closing = lambda.closing_loc();
        register(
            "lambda",
            &method,
            opening.start_offset()..closing.end_offset(),
            lambda.location().start_offset()..lambda.location().end_offset(),
            format!("lambda(&:{method})"),
            context,
        );
    } else if let Some(super_node) = node.as_super_node() {
        let Some(block) = super_node.block().and_then(|block| block.as_block_node()) else {
            return;
        };
        let Some(method) = block_method(&block) else {
            return;
        };
        if context.config_bool("AllowMethodsWithArguments", false)
            && super_node.arguments().is_some_and(|arguments| !arguments.arguments().is_empty())
        {
            return;
        }
        inspect_dispatch(
            "super",
            &method,
            &block,
            super_node.arguments().map(|arguments| arguments.location()),
            super_node.lparen_loc(),
            super_node.rparen_loc(),
            context,
        );
    } else if let Some(super_node) = node.as_forwarding_super_node() {
        let Some(block) = super_node.block() else {
            return;
        };
        let Some(method) = block_method(&block) else {
            return;
        };
        inspect_dispatch("super", &method, &block, None, None, None, context);
    }
}

fn inspect_call(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(block) = node.block().and_then(|block| block.as_block_node()) else {
        return;
    };
    let Some(method) = block_method(&block) else {
        return;
    };
    let dispatch = node.name().as_slice();
    if context.policy().allows_method(dispatch) {
        return;
    }
    if active_support_extensions_enabled(context) && proc_or_lambda(node) {
        return;
    }
    if unsafe_literal_receiver(node, dispatch) {
        return;
    }
    if context.config_bool("AllowMethodsWithArguments", false)
        && node.arguments().is_some_and(|arguments| !arguments.arguments().is_empty())
    {
        return;
    }
    let dispatch = String::from_utf8_lossy(dispatch);
    inspect_dispatch(
        &dispatch,
        &method,
        &block,
        node.arguments().map(|arguments| arguments.location()),
        node.opening_loc(),
        node.closing_loc(),
        context,
    );
}

fn inspect_dispatch(
    dispatch: &str,
    method: &str,
    block: &BlockNode<'_>,
    arguments: Option<ruby_prism::Location<'_>>,
    opening: Option<ruby_prism::Location<'_>>,
    closing: Option<ruby_prism::Location<'_>>,
    context: &mut CopContext<'_, '_>,
) {
    let offense = block.opening_loc().start_offset()..block.closing_loc().end_offset();
    if context.config_bool("AllowComments", false) && contains_comment(context.source(), &offense) {
        return;
    }

    let edits = dispatch_correction(method, block, arguments, opening, closing, context);
    context.replace_many(
        format!("Pass `&:{method}` as an argument to `{dispatch}` instead of a block."),
        offense,
        edits,
    );
}

fn register(
    dispatch: &str,
    method: &str,
    offense: std::ops::Range<usize>,
    edit: std::ops::Range<usize>,
    replacement: String,
    context: &mut CopContext<'_, '_>,
) {
    context.replace(
        format!("Pass `&:{method}` as an argument to `{dispatch}` instead of a block."),
        offense,
        edit,
        replacement,
    );
}

fn dispatch_correction(
    method: &str,
    block: &BlockNode<'_>,
    arguments: Option<ruby_prism::Location<'_>>,
    opening: Option<ruby_prism::Location<'_>>,
    closing: Option<ruby_prism::Location<'_>>,
    context: &CopContext<'_, '_>,
) -> Vec<(std::ops::Range<usize>, String)> {
    let block_range = block.location().start_offset()..block.location().end_offset();
    let file = context.source_file();
    if let (Some(arguments), Some(closing)) = (arguments.as_ref(), closing.as_ref()) {
        let between = arguments.end_offset()..closing.start_offset();
        let source = file.slice(between.clone()).unwrap_or_default();
        if let Some(comma) = source.find(',') {
            let whitespace = &source[comma + 1..];
            return vec![
                (
                    arguments.end_offset() + comma + 1..closing.start_offset(),
                    format!(" &:{method}{whitespace}"),
                ),
                (
                    file.whitespace_before(block_range.start).start..block_range.end,
                    String::new(),
                ),
            ];
        }
        return vec![(
            closing.start_offset()..block_range.end,
            format!(", &:{method})"),
        )];
    }
    if let Some(arguments) = arguments {
        return vec![
            (
                arguments.end_offset()..arguments.end_offset(),
                format!(", &:{method}"),
            ),
            (
                file.whitespace_before(block_range.start).start..block_range.end,
                String::new(),
            ),
        ];
    }
    if let Some(opening) = opening {
        return vec![(
            opening.start_offset()..block_range.end,
            format!("(&:{method})"),
        )];
    }
    let whitespace = file.whitespace_before(block_range.start);
    vec![(
        whitespace.start..block_range.end,
        format!("(&:{method})"),
    )]
}

fn block_method(block: &BlockNode<'_>) -> Option<String> {
    symbol_proc_method(block.parameters()?, block.body()?)
}

fn symbol_proc_method(parameters: Node<'_>, body: Node<'_>) -> Option<String> {
    let body = single_expression(body)?;
    let call = body.as_call_node()?;
    if call.arguments().is_some_and(|arguments| !arguments.arguments().is_empty())
        || call.block().is_some()
    {
        return None;
    }
    let receiver = call.receiver()?;

    let receiver_matches = if let Some(numbered) = parameters.as_numbered_parameters_node() {
        numbered.maximum() == 1
            && receiver
                .as_local_variable_read_node()
                .is_some_and(|read| read.name().as_slice() == b"_1")
    } else if parameters.as_it_parameters_node().is_some() {
        receiver.as_it_local_variable_read_node().is_some()
    } else {
        let block_parameters = parameters.as_block_parameters_node()?;
        let parameters = block_parameters.parameters()?;
        if parameters.requireds().len() != 1
            || !parameters.optionals().is_empty()
            || parameters.rest().is_some()
            || !parameters.posts().is_empty()
            || !parameters.keywords().is_empty()
            || parameters.keyword_rest().is_some()
            || parameters.block().is_some()
        {
            return None;
        }
        let parameter = parameters.requireds().first()?.as_required_parameter_node()?;
        receiver
            .as_local_variable_read_node()
            .is_some_and(|read| read.name().as_slice() == parameter.name().as_slice())
    };
    receiver_matches.then(|| String::from_utf8_lossy(call.name().as_slice()).into_owned())
}

fn single_expression(node: Node<'_>) -> Option<Node<'_>> {
    if let Some(statements) = node.as_statements_node() {
        let body = statements.body();
        return (body.len() == 1).then(|| body.first()).flatten();
    }
    Some(node)
}

fn active_support_extensions_enabled(context: &CopContext<'_, '_>) -> bool {
    context.related_config_value("AllCops", "ActiveSupportExtensionsEnabled") == Some("true")
}

fn proc_or_lambda(node: &CallNode<'_>) -> bool {
    match node.name().as_slice() {
        b"lambda" | b"proc" => node.receiver().is_none(),
        b"new" => node.receiver().is_some_and(|receiver| {
            receiver
                .as_constant_read_node()
                .is_some_and(|constant| constant.name().as_slice() == b"Proc")
                || receiver.as_constant_path_node().is_some_and(|path| {
                    path.name().is_some_and(|name| name.as_slice() == b"Proc")
                })
        }),
        _ => false,
    }
}

fn unsafe_literal_receiver(node: &CallNode<'_>, method: &[u8]) -> bool {
    let Some(receiver) = node.receiver() else {
        return false;
    };
    (receiver.as_hash_node().is_some() && matches!(method, b"reject" | b"select"))
        || (receiver.as_array_node().is_some() && matches!(method, b"min" | b"max"))
}

fn contains_comment(source: &str, range: &std::ops::Range<usize>) -> bool {
    let Some(source) = source.get(range.clone()) else {
        return false;
    };
    let mut quote = None;
    let mut escaped = false;
    for byte in source.bytes() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == delimiter {
                quote = None;
            }
        } else if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
        } else if byte == b'#' {
            return true;
        }
    }
    false
}
