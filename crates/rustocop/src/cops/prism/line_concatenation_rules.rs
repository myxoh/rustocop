use super::*;

define_cops! {
    LineEndConcatenation => "Style/LineEndConcatenation" => call(line_end_concatenation),
}

fn line_end_concatenation(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let operator = call_name(node);
    if !matches!(operator, b"+" | b"<<") {
        return;
    }
    let Some(right) = only_argument(node) else {
        return;
    };
    let Some(selector) = node.message_loc() else {
        return;
    };
    let before = context.source()[..selector.start_offset()].trim_end();
    if !before.ends_with('"') && !before.ends_with('\'') {
        return;
    }
    let gap = &context.source()[selector.end_offset()..right.location().start_offset()];
    if !gap.contains('\n') || gap.contains('#') || gap.matches('\n').count() != 1 {
        return;
    }
    let line_end = context.source()[selector.end_offset()..]
        .find('\n')
        .map_or(context.source().len(), |relative| {
            selector.end_offset() + relative
        });
    if !next_line_starts_string(context.source(), line_end) {
        return;
    }
    let trailing = context.source()[selector.end_offset()..line_end].trim();
    if !trailing.is_empty() && trailing != "\\" {
        return;
    }

    let operator = String::from_utf8_lossy(operator);
    context.replace(
        format!("Use `\\` instead of `{operator}` to concatenate multiline strings."),
        selector.start_offset()..selector.end_offset(),
        selector.start_offset()..line_end,
        "\\",
    );
}

fn next_line_starts_string(source: &str, line_end: usize) -> bool {
    let Some(next_end) = source[line_end + 1..].find('\n') else {
        return false;
    };
    let line = source[line_end + 1..line_end + 1 + next_end].trim_start();
    let Some(quote @ (b'\'' | b'"')) = line.as_bytes().first().copied() else {
        return false;
    };
    let Some(closing) = line.as_bytes().iter().rposition(|byte| *byte == quote) else {
        return false;
    };
    if closing == 0 {
        return false;
    }
    let trailing = line[closing + 1..].trim_start();
    trailing.is_empty()
        || trailing.starts_with('+')
        || trailing.starts_with("<<")
        || trailing.starts_with([')', ']', '}', ',', ';'])
}
