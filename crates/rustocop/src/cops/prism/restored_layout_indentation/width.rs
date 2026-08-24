fn indentation_width(
    statements: &ruby_prism::StatementsNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let Some(parent) = context.parent() else { return };
    let file = context.source_file();
    let Some(base_offset) = indentation_base_offset(parent, file) else {
        return;
    };
    if file.line(base_offset).trim_start().starts_with("ensure")
        && ensure_keyword_belongs_to_block(file, base_offset)
    {
        return;
    }
    let width = context.config_usize("Width", 2);
    if allowed_indentation_base(context, file, base_offset) {
        return;
    }
    let using_tabs = context
        .related_config_value("Layout/IndentationStyle", "EnforcedStyle")
        == Some("tabs");
    let base_column = indentation_base_column(context, parent, file, base_offset, using_tabs, width);
    let mut children = statements.body().iter().collect::<Vec<_>>();
    if (parent.as_if_node().is_some()
        || parent.as_unless_node().is_some()
        || parent.as_while_node().is_some()
        || parent.as_until_node().is_some())
        && children
        .first()
        .is_some_and(|child| base_offset > child.location().start_offset())
    {
        return;
    }
    let member_container = parent.as_class_node().is_some()
        || parent.as_module_node().is_some()
        || parent.as_singleton_class_node().is_some();
    let internal_style = context
        .related_config_value("Layout/IndentationConsistency", "EnforcedStyle")
        == Some("indented_internal_methods");
    if member_container
        && children.first().is_some_and(|child| {
            file.same_line(base_offset, child.location().start_offset())
        })
    {
        return;
    }
    if !member_container || internal_style {
        children.truncate(1);
    }
    let access_outdent = context
        .related_config_value("Layout/AccessModifierIndentation", "EnforcedStyle")
        == Some("outdent");
    let structural_branch = parent.as_rescue_node().is_some()
        || parent.as_else_node().is_some()
        || parent.as_ensure_node().is_some();
    let block_container = parent.as_block_node().is_some() || parent.as_lambda_node().is_some();
    for (index, child) in children.iter().enumerate() {
        if bare_access_modifier(child)
            && (block_container || index != 0 || access_outdent)
        {
            continue;
        }
        if access_modifier_declaration(child) {
            continue;
        }
        if child.as_call_node().is_some_and(|call| {
            call.receiver().is_none() && call.name().as_slice() == b"module_function"
        }) {
            continue;
        }
        let location = child.location();
        if file.same_line(base_offset, location.start_offset()) {
            continue;
        }
        let after_ensure = follows_keyword(file, location.start_offset(), "ensure");
        if inside_block_ensure(context, location.start_offset()) {
            continue;
        }
        let indentation = file.indentation(location.start_offset());
        if indentation.end != location.start_offset() {
            continue;
        }
        let actual_column = if using_tabs {
            visual_indentation(file.indentation_text(location.start_offset()), width)
        } else {
            file.column(location.start_offset())
        };
        let actual = actual_column as isize - base_column as isize;
        if actual == width as isize {
            continue;
        }
        let message = if using_tabs {
            format!(
                "Use 1 (not {}) tabs for indentation.",
                actual.max(0) as usize / width.max(1)
            )
        } else {
            format!("Use {width} (not {actual}) spaces for indentation.")
        };
        let branch_parent = (structural_branch || after_ensure)
            && enclosing_scope_has_width_offense(context, width);
        align_node_with_indentation_offense(
            context,
            child,
            base_column + width,
            &message,
            using_tabs,
            width,
            branch_parent,
        );
    }
    if internal_style {
        let internal_children = statements.body().iter().collect::<Vec<_>>();
        check_internal_method_indentation(context, &internal_children, width, using_tabs);
    }
}

fn indentation_base_offset(parent: &Node<'_>, file: SourceFile<'_>) -> Option<usize> {
    if let Some(node) = parent.as_if_node() {
        return node.if_keyword_loc().and_then(|location| {
            matches!(file.at(&location), "if" | "elsif").then_some(location.start_offset())
        });
    }
    if let Some(node) = parent.as_unless_node() {
        return Some(node.keyword_loc().start_offset());
    }
    if let Some(node) = parent.as_else_node() {
        let location = node.else_keyword_loc();
        return (file.at(&location) != ":").then_some(location.start_offset());
    }
    if let Some(node) = parent.as_while_node() {
        return Some(node.keyword_loc().start_offset());
    }
    if let Some(node) = parent.as_until_node() {
        return Some(node.keyword_loc().start_offset());
    }
    if let Some(node) = parent.as_for_node() {
        return Some(node.for_keyword_loc().start_offset());
    }
    if let Some(node) = parent.as_def_node() {
        return Some(node.def_keyword_loc().start_offset());
    }
    if let Some(node) = parent.as_class_node() {
        return Some(node.class_keyword_loc().start_offset());
    }
    if let Some(node) = parent.as_module_node() {
        return Some(node.module_keyword_loc().start_offset());
    }
    if let Some(node) = parent.as_singleton_class_node() {
        return Some(node.class_keyword_loc().start_offset());
    }
    if let Some(node) = parent.as_block_node() {
        let closing = node.closing_loc();
        return (file.indentation(closing.start_offset()).end == closing.start_offset())
            .then_some(closing.start_offset());
    }
    if let Some(node) = parent.as_lambda_node() {
        let closing = node.closing_loc();
        return (file.indentation(closing.start_offset()).end == closing.start_offset())
            .then_some(closing.start_offset());
    }
    if let Some(node) = parent.as_begin_node() {
        return node.end_keyword_loc().map(|location| location.start_offset());
    }
    if let Some(node) = parent.as_rescue_node() {
        return Some(node.keyword_loc().start_offset());
    }
    if let Some(node) = parent.as_ensure_node() {
        return Some(node.ensure_keyword_loc().start_offset());
    }
    if let Some(node) = parent.as_when_node() {
        return Some(node.keyword_loc().start_offset());
    }
    if let Some(node) = parent.as_in_node() {
        return Some(node.in_loc().start_offset());
    }
    None
}

fn configured_width(context: &CopContext<'_, '_>) -> usize {
    context
        .config_value("IndentationWidth")
        .and_then(|value| value.parse().ok())
        .or_else(|| {
            context
                .related_config_value("Layout/IndentationWidth", "Width")
                .and_then(|value| value.parse().ok())
        })
        .unwrap_or(2)
}

fn display_column(file: SourceFile<'_>, offset: usize) -> usize {
    unicode_width::UnicodeWidthStr::width(
        file.slice(file.line_start(offset)..offset).unwrap_or_default(),
    )
}

fn visual_indentation(indentation: &str, width: usize) -> usize {
    indentation
        .chars()
        .map(|character| if character == '\t' { width } else { 1 })
        .sum()
}

fn align_node(
    context: &mut CopContext<'_, '_>,
    node: &Node<'_>,
    expected: usize,
    message: &str,
    uncorrectable: bool,
) {
    let file = context.source_file();
    let location = node.location();
    if uncorrectable || contains_block_comment(file.at(&location)) {
        context.report(message, &location);
        return;
    }
    let actual = file.column(location.start_offset());
    let delta = expected as isize - actual as isize;
    let edits = shifted_indentation_edits(file, &location, delta, false, 1, false);
    context.add_offense(&location, message, |corrector| {
        for (range, replacement) in edits {
            corrector.replace(range, replacement);
        }
    });
}

fn align_node_with_indentation_offense(
    context: &mut CopContext<'_, '_>,
    node: &Node<'_>,
    expected: usize,
    message: &str,
    using_tabs: bool,
    width: usize,
    uncorrectable: bool,
) {
    let file = context.source_file();
    let location = node.location();
    let actual_column = if using_tabs {
        visual_indentation(file.indentation_text(location.start_offset()), width)
    } else {
        file.column(location.start_offset())
    };
    let base_column = expected.saturating_sub(width);
    let actual = actual_column as isize - base_column as isize;
    let offense = if actual == 0
        && (node.as_def_node().is_some()
            || context
                .parent()
                .is_some_and(|parent| {
                    parent.as_def_node().is_some()
                        || parent.as_block_node().is_some()
                        || parent.as_lambda_node().is_some()
                        || parent.as_class_node().is_some()
                        || parent.as_module_node().is_some()
                        || parent.as_singleton_class_node().is_some()
                }))
        && !message.contains("indented_internal_methods")
        && context.config_value("EnforcedStyleAlignWith") != Some("relative_to_receiver")
        && location.start_offset() > file.line_start(location.start_offset())
    {
        location.start_offset()..location.start_offset() - 1
    } else {
        indentation_offense_range(file, location.start_offset(), actual)
    };
    if using_tabs
        || uncorrectable
        || contains_block_comment(file.at(&location))
        || ancestor_contains_block_comment(context)
    {
        context.report(message, offense);
        return;
    }
    let delta = expected as isize - actual_column as isize;
    let mut edits = shifted_indentation_edits(file, &location, delta, using_tabs, width, true);
    edits.extend(branch_keyword_edits(
        context,
        &location,
        actual_column,
        delta,
        using_tabs,
        width,
    ));
    let edits = consolidate_branch_correction(context, edits);
    context.add_offense(offense, message, |corrector| {
        for (range, replacement) in edits {
            corrector.replace(range, replacement);
        }
    });
}

fn shifted_indentation_edits(
    file: SourceFile<'_>,
    location: &ruby_prism::Location<'_>,
    delta: isize,
    using_tabs: bool,
    width: usize,
    branch_aware: bool,
) -> Vec<(std::ops::Range<usize>, String)> {
    let mut edits = Vec::new();
    let mut line_start = file.line_start(location.start_offset());
    let heredocs = file.heredoc_ranges();
    let first_indentation = visual_indentation(file.indentation_text(line_start), width);
    let mut after_ensure = false;
    while line_start < location.end_offset() {
        let indentation = file.indentation(line_start);
        let trimmed = file.line(line_start).trim_start();
        let branch_keyword = ["rescue", "else", "ensure"].iter().any(|keyword| {
            trimmed == *keyword
                || trimmed
                    .strip_prefix(keyword)
                    .is_some_and(|rest| rest.starts_with(char::is_whitespace))
        });
        let ensure_keyword = trimmed == "ensure"
            || trimmed
                .strip_prefix("ensure")
                .is_some_and(|rest| rest.starts_with(char::is_whitespace));
        let current_indentation = visual_indentation(
            file.slice(indentation.clone()).unwrap_or_default(),
            width,
        );
        let skip_branch_keyword = branch_aware
            && branch_keyword
            && current_indentation != first_indentation;
        let skip_after_ensure = branch_aware && after_ensure;
        if !heredocs
            .iter()
            .any(|range| range.start <= line_start && line_start < range.end)
            && !skip_branch_keyword
            && !skip_after_ensure
        {
            let current = file.slice(indentation.clone()).unwrap_or_default();
            let visual = visual_indentation(current, width);
            let target = (visual as isize + delta).max(0) as usize;
            let replacement = if using_tabs {
                "\t".repeat(target / width.max(1))
                    + &" ".repeat(target % width.max(1))
            } else {
                shift_space_indentation(current, delta)
            };
            edits.push((indentation, replacement));
        }
        after_ensure |= branch_aware && ensure_keyword;
        let line_end = file.line_end(line_start);
        if line_end >= file.as_str().len() {
            break;
        }
        line_start = line_end + if file.as_str().as_bytes().get(line_end) == Some(&b'\r') { 2 } else { 1 };
    }
    edits
}

fn contains_block_comment(source: &str) -> bool {
    source.lines().any(|line| line.starts_with("=begin") || line.starts_with("=end"))
}

fn ancestor_contains_block_comment(context: &CopContext<'_, '_>) -> bool {
    let file = context.source_file();
    context
        .ancestors()
        .iter()
        .any(|ancestor| contains_block_comment(file.node(ancestor)))
}

fn shift_space_indentation(current: &str, delta: isize) -> String {
    if delta >= 0 {
        format!("{current}{}", " ".repeat(delta as usize))
    } else {
        let keep = current.chars().count().saturating_sub((-delta) as usize);
        current.chars().take(keep).collect()
    }
}
