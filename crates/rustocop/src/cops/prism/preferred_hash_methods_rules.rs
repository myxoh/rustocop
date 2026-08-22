use super::*;

define_cops! {
    PreferredHashMethods => "Style/PreferredHashMethods" => call(preferred_hash_methods),
}

fn preferred_hash_methods(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if argument_count(node) == 0 {
        return;
    }
    let rules = if context.policy().enforced_style("short") == "verbose" {
        [
            (
                "key?",
                "has_key?",
                "Use `Hash#has_key?` instead of `Hash#key?`.",
            ),
            (
                "value?",
                "has_value?",
                "Use `Hash#has_value?` instead of `Hash#value?`.",
            ),
        ]
    } else {
        [
            (
                "has_key?",
                "key?",
                "Use `Hash#key?` instead of `Hash#has_key?`.",
            ),
            (
                "has_value?",
                "value?",
                "Use `Hash#value?` instead of `Hash#has_value?`.",
            ),
        ]
    };

    for (old, new, message) in rules {
        if call_name(node) == old.as_bytes() {
            context.replace_selector(node, message, new);
        }
    }
}
