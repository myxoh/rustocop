use super::*;

define_cops! {
}

fn uri_escape_unescape(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let method = call_name(node);
    if !matches!(method, b"escape" | b"encode" | b"unescape" | b"decode")
        || !root_constant(node.receiver(), b"URI")
    {
        return;
    }
    let receiver = node.receiver().expect("URI call has a receiver");
    let receiver_source = context.source_file().node(&receiver);
    let method = String::from_utf8_lossy(method);
    let alternatives = if matches!(method.as_ref(), "escape" | "encode") {
        "`CGI.escape`, `URI.encode_www_form` or `URI.encode_www_form_component`"
    } else {
        "`CGI.unescape`, `URI.decode_www_form` or `URI.decode_www_form_component`"
    };
    context.report(
        format!("`{receiver_source}.{method}` method is obsolete and should not be used. Instead, use {alternatives} depending on your specific use case."),
        node.location().start_offset()..node.location().end_offset(),
    );
}
