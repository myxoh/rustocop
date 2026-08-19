use super::*;

pub(super) fn if_inside_else(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut reported = false;
    for pair in lines.windows(2) {
        let (else_offset, else_line) = pair[0];
        let (if_offset, if_line) = pair[1];
        if else_line.trim() != "else" || !if_line.trim_start().starts_with("if ") {
            continue;
        }
        let indent = if_line.len() - if_line.trim_start().len();
        let condition = if_line.trim_start()[3..].trim_end();
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
        let outer_indent = &else_line[..else_line.len() - else_line.trim_start().len()];
        let mut replacement = format!("{outer_indent}elsif {condition}");
        let nested_end = nested_end.expect("checked above");
        let mut nested_else = false;
        for (_, body_line) in lines.iter().take(nested_end).skip(nested_index + 1) {
            if body_line.trim_start().starts_with("else") {
                nested_else = true;
            }
            let dedented = if nested_else {
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
    correct_modifier_form(context, &lines);
}

fn correct_modifier_form(context: &mut CopContext<'_, '_>, lines: &[(usize, &str)]) {
    if context.config_bool("AllowIfModifier", false) {
        return;
    }
    for (else_index, (else_offset, else_line)) in lines.iter().enumerate() {
        if else_line.trim() != "else" {
            continue;
        }
        let Some((body_offset, body_line)) = lines[else_index + 1..]
            .iter()
            .find(|(_, line)| !line.trim_start().starts_with('#'))
        else {
            continue;
        };
        let Some(if_at) = body_line.find(" if ") else {
            continue;
        };
        if body_line[..if_at].trim().is_empty() {
            continue;
        }
        let condition = body_line[if_at + 4..]
            .split('#')
            .next()
            .unwrap_or_default()
            .trim();
        let body = body_line[..if_at].trim();
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
                    format!("{body_indent}  {body}"),
                ),
            ],
        );
    }
}
