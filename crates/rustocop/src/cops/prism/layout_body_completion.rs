use super::*;

define_cops! {
    EmptyLinesAroundMethodBody => "Layout/EmptyLinesAroundMethodBody" => compatibility_source(empty_method_body),
    EmptyLinesAroundBlockBody => "Layout/EmptyLinesAroundBlockBody" => compatibility_source(empty_block_body),
    EmptyLinesAroundArguments => "Layout/EmptyLinesAroundArguments" => compatibility_source(empty_around_arguments),
    EmptyLinesAroundExceptionHandlingKeywords => "Layout/EmptyLinesAroundExceptionHandlingKeywords" => compatibility_source(empty_exception_keywords),
}

fn empty_begin_body(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    blank_after(
        context,
        &["begin"],
        "Extra empty line detected at `begin` body beginning.",
    );
    blank_before(
        context,
        &["end"],
        "Extra empty line detected at `begin` body end.",
    );
}

fn empty_method_body(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut multiline_definition = false;
    for window in lines.windows(2) {
        let line = window[0].1.trim_start();
        if line.starts_with("def ") && line.contains('(') && !line.contains(')') {
            multiline_definition = true;
        }
        if ((line.starts_with("def ")
            && !line.trim_end().ends_with('(')
            && !window[0].1.contains("; end"))
            || (multiline_definition && line == ")"))
            && window[1].1.is_empty()
        {
            remove_blank(
                context,
                window[1].0,
                "Extra empty line detected at method body beginning.",
            );
        }
        if multiline_definition && line == ")" {
            multiline_definition = false;
        }
    }
    blank_before(
        context,
        &["end"],
        "Extra empty line detected at method body end.",
    );
}


fn empty_block_body(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let require_empty = context.policy().enforced_style("no_empty_lines") == "empty_lines";
    for window in lines.windows(2) {
        let opening = window[0].1.trim_end();
        let empty_body = matches!(window[1].1.trim(), "}" | "end");
        if require_empty
            && !empty_body
            && (opening.ends_with('{') || opening.ends_with(" do"))
            && !window[1].1.is_empty()
        {
            context.insert(
                "Empty line missing at block body beginning.",
                window[1].0..window[1].0,
                window[1].0,
                "\n",
            );
        } else if !require_empty
            && (opening.ends_with('{') || opening.ends_with(" do"))
            && window[1].1.is_empty()
        {
            remove_blank(
                context,
                window[1].0,
                "Extra empty line detected at block body beginning.",
            );
        }
        if require_empty
            && !(window[0].1.trim_end().ends_with('{') || window[0].1.trim_end().ends_with(" do"))
            && !window[0].1.is_empty()
            && matches!(window[1].1.trim(), "}" | "end")
        {
            context.insert(
                "Empty line missing at block body end.",
                window[1].0..window[1].0,
                window[1].0,
                "\n",
            );
        } else if !require_empty
            && window[0].1.is_empty()
            && matches!(window[1].1.trim(), "}" | "end")
        {
            remove_blank(
                context,
                window[0].0,
                "Extra empty line detected at block body end.",
            );
        }
    }
}

fn empty_around_arguments(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut depth = 0_i32;
    let mut block_depth = 0_i32;
    for window in lines.windows(2) {
        let previous_depth = depth;
        depth += window[0].1.matches('(').count() as i32;
        depth -= window[0].1.matches(')').count() as i32;
        let code = window[0].1.trim();
        if code == "end" && block_depth > 0 {
            block_depth -= 1;
        }
        if previous_depth > 0 && block_depth == 0 && code.is_empty() {
            remove_blank(
                context,
                window[0].0,
                "Empty line detected around arguments.",
            );
            continue;
        }
        if code.ends_with(" do") || code.contains(" do |") {
            block_depth += 1;
        }
    }
}

fn empty_exception_keywords(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for (index, (_, line)) in lines.iter().enumerate() {
        let keyword = ["rescue", "ensure", "else"]
            .into_iter()
            .find(|keyword| line.trim_start().starts_with(keyword));
        let Some(keyword) = keyword else { continue };
        if index > 0 && lines[index - 1].1.trim().is_empty() {
            remove_blank(
                context,
                lines[index - 1].0,
                format!("Extra empty line detected before the `{keyword}`."),
            );
        }
        if index + 1 < lines.len() && lines[index + 1].1.trim().is_empty() {
            remove_blank(
                context,
                lines[index + 1].0,
                format!("Extra empty line detected after the `{keyword}`."),
            );
        }
    }
}

fn blank_after(context: &mut CompatibilityCopContext<'_, '_, '_>, keywords: &[&str], message: &str) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if keywords.contains(&window[0].1.trim()) && window[1].1.trim().is_empty() {
            remove_blank(context, window[1].0, message);
        }
    }
}

fn blank_before(context: &mut CompatibilityCopContext<'_, '_, '_>, keywords: &[&str], message: &str) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if window[0].1.trim().is_empty() && keywords.contains(&window[1].1.trim()) {
            remove_blank(context, window[0].0, message);
        }
    }
}

fn remove_blank(context: &mut CompatibilityCopContext<'_, '_, '_>, offset: usize, message: impl Into<String>) {
    let range = context.source_file().line_range(offset);
    context.remove(message, offset..range.end, range);
}
