use super::*;

define_cops! {
    DataDefineOverride => "Lint/DataDefineOverride" => call(data_define_override),
    StructNewOverride => "Lint/StructNewOverride" => call(struct_new_override),
    MixedRegexpCaptureTypes => "Lint/MixedRegexpCaptureTypes" => node(as_regular_expression_node, mixed_regexp_capture_types),
}

const DATA_METHODS: &[&[u8]] = &[
    b"!",
    b"!=",
    b"!~",
    b"<=>",
    b"==",
    b"===",
    b"__id__",
    b"__send__",
    b"class",
    b"clone",
    b"deconstruct",
    b"deconstruct_keys",
    b"define_singleton_method",
    b"display",
    b"dup",
    b"enum_for",
    b"eql?",
    b"equal?",
    b"extend",
    b"freeze",
    b"frozen?",
    b"hash",
    b"inspect",
    b"instance_eval",
    b"instance_exec",
    b"instance_of?",
    b"instance_variable_defined?",
    b"instance_variable_get",
    b"instance_variable_set",
    b"instance_variables",
    b"is_a?",
    b"itself",
    b"kind_of?",
    b"members",
    b"method",
    b"methods",
    b"nil?",
    b"object_id",
    b"private_methods",
    b"protected_methods",
    b"public_method",
    b"public_methods",
    b"public_send",
    b"remove_instance_variable",
    b"respond_to?",
    b"send",
    b"singleton_class",
    b"singleton_method",
    b"singleton_methods",
    b"tap",
    b"then",
    b"to_enum",
    b"to_h",
    b"to_s",
    b"with",
    b"yield_self",
];

const STRUCT_METHODS: &[&[u8]] = &[
    b"!",
    b"!=",
    b"!~",
    b"<=>",
    b"==",
    b"===",
    b"[]",
    b"[]=",
    b"__id__",
    b"__send__",
    b"all?",
    b"any?",
    b"chain",
    b"chunk",
    b"chunk_while",
    b"class",
    b"clone",
    b"collect",
    b"collect_concat",
    b"compact",
    b"count",
    b"cycle",
    b"deconstruct",
    b"deconstruct_keys",
    b"define_singleton_method",
    b"detect",
    b"dig",
    b"display",
    b"drop",
    b"drop_while",
    b"dup",
    b"each",
    b"each_cons",
    b"each_entry",
    b"each_pair",
    b"each_slice",
    b"each_with_index",
    b"each_with_object",
    b"entries",
    b"enum_for",
    b"eql?",
    b"equal?",
    b"extend",
    b"filter",
    b"filter_map",
    b"find",
    b"find_all",
    b"find_index",
    b"first",
    b"flat_map",
    b"freeze",
    b"frozen?",
    b"grep",
    b"grep_v",
    b"group_by",
    b"hash",
    b"include?",
    b"inject",
    b"inspect",
    b"instance_eval",
    b"instance_exec",
    b"instance_of?",
    b"instance_variable_defined?",
    b"instance_variable_get",
    b"instance_variable_set",
    b"instance_variables",
    b"is_a?",
    b"itself",
    b"kind_of?",
    b"lazy",
    b"length",
    b"map",
    b"max",
    b"max_by",
    b"member?",
    b"members",
    b"method",
    b"methods",
    b"min",
    b"min_by",
    b"minmax",
    b"minmax_by",
    b"nil?",
    b"none?",
    b"object_id",
    b"one?",
    b"partition",
    b"private_methods",
    b"protected_methods",
    b"public_method",
    b"public_methods",
    b"public_send",
    b"reduce",
    b"reject",
    b"remove_instance_variable",
    b"respond_to?",
    b"reverse_each",
    b"select",
    b"send",
    b"singleton_class",
    b"singleton_method",
    b"singleton_methods",
    b"size",
    b"slice_after",
    b"slice_before",
    b"slice_when",
    b"sort",
    b"sort_by",
    b"sum",
    b"take",
    b"take_while",
    b"tally",
    b"tap",
    b"then",
    b"to_a",
    b"to_enum",
    b"to_h",
    b"to_s",
    b"to_set",
    b"uniq",
    b"values",
    b"values_at",
    b"yield_self",
    b"zip",
];

fn data_define_override(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !match_call(node)
        .named(b"define")
        .on_root_constant(b"Data")
        .with_arguments()
        .matches()
    {
        return;
    }
    report_overrides(node, context, "Data", DATA_METHODS, false);
}

fn struct_new_override(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !match_call(node)
        .named(b"new")
        .on_root_constant(b"Struct")
        .with_arguments()
        .matches()
    {
        return;
    }
    report_overrides(node, context, "Struct", STRUCT_METHODS, true);
}

fn report_overrides(
    call: &CallNode<'_>,
    context: &mut CopContext<'_, '_>,
    owner: &str,
    methods: &[&[u8]],
    first_string_is_class_name: bool,
) {
    let Some(arguments) = call.arguments() else {
        return;
    };
    for (index, argument) in arguments.arguments().iter().enumerate() {
        if first_string_is_class_name && index == 0 && argument.as_string_node().is_some() {
            continue;
        }
        let Some((name, inspected)) = member_name(&argument) else {
            continue;
        };
        if !methods.contains(&name.as_slice()) {
            continue;
        }
        let method = String::from_utf8_lossy(&name);
        context.report_node(
            &argument,
            format!("`{inspected}` member overrides `{owner}#{method}` and it may be unexpected."),
        );
    }
}

fn member_name(node: &Node<'_>) -> Option<(Vec<u8>, String)> {
    if let Some(symbol) = node.as_symbol_node() {
        let value = symbol.unescaped().to_vec();
        let inspected = format!(":{}", String::from_utf8_lossy(&value));
        return Some((value, inspected));
    }
    let string = node.as_string_node()?;
    let value = string.unescaped().to_vec();
    let inspected = format!("{:?}", String::from_utf8_lossy(&value));
    Some((value, inspected))
}

fn mixed_regexp_capture_types(
    node: &ruby_prism::RegularExpressionNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let (named, numbered) = capture_types(node.unescaped());
    if named && numbered {
        context.report(
            "Do not mix named captures and numbered captures in a Regexp literal.",
            node.location(),
        );
    }
}

fn capture_types(pattern: &[u8]) -> (bool, bool) {
    let mut named = false;
    let mut numbered = false;
    let mut escaped = false;
    let mut character_class = false;
    let mut index = 0;
    while index < pattern.len() {
        let byte = pattern[index];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'[' {
            character_class = true;
        } else if byte == b']' {
            character_class = false;
        } else if byte == b'(' && !character_class {
            if pattern.get(index + 1) != Some(&b'?') {
                numbered = true;
            } else if pattern.get(index + 2) == Some(&b'<')
                && !matches!(pattern.get(index + 3), Some(b'=') | Some(b'!'))
                || pattern.get(index + 2) == Some(&b'\'')
            {
                named = true;
            }
        }
        index += 1;
    }
    (named, numbered)
}

#[cfg(test)]
mod tests {
    use super::capture_types;

    #[test]
    fn distinguishes_capture_group_kinds() {
        assert_eq!(capture_types(br"(?<foo>bar)(baz)"), (true, true));
        assert_eq!(capture_types(br"(?<foo>bar)(?:baz)"), (true, false));
        assert_eq!(capture_types(br"(?<=foo)(bar)"), (false, true));
        assert_eq!(capture_types(br"[(](?<foo>bar)"), (true, false));
        assert_eq!(capture_types(br"\((?<foo>bar)"), (true, false));
    }
}
