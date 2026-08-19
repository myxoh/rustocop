use super::*;

define_cops! {
    ExpandPathArguments => "Style/ExpandPathArguments" => call(expand_path_arguments),
    RedundantDirGlobSort => "Lint/RedundantDirGlobSort" => call(redundant_dir_glob_sort),
    PercentQLiterals => "Style/PercentQLiterals" => node(as_string_node, percent_q_literals),
    RedundantCurrentDirectoryInPath => "Style/RedundantCurrentDirectoryInPath" => call(redundant_current_directory_in_path),
    SlicingWithRange => "Style/SlicingWithRange" => call(slicing_with_range),
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
        operator.start_offset()..node.location().end_offset()
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
    let relative_start = removed.start_offset() - range_location.start_offset();
    let relative_end = removed.end_offset() - range_location.start_offset();
    let preferred_range = format!(
        "{}{}",
        &range_source[..relative_start],
        &range_source[relative_end..]
    );
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
    let Some(relative) = relative_to_directory(&path) else {
        return false;
    };
    let preferred = if path == "." {
        "expand_path(__FILE__)".to_string()
    } else if relative.is_empty() {
        "expand_path(__dir__)".to_string()
    } else {
        format!("expand_path('{relative}', __dir__)")
    };
    let Some(selector) = node.message_loc() else {
        return false;
    };
    let current = context
        .source_file()
        .slice(selector.start_offset()..node.location().end_offset())
        .unwrap_or_default();
    context.replace(
        format!("Use `{preferred}` instead of `{current}`."),
        &selector,
        selector.start_offset()..node.location().end_offset(),
        preferred,
    );
    true
}

fn relative_to_directory(path: &str) -> Option<String> {
    if path == "." {
        return Some(String::new());
    }
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." if parts.last().is_some_and(|part| *part != "..") => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    (parts.first() == Some(&"..")).then(|| parts[1..].join("/"))
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
    if upper == wants_upper || content.contains('\\') || wants_upper && content.contains("#{") {
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

fn redundant_current_directory_in_path(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !match_call(node)
        .named(b"require_relative")
        .without_receiver()
        .with_arguments()
        .matches()
    {
        return;
    }
    let Some(argument) = first_argument(node) else {
        return;
    };
    if argument.as_string_node().is_none() && argument.as_interpolated_string_node().is_none() {
        return;
    }
    let raw = context.source_file().node(&argument);
    let Some(content_start) = raw.find("./") else {
        return;
    };
    if raw[..content_start]
        .bytes()
        .any(|byte| !matches!(byte, b'\'' | b'"' | b'%' | b'q' | b'Q' | b'(' | b'[' | b'{'))
    {
        return;
    }
    let content = raw.as_bytes();
    let Some(first_prefix) = current_directory_prefix(&content[content_start..]) else {
        return;
    };
    let mut all_prefixes = first_prefix;
    while let Some(length) = current_directory_prefix(&content[content_start + all_prefixes..]) {
        all_prefixes += length;
    }
    let start = argument.location().start_offset() + content_start;
    context.remove(
        "Remove the redundant current directory path.",
        start..start + first_prefix,
        start..start + all_prefixes,
    );
}

fn current_directory_prefix(source: &[u8]) -> Option<usize> {
    if !source.starts_with(b"./") {
        return None;
    }
    Some(1 + source[1..].iter().take_while(|byte| **byte == b'/').count())
}

#[cfg(test)]
mod tests {
    use super::current_directory_prefix;

    #[test]
    fn measures_one_current_directory_component() {
        assert_eq!(current_directory_prefix(b"./path"), Some(2));
        assert_eq!(current_directory_prefix(b".///./../path"), Some(4));
        assert_eq!(current_directory_prefix(b"../path"), None);
    }
}
