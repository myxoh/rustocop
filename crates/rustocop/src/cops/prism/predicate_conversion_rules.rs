use super::*;

define_cops! {
    BitwisePredicate => "Style/BitwisePredicate" => compatibility_prism_call(bitwise_predicate),
    ComparableBetween => "Style/ComparableBetween" => compatibility_prism_node(as_and_node, comparable_between),
}

fn min_max_comparison(node: &ruby_prism::IfNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(comparison) = unwrapped_call(node.predicate()) else {
        return;
    };
    if !matches!(call_name(&comparison), b">" | b">=" | b"<" | b"<=") {
        return;
    }
    let (Some(left), Some(right)) = (comparison.receiver(), only_argument(&comparison)) else {
        return;
    };
    let Some(truthy) = node
        .statements()
        .and_then(|statements| only_statement_in(&statements))
    else {
        return;
    };
    let Some(else_clause) = node.subsequent().and_then(|node| node.as_else_node()) else {
        return;
    };
    let Some(falsey) = else_clause
        .statements()
        .and_then(|statements| only_statement_in(&statements))
    else {
        return;
    };
    let file = context.source_file();
    let left_source = file.node(&left);
    let right_source = file.node(&right);
    let truthy_source = file.node(&truthy);
    let falsey_source = file.node(&falsey);
    let selects_left = truthy_source == left_source && falsey_source == right_source;
    let selects_right = truthy_source == right_source && falsey_source == left_source;
    if !selects_left && !selects_right {
        return;
    }
    let maximum = matches!(call_name(&comparison), b">" | b">=") && selects_left
        || matches!(call_name(&comparison), b"<" | b"<=") && selects_right;
    let method = if maximum { "max" } else { "min" };
    let preferred = format!("[{left_source}, {right_source}].{method}");
    let message = format!("Use `{preferred}` instead.");
    let elsif = node
        .if_keyword_loc()
        .is_some_and(|keyword| keyword.as_slice() == b"elsif");
    if elsif {
        let start = node.if_keyword_loc().unwrap().start_offset();
        let end = falsey.location().end_offset();
        let line = file.line_range(start);
        let indentation = &context.source()[line.start..start];
        context.replace(
            message,
            start..end,
            start..end,
            format!("else\n{indentation}  {preferred}"),
        );
    } else {
        context.replace(message, node.location(), node.location(), preferred);
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum BoundKind {
    Minimum,
    Maximum,
}

struct BoundCandidate {
    subject: String,
    kind: BoundKind,
    bound: String,
}

fn comparable_between(node: &ruby_prism::AndNode<'_>, context: &mut CopContext<'_, '_>) {
    let left = comparison_candidates(&node.left(), context);
    let right = comparison_candidates(&node.right(), context);
    let pair = left.iter().find_map(|left| {
        right.iter().find_map(|right| {
            (left.subject == right.subject && left.kind != right.kind).then_some((left, right))
        })
    });
    let Some((first, second)) = pair else {
        return;
    };
    let (minimum, maximum) = if first.kind == BoundKind::Minimum {
        (&first.bound, &second.bound)
    } else {
        (&second.bound, &first.bound)
    };
    let preferred = format!("{}.between?({minimum}, {maximum})", first.subject);
    context.replace_node(
        &node.as_node(),
        format!("Prefer `{preferred}` over logical comparison."),
        preferred,
    );
}

fn comparison_candidates(node: &Node<'_>, context: &CopContext<'_, '_>) -> Vec<BoundCandidate> {
    let Some(call) = node.as_call_node() else {
        return Vec::new();
    };
    if !matches!(call_name(&call), b">=" | b"<=") {
        return Vec::new();
    }
    let (Some(left), Some(right)) = (call.receiver(), only_argument(&call)) else {
        return Vec::new();
    };
    let left = context.source_file().node(&left).to_string();
    let right = context.source_file().node(&right).to_string();
    if call_name(&call) == b">=" {
        vec![
            BoundCandidate {
                subject: left.clone(),
                kind: BoundKind::Minimum,
                bound: right.clone(),
            },
            BoundCandidate {
                subject: right,
                kind: BoundKind::Maximum,
                bound: left,
            },
        ]
    } else {
        vec![
            BoundCandidate {
                subject: left.clone(),
                kind: BoundKind::Maximum,
                bound: right.clone(),
            },
            BoundCandidate {
                subject: right,
                kind: BoundKind::Minimum,
                bound: left,
            },
        ]
    }
}

fn even_odd(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !matches!(call_name(node), b"==" | b"!=") {
        return;
    }
    let (Some(receiver), Some(expected)) = (node.receiver(), only_argument(node)) else {
        return;
    };
    let expected = integer_value(&expected);
    if !matches!(expected, Some(0 | 1)) {
        return;
    }
    let Some(modulo) = unwrapped_call(receiver) else {
        return;
    };
    if call_name(&modulo) != b"%"
        || only_argument(&modulo).as_ref().and_then(integer_value) != Some(2)
    {
        return;
    }
    let Some(value) = modulo.receiver() else {
        return;
    };
    let odd = call_name(node) == b"==" && expected == Some(1)
        || call_name(node) == b"!=" && expected == Some(0);
    let predicate = if odd { "odd?" } else { "even?" };
    let replacement = format!("{}.{predicate}", context.source_file().node(&value));
    context.replace_call(
        node,
        format!("Replace with `Integer#{predicate}`."),
        replacement,
    );
}

fn integer_value(node: &Node<'_>) -> Option<i32> {
    let integer = node.as_integer_node()?;
    TryInto::<i32>::try_into(integer.value()).ok()
}

fn unwrapped_call(node: Node<'_>) -> Option<CallNode<'_>> {
    if let Some(call) = node.as_call_node() {
        return Some(call);
    }
    let body = node.as_parentheses_node()?.body()?;
    let statements = body.as_statements_node()?;
    (statements.body().len() == 1)
        .then(|| statements.body().first())
        .flatten()?
        .as_call_node()
}

fn bitwise_predicate(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(2, 5) {
        return;
    }
    if !matches!(call_name(node), b"!=" | b"==" | b">" | b">=" | b"positive?" | b"zero?") {
        return;
    }
    let Some(parentheses) = node.receiver().and_then(|receiver| receiver.as_parentheses_node()) else {
        return;
    };
    let Some(bit_operation) = parentheses
        .body()
        .and_then(|body| body.as_statements_node())
        .and_then(|statements| only_statement_in(&statements))
        .and_then(|statement| statement.as_call_node())
    else {
        return;
    };
    if call_name(&bit_operation) != b"&" {
        return;
    }
    let (Some(lhs), Some(rhs)) = (bit_operation.receiver(), only_argument(&bit_operation)) else {
        return;
    };

    let file = context.source_file();
    let lhs_source = file.node(&lhs);
    let rhs_source = file.node(&rhs);
    let argument = only_argument(node);
    let argument_source = argument.as_ref().map(|argument| file.node(argument));
    let method = match call_name(node) {
        b"positive?" if argument_count(node) == 0 => "anybits?",
        b">" if argument_source == Some("0") => "anybits?",
        b">=" if argument_source == Some("1") => "anybits?",
        b"!=" if argument_source == Some("0") => "anybits?",
        b"zero?" if argument_count(node) == 0 => "nobits?",
        b"==" if argument_source == Some("0") => "nobits?",
        b"==" if argument_source == Some(rhs_source) || argument_source == Some(lhs_source) => {
            "allbits?"
        }
        _ => return,
    };
    let preferred = if method == "allbits?" && argument_source == Some(lhs_source) {
        format!("{rhs_source}.allbits?({lhs_source})")
    } else {
        format!("{lhs_source}.{method}({rhs_source})")
    };
    context.replace_call(
        node,
        format!("Replace with `{preferred}` for comparison with bit flags."),
        preferred,
    );
}

fn dir_empty(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(2, 4) {
        return;
    }
    let (enumeration, argument, negative) = match call_name(node) {
        b"empty?" | b"none?" if argument_count(node) == 0 => {
            let Some(enumeration) = node.receiver().and_then(|receiver| receiver.as_call_node())
            else {
                return;
            };
            let expected = if call_name(node) == b"empty?" {
                b"children".as_slice()
            } else {
                b"each_child".as_slice()
            };
            if call_name(&enumeration) != expected {
                return;
            }
            let Some(argument) = only_argument(&enumeration) else {
                return;
            };
            (enumeration, argument, false)
        }
        b"==" | b"!=" | b">" if argument_count(node) == 1 => {
            let Some(size) = node.receiver().and_then(|receiver| receiver.as_call_node()) else {
                return;
            };
            if call_name(&size) != b"size" || argument_count(&size) != 0 {
                return;
            }
            let Some(enumeration) = size.receiver().and_then(|receiver| receiver.as_call_node())
            else {
                return;
            };
            let Some(expected_size) = only_argument(node).as_ref().and_then(integer_value) else {
                return;
            };
            let eligible = call_name(&enumeration) == b"entries" && expected_size == 2
                || call_name(&enumeration) == b"children" && expected_size == 0;
            if !eligible {
                return;
            }
            let Some(argument) = only_argument(&enumeration) else {
                return;
            };
            (
                enumeration,
                argument,
                matches!(call_name(node), b"!=" | b">"),
            )
        }
        _ => return,
    };
    let Some(dir) = enumeration.receiver() else {
        return;
    };
    if !node_is_root_constant(&dir, b"Dir") {
        return;
    }
    let file = context.source_file();
    let replacement = format!(
        "{}{}.empty?({})",
        if negative { "!" } else { "" },
        file.node(&dir),
        file.node(&argument)
    );
    context.replace_call(node, format!("Use `{replacement}` instead."), replacement);
}
