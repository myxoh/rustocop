use super::*;

define_cops! {
    ExpandPathArguments => "Style/ExpandPathArguments" => compatibility_prism_call(expand_path_arguments),
    SlicingWithRange => "Style/SlicingWithRange" => compatibility_prism_call(slicing_with_range),
}

fn slicing_with_range(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(2, 6)
        || call_name(node) != b"[]"
        || node.receiver().is_none()
    {
        return;
    }
    let Some(range) = only_argument(node).and_then(|argument| argument.as_range_node()) else {
        return;
    };
    let (Some(left), Some(right)) = (range.left(), range.right()) else {
        return;
    };
    let inclusive = range.operator_loc().as_slice() == b"..";
    let start_zero = integer_source_value(&left, context.source_file()) == Some(0);
    let end_minus_one = integer_source_value(&right, context.source_file()) == Some(-1);
    let end_nil = right.as_nil_node().is_some();
    let start_nil = left.as_nil_node().is_some();
    let bracket = node
        .opening_loc()
        .is_some_and(|opening| opening.as_slice() == b"[");
    let explicit_without_parentheses = node.opening_loc().is_none() && node.message_loc().is_some();
    let useless = start_zero && (inclusive && end_minus_one || end_nil);
    if explicit_without_parentheses && !useless {
        return;
    }
    let remove_right = !start_zero && (inclusive && end_minus_one || end_nil);
    let remove_left = start_nil && context.target_ruby_version().at_least(2, 7);
    if !useless && !remove_right && !remove_left {
        return;
    }
    let offense = if bracket {
        let (Some(opening), Some(closing)) = (node.opening_loc(), node.closing_loc()) else {
            return;
        };
        opening.start_offset()..closing.end_offset()
    } else {
        let Some(operator) = node.call_operator_loc() else {
            return;
        };
        let send_end = node.closing_loc().map_or_else(
            || range.location().end_offset(),
            |closing| closing.end_offset(),
        );
        operator.start_offset()..send_end
    };
    let current = context.source()[offense.clone()].to_string();
    if useless {
        context.remove(
            format!("Remove the useless `{current}`."),
            offense.clone(),
            offense,
        );
        return;
    }
    let range_location = range.location();
    let range_source = context.source_file().at(&range_location);
    let removed = if remove_right {
        right.location()
    } else {
        left.location()
    };
    let preferred_range = if remove_right {
        format!(
            "{}{}",
            context.source_file().node(&left),
            context.source_file().at(&range.operator_loc())
        )
    } else {
        format!(
            "{}{}",
            context.source_file().at(&range.operator_loc()),
            context.source_file().node(&right)
        )
    };
    let (preferred, original) = if bracket {
        (format!("[{preferred_range}]"), format!("[{range_source}]"))
    } else {
        (preferred_range, range_source.to_string())
    };
    context.remove(
        format!("Prefer `{preferred}` over `{original}`."),
        offense,
        &removed,
    );
}

fn integer_source_value(node: &Node<'_>, file: SourceFile<'_>) -> Option<i64> {
    file.node(node).parse().ok()
}

fn expand_path_arguments(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !file_expand_path(node, context) {
        pathname_expand_path(node, context);
    }
}

fn file_expand_path(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) -> bool {
    if !match_call(node)
        .named(b"expand_path")
        .on_root_constant(b"File")
        .with_argument_count(2)
        .matches()
    {
        return false;
    }
    let arguments = node.arguments().expect("two arguments");
    let values = arguments.arguments().iter().collect::<Vec<_>>();
    let Some(path) = values[0].as_string_node() else {
        return false;
    };
    if context.source_file().node(&values[1]) != "__FILE__" {
        return false;
    }
    let path = String::from_utf8_lossy(path.unescaped());
    let normalized_path = path.trim_end_matches('/');
    let components = if normalized_path.is_empty() {
        Vec::new()
    } else {
        normalized_path.split('/').collect::<Vec<_>>()
    };
    let depth = components.iter().filter(|part| **part != ".").count();
    let mut parent_parts = components
        .into_iter()
        .filter(|part| *part != ".")
        .collect::<Vec<_>>();
    if let Some(index) = parent_parts.iter().position(|part| *part == "..") {
        parent_parts.remove(index);
    }
    let parent_path = parent_parts.join("/");
    let preferred_default = if depth == 0 { "__FILE__" } else { "__dir__" };
    let preferred = if parent_path.is_empty() {
        format!("expand_path({preferred_default})")
    } else {
        format!("expand_path('{parent_path}', {preferred_default})")
    };
    let correction = match depth {
        0 => "expand_path(__FILE__)".to_string(),
        1 => "expand_path(__dir__)".to_string(),
        _ => format!("expand_path('{parent_path}', __dir__)"),
    };
    let Some(selector) = node.message_loc() else {
        return false;
    };
    let current = format!("expand_path('{path}', __FILE__)");
    context.replace(
        format!("Use `{preferred}` instead of `{current}`."),
        &selector,
        selector.start_offset()..node.location().end_offset(),
        correction,
    );
    true
}

fn pathname_expand_path(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) -> bool {
    if call_name(node) != b"expand_path" || argument_count(node) != 0 {
        return false;
    }
    let Some(parent) = node.receiver().and_then(|receiver| receiver.as_call_node()) else {
        return false;
    };
    if call_name(&parent) != b"parent" || argument_count(&parent) != 0 {
        return false;
    }
    let Some(pathname) = parent
        .receiver()
        .and_then(|receiver| receiver.as_call_node())
    else {
        return false;
    };
    let valid = if call_name(&pathname) == b"Pathname" && pathname.receiver().is_none() {
        only_argument(&pathname)
            .is_some_and(|argument| context.source_file().node(&argument) == "__FILE__")
    } else {
        match_call(&pathname)
            .named(b"new")
            .on_root_constant(b"Pathname")
            .with_only_argument_matching(|argument| {
                context.source_file().node(argument) == "__FILE__"
            })
            .matches()
    };
    if !valid {
        return false;
    }
    let current = context.source_file().node(&node.as_node()).to_string();
    let corrected = current
        .replacen("__FILE__", "__dir__", 1)
        .replacen(".parent", "", 1);
    let preferred = corrected.strip_prefix("::").unwrap_or(&corrected);
    let current_display = current.strip_prefix("::").unwrap_or(&current);
    context.replace_call(
        node,
        format!("Use `{preferred}` instead of `{current_display}`."),
        corrected,
    );
    true
}

fn redundant_dir_glob_sort(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(3, 0)
        || !match_call(node)
            .named(b"sort")
            .with_receiver()
            .without_arguments()
            .matches()
    {
        return;
    }
    let Some(glob) = node.receiver().and_then(|receiver| receiver.as_call_node()) else {
        return;
    };
    if !matches!(call_name(&glob), b"glob" | b"[]")
        || !root_constant(glob.receiver(), b"Dir")
        || argument_count(&glob) >= 2
        || first_argument(&glob).is_some_and(|argument| argument.as_splat_node().is_some())
    {
        return;
    }
    let (Some(operator), Some(selector)) = (node.call_operator_loc(), node.message_loc()) else {
        return;
    };
    context.remove(
        "Remove redundant `sort`.",
        &selector,
        operator.start_offset()..selector.end_offset(),
    );
}

fn percent_q_literals(node: &ruby_prism::StringNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(opening) = node.opening_loc() else {
        return;
    };
    let opening_source = opening.as_slice();
    if !opening_source.starts_with(b"%q") && !opening_source.starts_with(b"%Q") {
        return;
    }
    let upper = opening_source.starts_with(b"%Q");
    let wants_upper = context.policy().enforced_style("lower_case_q") == "upper_case_q";
    let content = context.source_file().at(&node.content_loc());
    // RuboCop's parser_prism adapter represents multiline percent strings as a
    // `dstr` made up of line-sized `str` children. PercentQLiterals only
    // implements `on_str`, and none of those children owns the `%Q` opening,
    // so the cop deliberately leaves the whole multiline literal alone.
    if upper == wants_upper || content.contains('\n') || wants_upper && content.contains("#{") {
        return;
    }
    let literal = context.source_file().node(&node.as_node());
    let mut corrected = literal.to_string();
    corrected.replace_range(1..2, if wants_upper { "Q" } else { "q" });
    let parsed = ruby_prism::parse(corrected.as_bytes());
    let corrected_value = parsed
        .node()
        .as_program_node()
        .and_then(|program| program.statements().body().first())
        .and_then(|value| value.as_string_node())
        .map(|value| value.unescaped().to_vec());
    if corrected_value.as_deref() != Some(node.unescaped()) {
        return;
    }
    let (message, replacement) = if wants_upper {
        ("Use `%Q` instead of `%q`.", "Q")
    } else {
        (
            "Do not use `%Q` unless interpolation is needed. Use `%q`.",
            "q",
        )
    };
    context.replace(
        message,
        &opening,
        opening.start_offset() + 1..opening.start_offset() + 2,
        replacement,
    );
}
