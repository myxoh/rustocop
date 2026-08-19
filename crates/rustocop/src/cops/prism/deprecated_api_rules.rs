use super::*;

define_cops! {
    DeprecatedClassMethods => "Lint/DeprecatedClassMethods" => call(deprecated_class_methods),
}

fn deprecated_class_methods(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) == b"iterator?" && node.receiver().is_none() && argument_count(node) == 0 {
        context.replace_selector(
            node,
            "`iterator?` is deprecated in favor of `block_given?`.",
            "block_given?",
        );
        return;
    }
    if call_name(node) == b"attr" && node.receiver().is_none() && argument_count(node) == 2 {
        let arguments = node.arguments().expect("two arguments checked").arguments();
        let (Some(name), Some(flag)) = (arguments.first(), arguments.iter().nth(1)) else {
            return;
        };
        let preferred = if flag.as_true_node().is_some() {
            "attr_accessor"
        } else if flag.as_false_node().is_some() {
            "attr_reader"
        } else {
            return;
        };
        let original = context.source_file().node(&node.as_node());
        let replacement = format!("{preferred} {}", context.source_file().node(&name));
        context.replace_call(
            node,
            format!("`{original}` is deprecated in favor of `{replacement}`."),
            replacement,
        );
        return;
    }
    if root_constant(node.receiver(), b"ENV") && argument_count(node) == 0 {
        let replacement = match call_name(node) {
            b"freeze" => "ENV",
            b"clone" | b"dup" => "ENV.to_h",
            _ => return,
        };
        let original = context.source_file().node(&node.as_node());
        context.replace_call(
            node,
            format!("`{original}` is deprecated in favor of `{replacement}`."),
            replacement,
        );
        return;
    }
    let receiver_class = if root_constant(node.receiver(), b"File") {
        "File"
    } else if root_constant(node.receiver(), b"Dir") {
        "Dir"
    } else if root_constant(node.receiver(), b"Socket") {
        "Socket"
    } else {
        return;
    };
    let Some(receiver) = node.receiver() else {
        return;
    };
    let Some(selector) = node.message_loc() else {
        return;
    };
    let offense = receiver.location().start_offset()..selector.end_offset();
    let original = &context.source()[offense.clone()];
    if matches!(receiver_class, "File" | "Dir") && call_name(node) == b"exists?" {
        let preferred = original
            .strip_suffix("exists?")
            .unwrap_or(original)
            .to_string()
            + "exist?";
        context.replace(
            format!("`{original}` is deprecated in favor of `{preferred}`."),
            offense,
            &selector,
            "exist?",
        );
    } else if receiver_class == "Socket" && call_name(node) == b"gethostbyaddr" {
        context.report(
            format!("`{original}` is deprecated in favor of `Addrinfo#getnameinfo`."),
            offense,
        );
    } else if receiver_class == "Socket" && call_name(node) == b"gethostbyname" {
        context.report(
            format!("`{original}` is deprecated in favor of `Addrinfo.getaddrinfo`."),
            offense,
        );
    }
}
