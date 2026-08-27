use super::*;

define_cops! {
    ComparableClamp => "Style/ComparableClamp" => compatibility_prism_any_node(comparable_clamp),
}

fn comparable_clamp(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(2, 4) {
        return;
    }
    if let Some(condition) = node.as_if_node() {
        inspect_conditional_clamp(&condition, context);
    } else if let Some(call) = node.as_call_node() {
        inspect_array_clamp(&call, context);
    }
}

fn inspect_conditional_clamp(node: &ruby_prism::IfNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(first_value) = only_statement(node.statements()) else {
        return;
    };
    let Some(elsif) = node
        .subsequent()
        .and_then(|subsequent| subsequent.as_if_node())
    else {
        return;
    };
    let Some(second_value) = only_statement(elsif.statements()) else {
        return;
    };
    let Some(final_value) = elsif
        .subsequent()
        .and_then(|subsequent| subsequent.as_else_node())
        .and_then(|clause| only_statement(clause.statements()))
    else {
        return;
    };
    let file = context.source_file();
    let Some(first) = clamp_branch(&node.predicate(), &first_value, file) else {
        return;
    };
    let Some(second) = clamp_branch(&elsif.predicate(), &second_value, file) else {
        return;
    };
    if first.kind == second.kind
        || first.value != second.value
        || file.node(&final_value) != first.value
    {
        return;
    }
    let (low, high) = if first.kind == Bound::Low {
        (first.bound, second.bound)
    } else {
        (second.bound, first.bound)
    };
    let replacement = format!("{}.clamp({low}, {high})", first.value);
    let keyword = node.if_keyword_loc().expect("if/elsif keyword");
    let offense = if file.at(&keyword) == "elsif" {
        keyword.start_offset()..final_value.location().end_offset()
    } else {
        node.location().start_offset()..node.location().end_offset()
    };
    let message = format!("Use `{replacement}` instead of `if/elsif/else`.");
    if file.at(&keyword) == "elsif" {
        let indentation = " ".repeat(file.column(keyword.start_offset()));
        context.replace(
            message,
            offense,
            node.location(),
            format!("else\n{indentation}  {replacement}\n{indentation}end"),
        );
    } else {
        context.replace(message, offense, node.location(), replacement);
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Bound {
    Low,
    High,
}

struct ClampBranch<'a> {
    kind: Bound,
    value: &'a str,
    bound: &'a str,
}

fn clamp_branch<'a>(
    predicate: &Node<'_>,
    result: &Node<'_>,
    file: SourceFile<'a>,
) -> Option<ClampBranch<'a>> {
    let comparison = predicate.as_call_node()?;
    let operator = call_name(&comparison);
    if !matches!(operator, b"<" | b">") {
        return None;
    }
    let left = file.node(&comparison.receiver()?);
    let right_node = only_argument(&comparison)?;
    let right = file.node(&right_node);
    let bound = file.node(result);
    if right == bound {
        Some(ClampBranch {
            kind: if operator == b"<" {
                Bound::Low
            } else {
                Bound::High
            },
            value: left,
            bound,
        })
    } else if left == bound {
        Some(ClampBranch {
            kind: if operator == b">" {
                Bound::Low
            } else {
                Bound::High
            },
            value: right,
            bound,
        })
    } else {
        None
    }
}

fn inspect_array_clamp(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let outer = call_name(node);
    if !matches!(outer, b"min" | b"max") || argument_count(node) != 0 {
        return;
    }
    let Some(array) = node
        .receiver()
        .and_then(|receiver| receiver.as_array_node())
    else {
        return;
    };
    let elements = array.elements().iter().collect::<Vec<_>>();
    if elements.len() != 2 {
        return;
    }
    let nested = elements.iter().find_map(|element| {
        element.as_call_node().filter(|call| {
            matches!(call_name(call), b"min" | b"max")
                && call
                    .receiver()
                    .is_some_and(|receiver| receiver.as_array_node().is_some())
        })
    });
    let Some(nested) = nested else {
        return;
    };
    let complementary = outer == b"min" && call_name(&nested) == b"max"
        || outer == b"max" && call_name(&nested) == b"min";
    let nested_pair = nested
        .receiver()
        .and_then(|receiver| receiver.as_array_node())
        .is_some_and(|array| array.elements().len() == 2);
    if complementary && nested_pair {
        context.report_call(node, "Use `Comparable#clamp` instead.");
    }
}
