use super::*;

define_cops! {
    NestedPercentLiteral => "Lint/NestedPercentLiteral" => compatibility_prism_node(as_array_node, nested_percent_literal),
}

fn numeric_literal_prefix(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if node.as_integer_node().is_none() {
        return;
    }
    let location = node.location();
    let literal = context.source_file().at(&location);
    let zero_only = context.config_value("EnforcedOctalStyle") == Some("zero_only");
    let (message, replacement) =
        if zero_only && (literal.starts_with("0o") || literal.starts_with("0O")) {
                ("Use 0 for octal literals.", format!("0{}", &literal[2..]))
            } else if let Some(digits) = literal.strip_prefix("0X") {
                ("Use 0x for hexadecimal literals.", format!("0x{digits}"))
            } else if let Some(digits) = literal.strip_prefix("0B") {
                ("Use 0b for binary literals.", format!("0b{digits}"))
            } else if let Some(digits) = literal.strip_prefix("0O") {
                ("Use 0o for octal literals.", format!("0o{digits}"))
            } else if literal.starts_with("0d") || literal.starts_with("0D") {
                (
                    "Do not use prefixes for decimal literals.",
                    literal[2..].to_string(),
                )
            } else if !zero_only
                && literal.len() > 1
                && literal.starts_with('0')
                && literal[1..].bytes().all(|byte| matches!(byte, b'0'..=b'7'))
            {
                ("Use 0o for octal literals.", format!("0o{}", &literal[1..]))
            } else {
                return;
        };
    context.replace(message, &location, &location, replacement);
}

fn nested_percent_literal(node: &ruby_prism::ArrayNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(opening) = node.opening_loc() else {
        return;
    };
    if !matches!(opening.as_slice(), bytes if bytes.starts_with(b"%w") || bytes.starts_with(b"%W") || bytes.starts_with(b"%i") || bytes.starts_with(b"%I")) {
        return;
    }
    let contains_percent_literal = node.elements().iter().any(|element| {
        let content = if let Some(string) = element.as_string_node() {
            string.unescaped().to_vec()
        } else if let Some(symbol) = element.as_symbol_node() {
            symbol.unescaped().to_vec()
        } else {
            return false;
        };
        [b"%i".as_slice(), b"%I", b"%q", b"%Q", b"%r", b"%s", b"%w", b"%W", b"%x", b"%"]
            .into_iter()
            .any(|prefix| {
                content.strip_prefix(prefix).is_some_and(|rest| {
                    rest.first().is_some_and(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
                })
            })
    });
    if contains_percent_literal {
        context.report(
            "Within percent literals, nested percent literals do not function and may be unwanted in the result.",
            node.location(),
        );
    }
}

fn open_struct_use(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if !node_is_root_constant(node, b"OpenStruct") {
        return;
    }
    let definition = context.parent().is_some_and(|parent| {
        parent
            .as_class_node()
            .is_some_and(|class| same_location(&class.constant_path(), node))
            || parent
                .as_module_node()
                .is_some_and(|module| same_location(&module.constant_path(), node))
    });
    if !definition {
        context.report(
            "Avoid using `OpenStruct`; use `Struct`, `Hash`, a class or test doubles instead.",
            node.location(),
        );
    }
}
