use super::*;

define_cops! {
    SingleLineDoEndBlock => "Style/SingleLineDoEndBlock" => any_node(single_line_do_end_block),
}

fn single_line_do_end_block(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(block) = node.as_block_node() {
        inspect_block(&block, context);
    } else if let Some(lambda) = node.as_lambda_node() {
        inspect_lambda(&lambda, context);
    }
}

fn inspect_block(node: &ruby_prism::BlockNode<'_>, context: &mut CopContext<'_, '_>) {
    let (opening, closing) = (node.opening_loc(), node.closing_loc());
    let file = context.source_file();
    if file.at(&opening) != "do" || !file.same_line(opening.start_offset(), closing.start_offset())
    {
        return;
    }
    let offense = context
        .ancestors()
        .iter()
        .rev()
        .find(|ancestor| ancestor.as_call_node().is_some() || ancestor.as_lambda_node().is_some())
        .map_or_else(
            || node.location().start_offset()..node.location().end_offset(),
            |ancestor| ancestor.location().start_offset()..ancestor.location().end_offset(),
        );
    if redundant_line_break_prefers_single_line(context, offense.end - offense.start) {
        return;
    }
    let header_end = node
        .parameters()
        .and_then(|parameters| parameters.as_block_parameters_node())
        .map_or(opening.end_offset(), |parameters| {
            parameters.location().end_offset()
        });
    report_block(closing, header_end, offense, context);
}

fn inspect_lambda(node: &ruby_prism::LambdaNode<'_>, context: &mut CopContext<'_, '_>) {
    let (opening, closing) = (node.opening_loc(), node.closing_loc());
    let file = context.source_file();
    if file.at(&opening) != "do" || !file.same_line(opening.start_offset(), closing.start_offset())
    {
        return;
    }
    let offense = node.location().start_offset()..node.location().end_offset();
    if redundant_line_break_prefers_single_line(context, offense.end - offense.start) {
        return;
    }
    let header_end = opening.end_offset();
    report_block(closing, header_end, offense, context);
}

fn report_block(
    closing: ruby_prism::Location<'_>,
    header_end: usize,
    offense: std::ops::Range<usize>,
    context: &mut CopContext<'_, '_>,
) {
    let source = context.source();
    let between = &source[header_end..closing.start_offset()];
    if let Some(marker) = heredoc_marker(between) {
        let file = context.source_file();
        if let Some((terminator_start, _)) = file
            .lines()
            .find(|(start, line)| *start >= closing.end_offset() && line.trim() == marker)
        {
            let insertion = file.line_range(terminator_start).end;
            context.replace_many(
                "Prefer multiline `do`...`end` block.",
                offense,
                vec![
                    (header_end..header_end, "\n".to_string()),
                    (closing.start_offset()..closing.end_offset(), String::new()),
                    (insertion..insertion, "end\n".to_string()),
                ],
            );
            return;
        }
    }
    context.replace_many(
        "Prefer multiline `do`...`end` block.",
        offense,
        vec![
            (header_end..header_end, "\n".to_string()),
            (
                closing.start_offset()..closing.start_offset(),
                "\n".to_string(),
            ),
        ],
    );
}

fn heredoc_marker(source: &str) -> Option<&str> {
    let start = source.find("<<")? + 2;
    let rest = source[start..]
        .strip_prefix(['-', '~'])
        .unwrap_or(&source[start..]);
    let rest = rest.strip_prefix(['\'', '"', '`']).unwrap_or(rest);
    let length = rest
        .bytes()
        .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        .count();
    (length > 0).then_some(&rest[..length])
}

fn redundant_line_break_prefers_single_line(context: &CopContext<'_, '_>, width: usize) -> bool {
    context.related_config_value("Layout/RedundantLineBreak", "Enabled") == Some("true")
        && context.related_config_value("Layout/RedundantLineBreak", "InspectBlocks")
            == Some("true")
        && width
            <= context
                .related_config_value("Layout/LineLength", "Max")
                .and_then(|value| value.parse().ok())
                .unwrap_or(120)
}
