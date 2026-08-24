
fn check_internal_method_indentation(
    context: &mut CopContext<'_, '_>,
    children: &[Node<'_>],
    width: usize,
    using_tabs: bool,
) {
    let file = context.source_file();
    let mut modifier_column = None;
    for child in children {
        if bare_access_modifier(child) {
            let call = child.as_call_node().expect("checked access modifier");
            modifier_column = matches!(call.name().as_slice(), b"protected" | b"private")
                .then(|| file.column(child.location().start_offset()));
            continue;
        }
        let Some(base_column) = modifier_column else {
            continue;
        };
        let location = child.location();
        if file.indentation(location.start_offset()).end != location.start_offset() {
            continue;
        }
        let actual_column = file.column(location.start_offset());
        let actual = actual_column as isize - base_column as isize;
        if actual == width as isize {
            continue;
        }
        let message = if using_tabs {
            format!(
                "Use 1 (not {}) tabs for indented_internal_methods indentation.",
                actual.max(0) as usize / width.max(1)
            )
        } else {
            format!(
                "Use {width} (not {actual}) spaces for indented_internal_methods indentation."
            )
        };
        align_node_with_indentation_offense(
            context,
            child,
            base_column + width,
            &message,
            using_tabs,
            width,
            false,
        );
    }
}

fn enclosing_scope_has_width_offense(
    context: &CopContext<'_, '_>,
    width: usize,
) -> bool {
    let file = context.source_file();
    context.ancestors().iter().rev().any(|ancestor| {
        if let Some(begin) = ancestor.as_begin_node() {
            let Some(first) = begin
                .statements()
                .and_then(|statements| statements.body().iter().next())
            else {
                return false;
            };
            let base = begin
                .begin_keyword_loc()
                .map_or_else(|| file.indentation(begin.location().start_offset()).len(), |keyword| {
                    file.indentation(keyword.start_offset()).len()
                });
            return file.column(first.location().start_offset()) != base + width;
        }
        if let Some(definition) = ancestor.as_def_node() {
            let Some(body) = definition.body() else {
                return false;
            };
            let Some(begin) = body.as_begin_node() else {
                return false;
            };
            let Some(first) = begin
                .statements()
                .and_then(|statements| statements.body().iter().next())
            else {
                return false;
            };
            let base = file.indentation(definition.def_keyword_loc().start_offset()).len();
            return file.column(first.location().start_offset()) != base + width;
        }
        false
    })
}

fn nested_alignment_offense(context: &CopContext<'_, '_>, _node: &CallNode<'_>) -> bool {
    context.ancestors().iter().rev().any(|ancestor| {
        ancestor.as_call_node().is_some_and(|call| {
            call.arguments().is_some_and(|arguments| {
                let arguments = arguments.arguments().iter().collect::<Vec<_>>();
                let Some(first) = arguments.first() else {
                    return false;
                };
                let expected = display_column(
                    context.source_file(),
                    first.location().start_offset(),
                );
                arguments.iter().any(|argument| {
                    let location = argument.location();
                    location.start_offset() <= _node.location().start_offset()
                        && _node.location().end_offset() <= location.end_offset()
                        && context.source_file().indentation(location.start_offset()).end
                            == location.start_offset()
                        && display_column(context.source_file(), location.start_offset())
                            != expected
                })
            })
        })
    })
}

fn nested_consistency_offense(
    context: &CopContext<'_, '_>,
    _statements: &ruby_prism::StatementsNode<'_>,
) -> bool {
    let Some(parent) = context.parent() else {
        return false;
    };
    let parent_location = parent.location();
    let file = context.source_file();
    context.ancestors().iter().rev().any(|ancestor| {
        let Some(outer) = ancestor.as_statements_node() else {
            return false;
        };
        let children = outer.body().iter().collect::<Vec<_>>();
        let Some(first) = children.first() else {
            return false;
        };
        let Some(container) = children.iter().find(|child| {
            child.location().start_offset() <= parent_location.start_offset()
                && parent_location.end_offset() <= child.location().end_offset()
        }) else {
            return false;
        };
        container.location().start_offset() != first.location().start_offset()
            && file.indentation(container.location().start_offset()).end
                == container.location().start_offset()
            && display_column(file, container.location().start_offset())
                != display_column(file, first.location().start_offset())
    })
}
