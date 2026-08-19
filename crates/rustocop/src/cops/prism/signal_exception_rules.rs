use super::*;

define_cops! {
    SignalException => "Style/SignalException" => call(signal_exception),
}

fn signal_exception(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let method = call_name(node);
    if !matches!(method, b"raise" | b"fail")
        || node
            .receiver()
            .is_some_and(|receiver| !node_is_root_constant(&receiver, b"Kernel"))
    {
        return;
    }
    if method == b"fail" && node.receiver().is_none() && custom_fail_defined(context.source()) {
        return;
    }
    let in_rescue = in_rescue_region(context.source(), node.location().start_offset());
    let style = context.policy().enforced_style("semantic");
    let (replacement, message) = match style {
        "semantic" if method == b"raise" && !in_rescue => (
            "fail",
            "Use `fail` instead of `raise` to signal exceptions.",
        ),
        "semantic" if method == b"fail" && in_rescue => (
            "raise",
            "Use `raise` instead of `fail` to rethrow exceptions.",
        ),
        "only_raise" if method == b"fail" => ("raise", "Always use `raise` to signal exceptions."),
        "only_fail" if method == b"raise" => ("fail", "Always use `fail` to signal exceptions."),
        _ => return,
    };
    context.replace_selector(node, message, replacement);
}

fn in_rescue_region(source: &str, offset: usize) -> bool {
    let mut boundary = None;
    for (line_start, line) in super::source_helpers::source_lines(source) {
        if line_start >= offset {
            break;
        }
        let keyword = line.trim_start();
        if keyword.starts_with("rescue") {
            boundary = Some(true);
        } else if keyword == "begin" || keyword.starts_with("def ") || keyword == "end" {
            boundary = Some(false);
        }
    }
    boundary == Some(true)
}

fn custom_fail_defined(source: &str) -> bool {
    source.lines().any(|line| {
        let definition = line.trim_start();
        definition.starts_with("def fail(")
            || definition.starts_with("def fail ")
            || definition.starts_with("def self.fail(")
            || definition.starts_with("def self.fail ")
    })
}
