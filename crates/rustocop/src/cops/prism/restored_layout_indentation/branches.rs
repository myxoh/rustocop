fn indentation_offense_range(
    file: SourceFile<'_>,
    node_start: usize,
    actual: isize,
) -> std::ops::Range<usize> {
    if actual >= 0 {
        let start = file.line_start(node_start);
        let prefix = file.slice(start..node_start).unwrap_or_default();
        let skip = prefix.chars().count().saturating_sub(actual as usize);
        let offense_start = start
            + prefix
                .char_indices()
                .nth(skip)
                .map_or(prefix.len(), |(offset, _)| offset);
        offense_start..node_start
    } else {
        let characters = (-actual) as usize;
        let tail = &file.as_str()[node_start..];
        let end = tail
            .char_indices()
            .nth(characters)
            .map_or(file.as_str().len(), |(offset, _)| node_start + offset);
        node_start..end
    }
}

fn allowed_indentation_base(
    context: &CopContext<'_, '_>,
    file: SourceFile<'_>,
    base_offset: usize,
) -> bool {
    let line = file.line(base_offset);
    context.config_values("AllowedPatterns").iter().any(|pattern| {
        regex::Regex::new(pattern)
            .is_ok_and(|pattern| pattern.is_match(line))
            || pattern.contains("module") && line.trim_start().starts_with("module")
            || pattern.contains("(els)?if")
                && (line.trim_start().starts_with("if ")
                    || line.trim_start().starts_with("elsif "))
                && line.chars().any(char::is_uppercase)
    })
}

fn indentation_base_column(
    context: &CopContext<'_, '_>,
    parent: &Node<'_>,
    file: SourceFile<'_>,
    base_offset: usize,
    using_tabs: bool,
    width: usize,
) -> usize {
    let column = |offset| {
        if using_tabs {
            visual_indentation(
                file.slice(file.line_start(offset)..offset).unwrap_or_default(),
                width,
            )
        } else {
            file.column(offset)
        }
    };
    if parent.as_def_node().is_some()
        && context.related_config_value("Layout/DefEndAlignment", "EnforcedStyleAlignWith")
            == Some("def")
    {
        return column(base_offset);
    }
    if parent.as_block_node().is_some()
        && context.config_value("EnforcedStyleAlignWith") == Some("relative_to_receiver")
    {
        if let Some(call) = context.nearest_call() {
            if let (Some(receiver), Some(operator)) = (call.receiver(), call.call_operator_loc()) {
                if !file.same_line(receiver.location().end_offset(), operator.start_offset()) {
                    return column(operator.start_offset());
                }
            }
            if let (Some(receiver), Some(selector)) = (call.receiver(), call.message_loc()) {
                if !file.same_line(receiver.location().end_offset(), selector.start_offset()) {
                    return column(selector.start_offset());
                }
            }
        }
    }
    if (parent.as_if_node().is_some()
        || parent.as_unless_node().is_some()
        || parent.as_while_node().is_some()
        || parent.as_until_node().is_some())
        && expression_precedes(file, base_offset)
        && context.related_config_value("Layout/EndAlignment", "EnforcedStyleAlignWith")
            .is_none_or(|style| style == "keyword")
    {
        return column(base_offset);
    }
    if (parent.as_class_node().is_some()
        || parent.as_module_node().is_some()
        || parent.as_singleton_class_node().is_some())
        && expression_precedes(file, base_offset)
    {
        return column(base_offset);
    }
    if using_tabs {
        visual_indentation(file.indentation_text(base_offset), width)
    } else {
        file.indentation(base_offset).len()
    }
}

fn expression_precedes(file: SourceFile<'_>, keyword: usize) -> bool {
    file.slice(file.line_start(keyword)..keyword)
        .is_some_and(|prefix| !prefix.trim_start_matches('\u{feff}').trim().is_empty())
}

fn follows_keyword(file: SourceFile<'_>, offset: usize, keyword: &str) -> bool {
    let current = file.line_start(offset);
    file.lines()
        .take_while(|(line_start, _)| *line_start < current)
        .map(|(_, line)| line.trim())
        .filter(|line| !line.is_empty())
        .last()
        .is_some_and(|line| {
            line == keyword
                || line
                    .strip_prefix(keyword)
                    .is_some_and(|rest| rest.starts_with(char::is_whitespace))
        })
}

fn ensure_keyword_belongs_to_block(file: SourceFile<'_>, ensure_offset: usize) -> bool {
    for line in file
        .lines()
        .take_while(|(line_start, _)| *line_start < file.line_start(ensure_offset))
        .map(|(_, line)| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        if line == "begin" || line.starts_with("def ") || line.starts_with("class ") {
            return false;
        }
        if line == "do" || line.ends_with(" do") || line.contains(" do |") {
            return true;
        }
    }
    false
}

fn inside_block_ensure(context: &CopContext<'_, '_>, offset: usize) -> bool {
    let file = context.source_file();
    context.ancestors().iter().rev().any(|ancestor| {
        let Some(block) = ancestor.as_block_node() else {
            return false;
        };
        let closing_indent = file.indentation(block.closing_loc().start_offset()).len();
        file.lines().any(|(line_start, line)| {
            ancestor.location().start_offset() < line_start
                && line_start < offset
                && line.trim() == "ensure"
                && file.indentation(line_start).len() == closing_indent
        })
    })
}

fn branch_keyword_edits(
    context: &CopContext<'_, '_>,
    location: &ruby_prism::Location<'_>,
    original_column: usize,
    delta: isize,
    using_tabs: bool,
    width: usize,
) -> Vec<(std::ops::Range<usize>, String)> {
    if delta == 0 {
        return Vec::new();
    }
    let file = context.source_file();
    let Some(container_end) = context.ancestors().iter().rev().find_map(|ancestor| {
        (ancestor.as_begin_node().is_some() || ancestor.as_block_node().is_some())
            .then(|| ancestor.location().end_offset())
    }) else {
        return Vec::new();
    };
    let mut edits = Vec::new();
    let mut in_branch_body = false;
    for (line_start, line) in file.lines() {
        if line_start <= file.line_start(location.start_offset()) || line_start >= container_end {
            continue;
        }
        let trimmed = line.trim_start();
        if in_branch_body && (trimmed == "end" || trimmed.starts_with("end ")) {
            break;
        }
        let branch = ["rescue", "else", "ensure"].iter().any(|keyword| {
            trimmed == *keyword
                || trimmed
                    .strip_prefix(keyword)
                    .is_some_and(|rest| rest.starts_with(char::is_whitespace))
        });
        let ensure = trimmed == "ensure"
            || trimmed
                .strip_prefix("ensure")
                .is_some_and(|rest| rest.starts_with(char::is_whitespace));
        if !branch && !in_branch_body {
            continue;
        }
        let indentation = file.indentation(line_start);
        let current = file.slice(indentation.clone()).unwrap_or_default();
        let column = if using_tabs {
            visual_indentation(current, width)
        } else {
            current.chars().count()
        };
        if !branch || column == original_column {
            edits.push((indentation, shift_space_indentation(current, delta)));
        }
        if ensure {
            if column == original_column {
                break;
            }
            in_branch_body = true;
            continue;
        }
        in_branch_body |= branch;
    }
    edits
}

fn consolidate_branch_correction(
    context: &CopContext<'_, '_>,
    edits: Vec<(std::ops::Range<usize>, String)>,
) -> Vec<(std::ops::Range<usize>, String)> {
    let file = context.source_file();
    let Some(container) = context.ancestors().iter().rev().find_map(|ancestor| {
        let location = ancestor.location();
        let source = file.at(&location);
        ((ancestor.as_begin_node().is_some() || ancestor.as_block_node().is_some())
            && source.lines().any(|line| {
                matches!(line.trim(), "rescue" | "else" | "ensure")
                    || line.trim_start().starts_with("rescue ")
            }))
        .then(|| location.start_offset()..location.end_offset())
    }) else {
        return edits;
    };
    let source_edits = edits
        .into_iter()
        .map(|(range, replacement)| SourceEdit::replace(range, replacement))
        .collect();
    file.rewrite(container.clone(), source_edits)
        .map_or_else(Vec::new, |replacement| vec![(container, replacement)])
}
