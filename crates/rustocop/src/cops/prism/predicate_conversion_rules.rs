use super::*;

define_cops! {
    BitwisePredicate => "Style/BitwisePredicate" => source(bitwise_predicate),
    ComparableBetween => "Style/ComparableBetween" => node(as_and_node, comparable_between),
    DirEmpty => "Style/DirEmpty" => source(dir_empty),
    EvenOdd => "Style/EvenOdd" => call(even_odd),
    MinMaxComparison => "Style/MinMaxComparison" => node(as_if_node, min_max_comparison),
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

fn bitwise_predicate(context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(2, 5) {
        return;
    }
    for (line_start, line) in context.source_file().lines() {
        let indentation = line.len() - line.trim_start().len();
        let expression = line.trim();
        let Some(close) = expression.find(')') else {
            continue;
        };
        let Some(bitwise) = expression
            .strip_prefix('(')
            .and_then(|value| value[..close - 1].split_once(" & "))
        else {
            continue;
        };
        let (left, right) = bitwise;
        let tail = &expression[close + 1..];
        let conversion = match tail {
            ".positive?" | " > 0" | " >= 1" | " != 0" => Some((left, "anybits?", right)),
            ".zero?" | " == 0" => Some((left, "nobits?", right)),
            _ => tail.strip_prefix(" == ").and_then(|compared| {
                if compared == right {
                    Some((left, "allbits?", right))
                } else if compared == left {
                    Some((right, "allbits?", left))
                } else {
                    None
                }
            }),
        };
        let Some((receiver, method, flags)) = conversion else {
            continue;
        };
        let range = line_start + indentation..line_start + indentation + expression.len();
        let replacement = format!("{receiver}.{method}({flags})");
        context.replace(
            format!("Replace with `{replacement}` for comparison with bit flags."),
            range.clone(),
            range,
            replacement,
        );
    }
}

fn dir_empty(context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(2, 4) {
        return;
    }
    let source = context.source();
    let trimmed = source.trim();
    if trimmed.starts_with("Dir.\n") {
        let normalized = trimmed.replace(".\n", ".").replace("  entries", "entries");
        if let Some((argument, negative)) = dir_conversion(&normalized) {
            let replacement = format!("{}Dir.empty?({argument})", if negative { "!" } else { "" });
            context.replace(
                format!("Use `{replacement}` instead."),
                0..trimmed.len(),
                0..trimmed.len(),
                replacement,
            );
        }
        return;
    }
    for (line_start, line) in context.source_file().lines() {
        let indentation = line.len() - line.trim_start().len();
        let expression = line.trim();
        let leading_not = expression.starts_with('!');
        let candidate = expression.strip_prefix('!').unwrap_or(expression);
        let Some((argument, negative)) = dir_conversion(candidate) else {
            continue;
        };
        let replacement = format!("{}Dir.empty?({argument})", if negative { "!" } else { "" });
        let start = line_start + indentation + usize::from(leading_not);
        let end = line_start + indentation + expression.len();
        context.replace(
            format!("Use `{replacement}` instead."),
            start..end,
            start..end,
            replacement,
        );
    }
}

fn dir_conversion(expression: &str) -> Option<(&str, bool)> {
    let open = expression.find('(')?;
    let close = super::source_syntax::matching_delimiter(expression, open, b'(', b')')?;
    let argument = &expression[open + 1..close];
    let prefix = &expression[..open];
    let tail = &expression[close + 1..];
    match (prefix, tail) {
        ("Dir.entries", ".size == 2")
        | ("Dir.children", ".empty?")
        | ("Dir.children", ".size == 0")
        | ("Dir.each_child", ".none?") => Some((argument, false)),
        ("Dir.entries", ".size != 2" | ".size > 2") => Some((argument, true)),
        _ => None,
    }
}
