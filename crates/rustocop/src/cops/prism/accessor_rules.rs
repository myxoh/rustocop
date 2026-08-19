use super::*;

define_cops! {
    BisectedAttrAccessor => "Style/BisectedAttrAccessor" => any_node(bisected_attr_accessor),
}

struct Accessor<'pr> {
    call: CallNode<'pr>,
    visibility: Vec<u8>,
    reader: bool,
    arguments: Vec<Node<'pr>>,
}

struct Offense {
    range: std::ops::Range<usize>,
    message: String,
}

fn bisected_attr_accessor(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let body = if let Some(class) = node.as_class_node() {
        class.body()
    } else if let Some(module) = node.as_module_node() {
        module.body()
    } else if let Some(singleton) = node.as_singleton_class_node() {
        singleton.body()
    } else {
        return;
    };
    let Some(statements) = body.and_then(|body| body.as_statements_node()) else {
        return;
    };
    let accessors = collect_accessors(&statements);
    let common = common_attributes(&accessors, context.source_file());
    if common.is_empty() {
        return;
    }

    let offenses = accessors
        .iter()
        .flat_map(|accessor| {
            accessor.arguments.iter().filter_map(|argument| {
                let key = context.source_file().node(argument);
                common
                    .iter()
                    .any(|(visibility, common)| {
                        visibility == &accessor.visibility && common.iter().any(|name| name == key)
                    })
                    .then(|| Offense {
                        range: argument.location().start_offset()..argument.location().end_offset(),
                        message: format!("Combine both accessors into `attr_accessor {key}`."),
                    })
            })
        })
        .collect::<Vec<_>>();
    let (correction_range, replacement) = corrected_body(
        context.source(),
        &statements,
        &accessors,
        &common,
        context.source_file(),
    );
    for offense in offenses {
        context.replace_indirectly(
            offense.message,
            offense.range,
            correction_range.clone(),
            replacement.clone(),
        );
    }
}

fn collect_accessors<'pr>(statements: &ruby_prism::StatementsNode<'pr>) -> Vec<Accessor<'pr>> {
    let mut visibility = b"public".to_vec();
    let mut accessors = Vec::new();
    for statement in statements.body().iter() {
        let Some(call) = statement.as_call_node() else {
            continue;
        };
        if call.receiver().is_none()
            && argument_count(&call) == 0
            && matches!(call_name(&call), b"public" | b"private" | b"protected")
        {
            visibility = call_name(&call).to_vec();
            continue;
        }
        let reader = match call_name(&call) {
            b"attr" | b"attr_reader" => true,
            b"attr_writer" => false,
            _ => continue,
        };
        let arguments = call
            .arguments()
            .map(|arguments| arguments.arguments().iter().collect())
            .unwrap_or_default();
        accessors.push(Accessor {
            call,
            visibility: visibility.clone(),
            reader,
            arguments,
        });
    }
    accessors
}

fn common_attributes(
    accessors: &[Accessor<'_>],
    file: SourceFile<'_>,
) -> Vec<(Vec<u8>, Vec<String>)> {
    let mut groups: Vec<(Vec<u8>, Vec<String>)> = Vec::new();
    for accessor in accessors {
        let group_index = groups
            .iter()
            .position(|(visibility, _)| visibility == &accessor.visibility)
            .unwrap_or_else(|| {
                groups.push((accessor.visibility.clone(), Vec::new()));
                groups.len() - 1
            });
        let group = &mut groups[group_index];
        for argument in &accessor.arguments {
            let key = file.node(argument);
            let opposite_exists = accessors.iter().any(|other| {
                other.visibility == accessor.visibility
                    && other.reader != accessor.reader
                    && other
                        .arguments
                        .iter()
                        .any(|argument| file.node(argument) == key)
            });
            if opposite_exists && !group.1.iter().any(|existing| existing == key) {
                group.1.push(key.to_string());
            }
        }
    }
    groups.retain(|(_, names)| !names.is_empty());
    groups
}

fn corrected_body(
    source: &str,
    body: &ruby_prism::StatementsNode<'_>,
    accessors: &[Accessor<'_>],
    common: &[(Vec<u8>, Vec<String>)],
    file: SourceFile<'_>,
) -> (std::ops::Range<usize>, String) {
    let mut edits = Vec::new();
    for (visibility, names) in common {
        let relevant = accessors
            .iter()
            .filter(|accessor| {
                &accessor.visibility == visibility
                    && accessor
                        .arguments
                        .iter()
                        .any(|argument| names.iter().any(|name| name == file.node(argument)))
            })
            .collect::<Vec<_>>();
        let Some(first_start) = relevant
            .first()
            .map(|accessor| accessor.call.location().start_offset())
        else {
            continue;
        };
        for accessor in relevant {
            let remaining = accessor
                .arguments
                .iter()
                .map(|argument| file.node(argument))
                .filter(|argument| !names.iter().any(|name| name == argument))
                .collect::<Vec<_>>();
            let location = accessor.call.location();
            if location.start_offset() == first_start {
                let mut replacement = format!("attr_accessor {}", names.join(", "));
                if !remaining.is_empty() {
                    let indentation = file.indentation_text(location.start_offset());
                    replacement.push_str(&format!(
                        "\n{indentation}{} {}",
                        String::from_utf8_lossy(call_name(&accessor.call)),
                        remaining.join(", ")
                    ));
                }
                edits.push(SourceEdit::replace(
                    location.start_offset()..location.end_offset(),
                    replacement,
                ));
            } else if remaining.is_empty() {
                edits.push(SourceEdit::remove(
                    file.full_line_range(location.start_offset()..location.end_offset()),
                ));
            } else {
                edits.push(SourceEdit::replace(
                    location.start_offset()..location.end_offset(),
                    format!(
                        "{} {}",
                        String::from_utf8_lossy(call_name(&accessor.call)),
                        remaining.join(", ")
                    ),
                ));
            }
        }
    }

    let body_location = body.location();
    let container = file.full_line_range(body_location.start_offset()..body_location.end_offset());
    let corrected = file
        .rewrite(container.clone(), edits)
        .unwrap_or_else(|| source[container.clone()].to_string());
    (container, corrected)
}
