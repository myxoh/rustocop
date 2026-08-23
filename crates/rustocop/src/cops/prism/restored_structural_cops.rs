use std::collections::HashSet;

use ruby_prism::{CallNode, HashNode, Node, ReturnNode, StringNode};

use super::*;

define_cops! {
    DuplicateHashKey => "Lint/DuplicateHashKey" => node(as_hash_node, duplicate_hash_key),
    InterpolationCheck => "Lint/InterpolationCheck" => node(as_string_node, interpolation_check),
    TopLevelReturnWithArgument => "Lint/TopLevelReturnWithArgument" => node(as_return_node, top_level_return_with_argument),
    ImplicitRuntimeError => "Style/ImplicitRuntimeError" => call(implicit_runtime_error),
    RedundantConstantBase => "Style/RedundantConstantBase" => any_node(redundant_constant_base),
}

fn duplicate_hash_key(node: &HashNode<'_>, context: &mut CopContext<'_, '_>) {
    let mut seen = HashSet::new();
    for element in node.elements().iter() {
        let Some(pair) = element.as_assoc_node() else {
            continue;
        };
        let key = pair.key();
        if !duplicate_hash_key_candidate(&key) {
            continue;
        }
        let fingerprint = duplicate_hash_key_fingerprint(&key, context);
        if !seen.insert(fingerprint) {
            let label_value = key.as_symbol_node().and_then(|symbol| {
                symbol.opening_loc().is_none().then(|| symbol.value_loc()).flatten()
            });
            context.report(
                "Duplicated key in hash literal.",
                label_value.unwrap_or_else(|| key.location()),
            );
        }
    }
}

fn duplicate_hash_key_fingerprint(node: &Node<'_>, context: &CopContext<'_, '_>) -> String {
    if let Some(symbol) = node.as_symbol_node() {
        return format!("symbol:{:?}", symbol.unescaped());
    }
    if let Some(string) = node.as_string_node() {
        return format!("string:{:?}", string.unescaped());
    }
    if node.as_true_node().is_some() {
        return "true".to_string();
    }
    if node.as_false_node().is_some() {
        return "false".to_string();
    }
    if node.as_nil_node().is_some() {
        return "nil".to_string();
    }
    context.source_file().node(node).to_string()
}

fn duplicate_hash_key_candidate(node: &Node<'_>) -> bool {
    const STATIC_LITERAL: u16 = 0x2;
    if node.as_constant_read_node().is_some() || node.as_constant_path_node().is_some() {
        return true;
    }
    if node.as_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_rational_node().is_some()
        || node.as_imaginary_node().is_some()
        || node.as_regular_expression_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_nil_node().is_some()
    {
        return true;
    }
    if let Some(call) = node.as_call_node() {
        let literal_operator = matches!(
            call.name().as_slice(),
            b"!" | b"<=>" | b"==" | b"!=" | b"<" | b">" | b"<=" | b">="
                | b"+" | b"-" | b"*" | b"/" | b"%" | b"**" | b"~"
        );
        return literal_operator
            && call.receiver().as_ref().is_some_and(duplicate_hash_key_candidate)
            && call.arguments().is_none_or(|arguments| {
                arguments.arguments().iter().all(|argument| {
                    duplicate_hash_key_candidate(&argument)
                })
            });
    }
    if let Some(array) = node.as_array_node() {
        return array.elements().iter().all(|element| {
            duplicate_hash_key_candidate(&element)
        });
    }
    if let Some(hash) = node.as_hash_node() {
        return hash.elements().iter().all(|element| {
            element.as_assoc_node().is_some_and(|pair| {
                duplicate_hash_key_candidate(&pair.key())
                    && duplicate_hash_key_candidate(&pair.value())
            })
        });
    }
    if let Some(parentheses) = node.as_parentheses_node() {
        return parentheses
            .body()
            .and_then(|body| body.as_statements_node())
            .is_some_and(|statements| {
                statements.body().len() == 1
                    && statements
                        .body()
                        .first()
                        .as_ref()
                        .is_some_and(duplicate_hash_key_candidate)
            });
    }
    if let Some(and_node) = node.as_and_node() {
        return duplicate_hash_key_candidate(&and_node.left())
            && duplicate_hash_key_candidate(&and_node.right());
    }
    if let Some(or_node) = node.as_or_node() {
        return duplicate_hash_key_candidate(&or_node.left())
            && duplicate_hash_key_candidate(&or_node.right());
    }
    if let Some(string) = node.as_interpolated_string_node() {
        return interpolated_parts_are_static(string.parts().iter());
    }
    if let Some(symbol) = node.as_interpolated_symbol_node() {
        return interpolated_parts_are_static(symbol.parts().iter());
    }
    if let Some(regexp) = node.as_interpolated_regular_expression_node() {
        return interpolated_parts_are_static(regexp.parts().iter());
    }
    macro_rules! has_static_literal_flag {
        ($($cast:ident),+ $(,)?) => {$ (
            if let Some(node) = node.$cast() {
                return node.flags() & STATIC_LITERAL != 0;
            }
        )+ };
    }
    has_static_literal_flag!(
        as_false_node,
        as_float_node,
        as_imaginary_node,
        as_integer_node,
        as_interpolated_regular_expression_node,
        as_interpolated_string_node,
        as_interpolated_symbol_node,
        as_nil_node,
        as_range_node,
        as_rational_node,
        as_regular_expression_node,
        as_string_node,
        as_symbol_node,
        as_true_node,
    );
    false
}

fn interpolated_parts_are_static<'pr>(mut parts: impl Iterator<Item = Node<'pr>>) -> bool {
    parts.all(|part| {
        if part.as_string_node().is_some() {
            return true;
        }
        let Some(embedded) = part.as_embedded_statements_node() else {
            return false;
        };
        embedded.statements().is_some_and(|statements| {
            statements.body().len() == 1
                && statements
                    .body()
                    .first()
                    .as_ref()
                    .is_some_and(duplicate_hash_key_candidate)
        })
    })
}

fn interpolation_check(node: &StringNode<'_>, context: &mut CopContext<'_, '_>) {
    const MESSAGE: &str = "Interpolation in single quoted string detected. Use double quoted strings if you need interpolation.";
    let location = node.location();
    let source = context.source_file().at(&location);
    if !source.starts_with('\'')
        || !source.ends_with('\'')
        || source.contains('\n')
        || !contains_unescaped_interpolation(source)
        || context.ancestors().iter().any(|ancestor| {
            ancestor.as_regular_expression_node().is_some()
                || ancestor.as_interpolated_regular_expression_node().is_some()
        })
    {
        return;
    }

    let replacement = if source.contains('"') {
        format!("%{{{}}}", &source[1..source.len() - 1])
    } else {
        format!("\"{}\"", &source[1..source.len() - 1])
    };
    let probe = format!("def __rustocop_interpolation_probe__; {replacement}; end");
    let valid = {
        let parsed = ruby_prism::parse(probe.as_bytes());
        parsed.errors().next().is_none() && contains_interpolated_string(&parsed.node())
    };
    if !valid {
        return;
    }
    context.replace(MESSAGE, &location, &location, replacement);
}

fn contains_unescaped_interpolation(source: &str) -> bool {
    source
        .match_indices("#{")
        .any(|(at, _)| source.as_bytes().get(at.wrapping_sub(1)) != Some(&b'\\'))
}

fn contains_interpolated_string(root: &Node<'_>) -> bool {
    struct Finder(bool);
    impl<'pr> Visit<'pr> for Finder {
        fn visit_interpolated_string_node(
            &mut self,
            _node: &ruby_prism::InterpolatedStringNode<'pr>,
        ) {
            self.0 = true;
        }
    }
    let mut finder = Finder(false);
    finder.visit(root);
    finder.0
}

fn top_level_return_with_argument(node: &ReturnNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.arguments().is_none()
        || context.ancestors().iter().any(|ancestor| {
            ancestor.as_block_node().is_some()
                || ancestor.as_def_node().is_some()
                || ancestor.as_lambda_node().is_some()
        })
    {
        return;
    }
    let location = node.location();
    context.replace(
        "Top level return with argument detected.",
        &location,
        &location,
        "return",
    );
}

fn implicit_runtime_error(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.receiver().is_some() || !matches!(node.name().as_slice(), b"raise" | b"fail") {
        return;
    }
    let Some(argument) = node.arguments().and_then(|arguments| {
        (arguments.arguments().len() == 1).then(|| arguments.arguments().first()).flatten()
    }) else {
        return;
    };
    if argument.as_string_node().is_none() && argument.as_interpolated_string_node().is_none() {
        return;
    }
    let method = String::from_utf8_lossy(node.name().as_slice());
    context.report_call(
        node,
        format!("Use `{method}` with an explicit exception class and message, rather than just a message."),
    );
}

fn redundant_constant_base(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let path = if let Some(path) = node.as_constant_path_node() {
        path
    } else if let Some(write) = node.as_constant_path_write_node() {
        write.target()
    } else if let Some(write) = node.as_constant_path_and_write_node() {
        write.target()
    } else if let Some(write) = node.as_constant_path_or_write_node() {
        write.target()
    } else if let Some(write) = node.as_constant_path_operator_write_node() {
        write.target()
    } else {
        return;
    };
    if path.parent().is_some()
        || path.delimiter_loc().as_slice() != b"::"
        || context.related_config_value("Lint/ConstantResolution", "Enabled") == Some("true")
    {
        return;
    }
    let location = path.location();
    for ancestor in context.ancestors().iter().rev() {
        if ancestor.as_module_node().is_some() {
            return;
        }
        if let Some(class) = ancestor.as_class_node() {
            let in_superclass = class.superclass().is_some_and(|superclass| {
                let superclass = superclass.location();
                superclass.start_offset() <= location.start_offset()
                    && location.end_offset() <= superclass.end_offset()
            });
            if !in_superclass {
                return;
            }
        }
    }
    let delimiter = path.delimiter_loc();
    context.remove("Remove redundant `::`.", &delimiter, &delimiter);
}
