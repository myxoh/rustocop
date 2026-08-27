use super::*;

define_cops! {
    RedundantSortBlock => "Performance/RedundantSortBlock" => compatibility_prism_node(as_block_node, redundant_sort_block),
    ReverseEach => "Performance/ReverseEach" => compatibility_prism_call(reverse_each),
    ReverseFirst => "Performance/ReverseFirst" => compatibility_prism_call(reverse_first),
    Size => "Performance/Size" => compatibility_prism_call(size),
    StringBytesize => "Performance/StringBytesize" => compatibility_prism_call(string_bytesize),
}

fn reverse_first(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !match_call(node).named(b"first").matches() {
        return;
    }
    let Some(reverse) = receiver_call(node).filter(|call| {
        match_call(call)
            .named(b"reverse")
            .without_arguments()
            .without_block()
            .matches()
    }) else {
        return;
    };
    let argument = match argument_count(node) {
        0 => None,
        1 => only_argument(node).filter(|argument| argument.as_integer_node().is_some()),
        _ => return,
    };
    if argument_count(node) == 1 && argument.is_none() {
        return;
    }
    let (Some(reverse_selector), Some(operator)) =
        (reverse.message_loc(), node.call_operator_loc())
    else {
        return;
    };
    let (replacement, good_method) = argument.map_or_else(
        || ("last".to_string(), "last".to_string()),
        |argument| {
            let argument = context.source_file().node(&argument);
            let dot = context.source_file().at(&operator);
            let method = format!("last({argument}){dot}reverse");
            (method.clone(), method)
        },
    );
    let range = reverse_selector.start_offset()..node.location().end_offset();
    let bad_method = context.source_file().slice(range.clone()).unwrap_or_default();
    context.replace(
        format!("Use `{good_method}` instead of `{bad_method}`."),
        range.clone(),
        range,
        replacement,
    );
}

fn size(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !match_call(node)
        .named(b"count")
        .without_arguments()
        .matches()
        || node.block().is_some()
        || context
            .ancestors()
            .iter()
            .rev()
            .find(|parent| parent.as_statements_node().is_none())
            .is_some_and(|parent| parent.as_block_node().is_some())
    {
        return;
    }
    let Some(_) = node.receiver().filter(known_collection) else {
        return;
    };
    context.replace_selector(node, "Use `size` instead of `count`.", "size");
}

fn known_collection(node: &Node<'_>) -> bool {
    if node.as_array_node().is_some() || node.as_hash_node().is_some() {
        return true;
    }
    let Some(call) = node.as_call_node() else {
        return false;
    };
    match call_name(&call) {
        b"to_a" | b"to_h" => argument_count(&call) == 0,
        b"[]" => {
            argument_count(&call) == 1
                && (root_constant(call.receiver(), b"Array")
                    || root_constant(call.receiver(), b"Hash"))
        }
        b"Array" | b"Hash" => call.receiver().is_none() && argument_count(&call) == 1,
        _ => false,
    }
}

fn string_bytesize(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !match_call(node)
        .named_any(&[b"size", b"length", b"count"])
        .without_arguments()
        .matches()
    {
        return;
    }
    let Some(bytes) = receiver_call(node).filter(|call| {
        match_call(call)
            .named(b"bytes")
            .with_receiver()
            .without_arguments()
            .without_block()
            .matches()
            && call.receiver().is_some_and(|receiver| receiver.as_integer_node().is_none())
    }) else {
        return;
    };
    let (Some(selector), Some(outer_selector)) = (bytes.message_loc(), node.message_loc()) else {
        return;
    };
    let range = selector.start_offset()..outer_selector.end_offset();
    context.replace(
        "Use `String#bytesize` instead of calculating the size of the bytes array.",
        range.clone(),
        range,
        "bytesize",
    );
}

fn reverse_each(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let ancestors = context.ancestors();
    if !match_call(node)
        .named(b"each")
        .without_arguments()
        .matches()
        || ancestors.iter().enumerate().any(|(index, ancestor)| {
            assignment_node(ancestor)
                || ancestor.as_return_node().is_some()
                || ancestor.as_call_node().is_some()
                    && ancestors
                        .get(index + 1)
                        .is_none_or(|child| child.as_block_node().is_none())
        })
    {
        return;
    }
    let Some(reverse) = receiver_call(node).filter(|call| {
        match_call(call)
            .named(b"reverse")
            .without_arguments()
            .without_block()
            .matches()
    }) else {
        return;
    };
    let (Some(reverse_selector), Some(each_selector)) =
        (reverse.message_loc(), node.message_loc())
    else {
        return;
    };
    let range = reverse_selector.start_offset()..each_selector.end_offset();
    context.replace(
        "Use `reverse_each` instead of `reverse.each`.",
        range.clone(),
        range,
        "reverse_each",
    );
}

fn assignment_node(node: &Node<'_>) -> bool {
    macro_rules! assignment {
        ($($cast:ident),+ $(,)?) => {
            $(if node.$cast().is_some() { return true; })+
        };
    }
    assignment!(
        as_multi_write_node,
        as_local_variable_write_node,
        as_instance_variable_write_node,
        as_class_variable_write_node,
        as_global_variable_write_node,
        as_constant_write_node,
        as_constant_path_write_node,
        as_local_variable_or_write_node,
        as_instance_variable_or_write_node,
        as_class_variable_or_write_node,
        as_global_variable_or_write_node,
        as_constant_or_write_node,
        as_constant_path_or_write_node,
        as_local_variable_and_write_node,
        as_instance_variable_and_write_node,
        as_class_variable_and_write_node,
        as_global_variable_and_write_node,
        as_constant_and_write_node,
        as_constant_path_and_write_node,
        as_local_variable_operator_write_node,
        as_instance_variable_operator_write_node,
        as_class_variable_operator_write_node,
        as_global_variable_operator_write_node,
        as_constant_operator_write_node,
        as_constant_path_operator_write_node,
    );
    false
}

fn redundant_sort_block(node: &ruby_prism::BlockNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(sort) = context.parent().and_then(Node::as_call_node).filter(|call| {
        match_call(call)
            .named(b"sort")
            .without_arguments()
            .matches()
    }) else {
        return;
    };
    let Some((left, right)) = sort_parameters(node) else {
        return;
    };
    let Some(comparison) = node
        .body()
        .and_then(single_expression)
        .and_then(|body| body.as_call_node())
        .filter(|call| call_name(call) == b"<=>" && argument_count(call) == 1)
    else {
        return;
    };
    let receiver_matches = comparison.receiver().is_some_and(|receiver| {
        receiver
            .as_local_variable_read_node()
            .is_some_and(|read| read.name().as_slice() == left)
    });
    let argument_matches = only_argument(&comparison).is_some_and(|argument| {
        argument
            .as_local_variable_read_node()
            .is_some_and(|read| read.name().as_slice() == right)
    });
    if !receiver_matches || !argument_matches {
        return;
    }
    let (Some(selector), closing) = (sort.message_loc(), node.closing_loc()) else {
        return;
    };
    let range = selector.start_offset()..closing.end_offset();
    context.replace(
        "Use `sort` without block.",
        range.clone(),
        range,
        "sort",
    );
}

fn sort_parameters(node: &ruby_prism::BlockNode<'_>) -> Option<(Vec<u8>, Vec<u8>)> {
    let parameters = node.parameters()?;
    if let Some(numbered) = parameters.as_numbered_parameters_node() {
        return (numbered.maximum() == 2).then(|| (b"_1".to_vec(), b"_2".to_vec()));
    }
    let parameters = parameters.as_block_parameters_node()?.parameters()?;
    let required = parameters.requireds().iter().collect::<Vec<_>>();
    let [left, right] = required.as_slice() else {
        return None;
    };
    if !parameters.optionals().is_empty()
        || parameters.rest().is_some()
        || !parameters.posts().is_empty()
        || !parameters.keywords().is_empty()
        || parameters.keyword_rest().is_some()
        || parameters.block().is_some()
    {
        return None;
    }
    Some((
        left.as_required_parameter_node()?.name().as_slice().to_vec(),
        right.as_required_parameter_node()?.name().as_slice().to_vec(),
    ))
}
