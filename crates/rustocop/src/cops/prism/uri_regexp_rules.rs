use super::*;

define_cops! {
    UriRegexp => "Lint/UriRegexp" => call(on_send),
}

fn on_send(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) != b"regexp" || !root_constant(node.receiver(), b"URI") {
        return;
    }

    let parser = if context.target_ruby_version().at_least(3, 4) {
        "RFC2396_PARSER"
    } else {
        "DEFAULT_PARSER"
    };
    let receiver = context
        .source_file()
        .node(&node.receiver().expect("matched URI receiver"));
    let argument = first_argument(node).map_or_else(String::new, |argument| {
        format!("({})", context.source_file().node(&argument))
    });
    let current = context.source_file().at(&node.location());
    let preferred = format!("{receiver}::{parser}.make_regexp{argument}");
    let selector = node.message_loc().expect("regexp selector");

    context.replace(
        format!(
            "`{current}` is obsolete and should not be used. Instead, use `{preferred}`."
        ),
        &selector,
        node.location(),
        preferred,
    );
}
