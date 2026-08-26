use super::*;

define_cops! {
    Dir => "Style/Dir" => call(dir_method),
}

fn dir_method(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let matched = if match_call(node)
        .named(b"expand_path")
        .on_root_constant(b"File")
        .with_argument_count(1)
        .matches()
    {
        only_argument(node).is_some_and(|argument| file_call_with_source_file(&argument, b"dirname"))
    } else if match_call(node)
        .named(b"dirname")
        .on_root_constant(b"File")
        .with_argument_count(1)
        .matches()
    {
        only_argument(node).is_some_and(|argument| file_call_with_source_file(&argument, b"realpath"))
    } else {
        false
    };
    if matched {
        context.replace_call(
            node,
            "Use `__dir__` to get an absolute path to the current file's directory.",
            "__dir__",
        );
    }
}

fn file_call_with_source_file(node: &Node<'_>, method: &[u8]) -> bool {
    node.as_call_node().is_some_and(|call| {
        match_call(&call)
            .named(method)
            .on_root_constant(b"File")
            .with_only_argument_matching(|argument| argument.as_source_file_node().is_some())
            .matches()
    })
}
