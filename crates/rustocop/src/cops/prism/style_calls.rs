use super::*;

define_cops! {
    ColonMethodCall => "Style/ColonMethodCall" => call(colon_method_call),
}

fn colon_method_call(node: &CallNode<'_>, reporter: &mut CopContext<'_, '_>) {
    let method = call_name(node);
    if !match_call(node).with_operator(b"::").matches()
        || method.first().is_some_and(u8::is_ascii_uppercase)
        || root_constant(node.receiver(), b"Java")
    {
        return;
    }
    let Some(operator) = node.call_operator_loc() else {
        return;
    };

    reporter.replace(
        "Do not use `::` for method calls.",
        &operator,
        &operator,
        ".",
    );
}
