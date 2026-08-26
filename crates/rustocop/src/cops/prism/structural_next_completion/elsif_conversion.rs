use super::*;

pub(super) fn if_inside_else(context: &mut CopContext<'_, '_>) {
    let parsed = parse(context.source().as_bytes());
    let mut collector = IfInsideElseCollector {
        allow_modifier: context.config_bool("AllowIfModifier", false),
        comments: context.source_file().comment_ranges(),
        offsets: std::collections::HashSet::new(),
    };
    collector.visit(&parsed.node());
    let valid = collector.offsets;
    let mut reported_offsets = std::collections::HashSet::new();
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut reported = false;
    for pair in lines.windows(2) {
        let (else_offset, else_line) = pair[0];
        let (if_offset, if_line) = pair[1];
        if else_line.trim() != "else" || !if_line.trim_start().starts_with("if ") {
            continue;
        }
        let indent = if_line.len() - if_line.trim_start().len();
        let keyword_offset = if_offset + indent;
        if !valid.contains(&keyword_offset) {
            continue;
        }
        let condition = if_line.trim_start()[3..].trim_end();
        if condition.contains(" then ")
            && correct_then_form(context, &lines, else_offset, if_offset, if_line)
        {
            return;
        }
        if condition.ends_with(" then") || condition.contains(" #") {
            continue;
        }
        let nested_index = lines
            .iter()
            .position(|candidate| candidate.0 == if_offset)
            .unwrap_or(0);
        let mut depth = 0usize;
        let mut nested_end = None;
        for (index, (_, candidate)) in lines.iter().enumerate().skip(nested_index) {
            let candidate = candidate.trim();
            if index > nested_index
                && (candidate.starts_with("if ")
                    || candidate.starts_with("unless ")
                    || candidate.starts_with("case ")
                    || candidate.starts_with("begin")
                    || candidate.starts_with("def ")
                    || candidate.starts_with("class ")
                    || candidate.ends_with(" do"))
            {
                depth += 1;
            } else if candidate == "end" {
                if depth == 0 {
                    nested_end = Some(index);
                    break;
                }
                depth -= 1;
            }
        }
        if nested_end.is_some_and(|end| {
            lines[end + 1..]
                .iter()
                .find(|(_, line)| !line.trim().is_empty())
                .is_some_and(|(_, line)| line.trim() != "end")
        }) {
            continue;
        }
        let offense = if_offset + indent..if_offset + indent + 2;
        reported_offsets.insert(keyword_offset);
        let outer_indent = &else_line[..else_line.len() - else_line.trim_start().len()];
        let mut replacement = format!("{outer_indent}elsif {condition}");
        let nested_end = nested_end.expect("checked above");
        let mut nested_else = false;
        let before_else = lines.iter().take(nested_end).skip(nested_index + 1).take_while(|(_, line)| {
            !line.trim_start().starts_with("else") && !line.trim_start().starts_with("elsif")
        }).map(|(_, line)| *line).collect::<Vec<_>>();
        let has_comment = before_else.iter().any(|line| line.trim_start().starts_with('#'));
        let has_expression = before_else.iter().any(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'));
        for (_, body_line) in lines.iter().take(nested_end).skip(nested_index + 1) {
            if body_line.trim_start().starts_with("else") || body_line.trim_start().starts_with("elsif") {
                nested_else = true;
            }
            let dedented = if nested_else || (has_comment && (!body_line.trim_start().starts_with('#') || !has_expression)) {
                body_line
            } else {
                body_line.strip_prefix("  ").unwrap_or(body_line)
            };
            replacement.push('\n');
            replacement.push_str(dedented);
        }
        let correction_end = lines[nested_end].0 + lines[nested_end].1.len();
        if reported {
            context.replace_indirectly(
                "Convert `if` nested inside `else` to `elsif`.",
                offense.clone(),
                offense.clone(),
                &context.source()[offense],
            );
        } else {
            context.replace(
                "Convert `if` nested inside `else` to `elsif`.",
                offense,
                else_offset..correction_end,
                replacement,
            );
            reported = true;
        }
    }
    correct_modifier_form(context, &lines, &valid, &mut reported_offsets);
    for offset in valid.difference(&reported_offsets) {
        context.report(
            "Convert `if` nested inside `else` to `elsif`.",
            *offset..*offset + 2,
        );
    }
}

struct IfInsideElseCollector {
    allow_modifier: bool,
    comments: Vec<std::ops::Range<usize>>,
    offsets: std::collections::HashSet<usize>,
}

impl<'pr> Visit<'pr> for IfInsideElseCollector {
    fn visit_if_node(&mut self, node: &ruby_prism::IfNode<'pr>) {
        if node.if_keyword_loc().is_none() {
            ruby_prism::visit_if_node(self, node);
            return;
        }
        if let Some(else_clause) = node.subsequent().and_then(|branch| branch.as_else_node()) {
            if let Some(nested) = only_statement(else_clause.statements()).and_then(|body| body.as_if_node()) {
                if let Some(keyword) = nested
                    .if_keyword_loc()
                    .filter(|keyword| keyword.as_slice() == b"if")
                {
                    let modifier = nested.end_keyword_loc().is_none();
                    let else_end = else_clause.else_keyword_loc().end_offset();
                    let commented = !modifier
                        && self.comments.iter().any(|comment| {
                            else_end < comment.start && comment.start < keyword.start_offset()
                        });
                    if !(commented || modifier && self.allow_modifier) {
                        self.offsets.insert(keyword.start_offset());
                    }
                }
            }
        }
        ruby_prism::visit_if_node(self, node);
    }
}

fn correct_modifier_form(
    context: &mut CopContext<'_, '_>,
    lines: &[(usize, &str)],
    valid: &std::collections::HashSet<usize>,
    reported: &mut std::collections::HashSet<usize>,
) {
    if context.config_bool("AllowIfModifier", false) {
        return;
    }
    for (else_index, (else_offset, else_line)) in lines.iter().enumerate() {
        if else_line.trim() != "else" {
            continue;
        }
        let Some((body_relative, (body_offset, body_line))) = lines[else_index + 1..]
            .iter()
            .enumerate()
            .find(|(_, (_, line))| {
                !line.trim().is_empty() && !line.trim_start().starts_with('#')
            })
        else {
            continue;
        };
        let body_index = else_index + 1 + body_relative;
        let else_indent = else_line.len() - else_line.trim_start().len();
        let sole_else_expression = lines[body_index + 1..]
            .iter()
            .find(|(_, line)| {
                !line.trim().is_empty() && !line.trim_start().starts_with('#')
            })
            .is_some_and(|(_, line)| {
                line.trim_start().starts_with("end")
                    && line.len() - line.trim_start().len() == else_indent
            });
        if !sole_else_expression {
            continue;
        }
        let Some(if_at) = body_line.find(" if ") else {
            continue;
        };
        let keyword_offset = body_offset + if_at + 1;
        if !valid.contains(&keyword_offset) {
            continue;
        }
        if body_line[..if_at].trim().is_empty() {
            continue;
        }
        let condition = body_line[if_at + 4..]
            .split('#')
            .next()
            .unwrap_or_default()
            .trim();
        let body = body_line[..if_at].trim();
        let trailing_comment = body_line[if_at + 4..].find('#').map(|at| {
            format!(" {}", body_line[if_at + 4 + at..].trim())
        }).unwrap_or_default();
        let body_indent = &body_line[..body_line.len() - body_line.trim_start().len()];
        context.replace_many(
            "Convert `if` nested inside `else` to `elsif`.",
            body_offset + if_at + 1..body_offset + if_at + 3,
            vec![
                (
                    *else_offset..else_offset + else_line.len(),
                    format!(
                        "{}elsif {condition}",
                        &else_line[..else_line.len() - else_line.trim_start().len()]
                    ),
                ),
                (
                    *body_offset..body_offset + body_line.len(),
                    format!("{body_indent}{body}{trailing_comment}"),
                ),
            ],
        );
        reported.insert(keyword_offset);
    }
}

fn correct_then_form(
    context: &mut CopContext<'_, '_>,
    lines: &[(usize, &str)],
    else_offset: usize,
    if_offset: usize,
    if_line: &str,
) -> bool {
    let indent = &if_line[..if_line.len() - if_line.trim_start().len()];
    let nested_index = lines.iter().position(|(offset, _)| *offset == if_offset).unwrap_or(0);
    let inline_end = lines[nested_index..].iter().position(|(_, line)| line.trim_end().ends_with(" end")).map(|at| nested_index + at);
    let Some(end_index) = inline_end else { return false };
    let multiline = end_index > nested_index;
    let mut expanded = String::new();
    for (position, (_, line)) in lines[nested_index..=end_index].iter().enumerate() {
        if position > 0 { expanded.push('\n'); }
        let mut line = line.to_string();
        if !line.trim_start().starts_with("elsif ") {
            line = line.replace(" elsif ", &format!("\n{indent}elsif "));
        }
        line = line.replace(" then ", &format!("\n{indent}"));
        if line.trim_start().starts_with("else ") {
            let leading = &line[..line.len() - line.trim_start().len()];
            line = format!("{leading}else\n{indent}{}", line.trim_start()[5..].trim_start());
        } else if let Some((before, after)) = line.split_once(" else ") {
            line = format!("{before}\n{indent}else\n{indent}{after}");
        }
        if line.ends_with(" end") {
            line.truncate(line.len() - 4);
            line.push('\n');
            line.push_str(if multiline { indent } else { "" });
            line.push_str("end");
        }
        expanded.push_str(&line);
    }
    let offense = if_offset + indent.len()..if_offset + indent.len() + 2;
    let correction_end = lines[end_index].0 + lines[end_index].1.len();
    let mut expanded_lines = expanded.lines();
    let first = expanded_lines
        .next()
        .unwrap_or_default()
        .trim_start()
        .strip_prefix("if ")
        .unwrap_or_default();
    let outer_line = context.source()[else_offset..if_offset]
        .lines()
        .next()
        .unwrap_or("");
    let outer_indent = &outer_line[..outer_line.len() - outer_line.trim_start().len()];
    let mut replacement = format!("{outer_indent}elsif {first}");
    let rest = expanded_lines.collect::<Vec<_>>();
    for line in rest.iter().take(rest.len().saturating_sub(1)) {
        replacement.push('\n');
        replacement.push_str(line);
    }
    context.replace(
        "Convert `if` nested inside `else` to `elsif`.",
        offense,
        else_offset..correction_end,
        replacement,
    );
    true
}
