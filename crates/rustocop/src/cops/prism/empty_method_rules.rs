use super::*;

define_cops! {
    EmptyMethod => "Style/EmptyMethod" => compatibility_prism_node(as_def_node, empty_method),
}

fn empty_method(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.body().is_some() {
        return;
    }
    let location = node.location();
    let file = context.source_file();
    let source = file.at(&location);
    let Some(end) = node.end_keyword_loc() else {
        return;
    };
    let before_end = &context.source()[location.start_offset()..end.start_offset()];
    let through_end_line = &context.source()
        [location.start_offset()..file.line_range(end.start_offset()).end];
    if through_end_line.lines().any(|line| line.contains('#')) {
        return;
    }
    let style = context.policy().enforced_style("compact").to_string();
    let same_line = file.same_line(location.start_offset(), end.start_offset());
    if style == "compact" {
        if same_line {
            return;
        }
        let header = compact_header(before_end);
        let replacement = format!("{header}; end");
        let message = "Put empty method definitions on a single line.";
        let line_length_enabled =
            context.related_config_value("Layout/LineLength", "Enabled") != Some("false");
        let max = context
            .related_config_value("Layout/LineLength", "Max")
            .and_then(|value| value.parse().ok())
            .unwrap_or(120);
        if line_length_enabled && file.column(location.start_offset()) + replacement.len() > max {
            context.report(message, &location);
        } else {
            context.replace(message, &location, &location, replacement);
        }
    } else if same_line {
        let Some(relative_end) = source.rfind("end") else {
            return;
        };
        let header = source[..relative_end]
            .trim_end()
            .trim_end_matches(';')
            .trim_end();
        let indentation = " ".repeat(file.column(location.start_offset()));
        context.replace(
            "Put the `end` of empty method definitions on the next line.",
            &location,
            &location,
            format!("{header}\n{indentation}end"),
        );
    }
}

fn compact_header(source: &str) -> String {
    let mut result = String::new();
    for line in source.lines() {
        let part = line.trim();
        if part.is_empty() {
            continue;
        }
        if !result.is_empty() && !part.starts_with(')') && !result.ends_with('(') {
            result.push(' ');
        }
        result.push_str(part);
    }
    result.trim_end_matches(';').trim_end().to_string()
}
