use super::*;

define_cops! {
    MixinGrouping => "Style/MixinGrouping" => any_node(mixin_grouping),
}

struct MixinCall<'pr> {
    call: CallNode<'pr>,
    method: String,
    arguments: Vec<String>,
    statement_index: usize,
}

fn mixin_grouping(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let body = if let Some(class) = node.as_class_node() {
        class.body()
    } else if let Some(module) = node.as_module_node() {
        module.body()
    } else {
        return;
    };
    let Some(statements) = body.and_then(|body| body.as_statements_node()) else {
        return;
    };
    let calls = collect_mixin_calls(&statements, context.source_file());
    let style = context.policy().enforced_style("separated").to_string();
    let offending = if style == "grouped" {
        calls
            .iter()
            .filter(|call| {
                calls
                    .iter()
                    .filter(|other| other.method == call.method)
                    .count()
                    > 1
            })
            .collect::<Vec<_>>()
    } else {
        calls
            .iter()
            .filter(|call| call.arguments.len() > 1)
            .collect::<Vec<_>>()
    };
    if offending.is_empty() {
        return;
    }

    let (correction_range, replacement) = if style == "grouped" {
        grouped_correction(context.source(), &statements, &calls)
    } else {
        separated_correction(context.source(), &statements, &calls)
    };
    for call in offending {
        let message = if style == "grouped" {
            format!("Put `{}` mixins in a single statement.", call.method)
        } else {
            format!("Put `{}` mixins in separate statements.", call.method)
        };
        context.replace(
            message,
            call.call.location(),
            correction_range.clone(),
            replacement.clone(),
        );
    }
}

fn collect_mixin_calls<'pr>(
    statements: &ruby_prism::StatementsNode<'pr>,
    file: SourceFile<'pr>,
) -> Vec<MixinCall<'pr>> {
    statements
        .body()
        .iter()
        .enumerate()
        .filter_map(|(statement_index, statement)| {
            let call = statement.as_call_node()?;
            if call.receiver().is_some()
                || !matches!(call_name(&call), b"include" | b"extend" | b"prepend")
            {
                return None;
            }
            let arguments = call
                .arguments()?
                .arguments()
                .iter()
                .map(|argument| file.node(&argument).to_string())
                .collect::<Vec<_>>();
            (!arguments.is_empty()).then(|| MixinCall {
                method: String::from_utf8_lossy(call_name(&call)).to_string(),
                call,
                arguments,
                statement_index,
            })
        })
        .collect()
}

fn separated_correction(
    source: &str,
    body: &ruby_prism::StatementsNode<'_>,
    calls: &[MixinCall<'_>],
) -> (std::ops::Range<usize>, String) {
    let file = SourceFile::new(source);
    let edits = calls
        .iter()
        .filter(|call| call.arguments.len() > 1)
        .map(|call| {
            let indentation = file.indentation_text(call.call.location().start_offset());
            let replacement = call
                .arguments
                .iter()
                .rev()
                .map(|argument| format!("{} {argument}", call.method))
                .collect::<Vec<_>>()
                .join(&format!("\n{indentation}"));
            SourceEdit::replace(
                call.call.location().start_offset()..call.call.location().end_offset(),
                replacement,
            )
        })
        .collect();
    apply_body_edits(source, body, edits)
}

fn grouped_correction(
    source: &str,
    body: &ruby_prism::StatementsNode<'_>,
    calls: &[MixinCall<'_>],
) -> (std::ops::Range<usize>, String) {
    let mut edits = Vec::new();
    for method in ["include", "extend", "prepend"] {
        let group = calls
            .iter()
            .filter(|call| call.method == method)
            .collect::<Vec<_>>();
        if group.len() < 2 {
            continue;
        }
        let first = group[0];
        let arguments = group
            .iter()
            .rev()
            .flat_map(|call| call.arguments.iter())
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        edits.push(SourceEdit::replace(
            first.call.location().start_offset()..first.call.location().end_offset(),
            format!("{method} {arguments}"),
        ));
        for call in &group[1..] {
            let contiguous = (first.statement_index..call.statement_index).all(|index| {
                calls.iter().any(|candidate| {
                    candidate.statement_index == index && candidate.method == method
                })
            });
            let range = if contiguous {
                SourceFile::new(source).full_line_range(
                    call.call.location().start_offset()..call.call.location().end_offset(),
                )
            } else {
                call.call.location().start_offset()..call.call.location().end_offset()
            };
            edits.push(SourceEdit::remove(range));
        }
    }
    apply_body_edits(source, body, edits)
}

fn apply_body_edits(
    source: &str,
    body: &ruby_prism::StatementsNode<'_>,
    edits: Vec<SourceEdit>,
) -> (std::ops::Range<usize>, String) {
    let file = SourceFile::new(source);
    let location = body.location();
    let container = file.full_line_range(location.start_offset()..location.end_offset());
    let replacement = file
        .rewrite(container.clone(), edits)
        .unwrap_or_else(|| source[container.clone()].to_string());
    (container, replacement)
}
