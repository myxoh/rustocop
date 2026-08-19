use super::*;

define_cops! {
    IfInsideElse => "Style/IfInsideElse" => source(if_inside_else),
    MultilineTernaryOperator => "Style/MultilineTernaryOperator" => node(as_if_node, multiline_ternary_operator),
    CaseLikeIf => "Style/CaseLikeIf" => source(case_like_if),
}

fn case_like_if(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let minimum = context.config_usize("MinBranchesCount", 2);
    let mut index = 0usize;
    while index < lines.len() {
        let (start_offset, first_line) = lines[index];
        let first = first_line.trim_start();
        let Some(condition) = first.strip_prefix("if ") else {
            index += 1;
            continue;
        };
        if let Some((left, right)) = condition.split_once(" =~ ") {
            if !left.trim().starts_with('/') && !right.trim().starts_with('/') {
                index += 1;
                continue;
            }
        }
        if let Some((receiver, argument)) = condition.split_once(".match?(") {
            if !receiver.trim().starts_with('/')
                && !argument.trim_end_matches(')').trim().starts_with('/')
            {
                index += 1;
                continue;
            }
        }
        let Some((subject, value)) = case_comparison(condition) else {
            index += 1;
            continue;
        };
        let mut branches = vec![(index, value)];
        let mut end_index = None;
        let mut cursor = index + 1;
        while cursor < lines.len() {
            let line = lines[cursor].1.trim_start();
            if let Some(condition) = line.strip_prefix("elsif ") {
                let Some((candidate, value)) = case_comparison(condition) else {
                    break;
                };
                if candidate != subject {
                    break;
                }
                branches.push((cursor, value));
            } else if line.trim() == "end" {
                end_index = Some(cursor);
                break;
            }
            cursor += 1;
        }
        let Some(end_index) = end_index else {
            index += 1;
            continue;
        };
        if branches.len() < minimum {
            index = end_index + 1;
            continue;
        }
        let indent = &first_line[..first_line.len() - first.len()];
        let mut edits = Vec::new();
        for (branch, value) in &branches {
            let (offset, line) = lines[*branch];
            let replacement = if *branch == index {
                format!("{indent}case {subject}\n{indent}when {value}")
            } else {
                format!("{indent}when {value}")
            };
            edits.push((offset..offset + line.len(), replacement));
        }
        let end = lines[end_index].0 + lines[end_index].1.len();
        context.replace_many(
            "Convert `if-elsif` to `case-when`.",
            start_offset..end,
            edits,
        );
        index = end_index + 1;
    }
}

fn case_comparison(condition: &str) -> Option<(String, String)> {
    let condition = condition.split('#').next().unwrap_or(condition).trim();
    if condition.contains(" && ") {
        return None;
    }
    let condition = if condition.contains(" == ") {
        condition.trim_matches(['(', ')']).trim()
    } else {
        condition
    };
    if condition.contains(" || ") {
        let comparisons = condition
            .split(" || ")
            .map(case_comparison)
            .collect::<Option<Vec<_>>>()?;
        let subject = comparisons.first()?.0.clone();
        if comparisons.iter().any(|comparison| comparison.0 != subject) {
            return None;
        }
        return Some((
            subject,
            comparisons
                .into_iter()
                .map(|comparison| comparison.1)
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }
    if let Some((value, subject)) = condition.split_once(" === ") {
        return Some((subject.trim().to_string(), value.trim().to_string()));
    }
    if let Some((subject, value)) = condition.split_once(" == ") {
        let subject = subject.trim();
        let value = value.trim();
        if value.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
            && (value.len() == 1 || value.bytes().any(|byte| byte.is_ascii_lowercase()))
        {
            return None;
        }
        if case_literal(subject) && !case_literal(value) {
            return Some((value.to_string(), subject.to_string()));
        }
        return Some((subject.to_string(), value.to_string()));
    }
    if let Some((subject, class)) = condition.split_once(".is_a?(") {
        return Some((
            subject.trim().to_string(),
            class.trim_end_matches(')').trim().to_string(),
        ));
    }
    if let Some((receiver, argument)) = condition.split_once(".match?(") {
        let receiver = receiver.trim();
        let argument = argument.trim_end_matches(')').trim();
        return if receiver.starts_with('/') {
            Some((argument.to_string(), receiver.to_string()))
        } else if argument.starts_with('/') {
            Some((receiver.to_string(), argument.to_string()))
        } else {
            Some((argument.to_string(), receiver.to_string()))
        };
    }
    if let Some((range, argument)) = condition.split_once(".include?(") {
        if range.contains("..") {
            return Some((
                argument.trim_end_matches(')').trim().to_string(),
                range.trim().trim_matches(['(', ')']).to_string(),
            ));
        }
    }
    if let Some((left, right)) = condition.split_once(" =~ ") {
        let left = left.trim();
        let right = right.trim();
        if left.contains("(?<") {
            return None;
        }
        return if left.starts_with('/') {
            Some((right.to_string(), left.to_string()))
        } else {
            Some((left.to_string(), right.to_string()))
        };
    }
    None
}

fn case_literal(value: &str) -> bool {
    value.starts_with(['\'', '"', ':', '/', '[', '{'])
        || value.as_bytes().first().is_some_and(u8::is_ascii_digit)
        || matches!(value, "nil" | "true" | "false")
        || value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

fn if_inside_else(context: &mut CopContext<'_, '_>) {
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
    for (else_index, (else_offset, else_line)) in lines.iter().enumerate() {
        if context.config_bool("AllowIfModifier", false) {
            break;
        }
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

fn multiline_ternary_operator(node: &ruby_prism::IfNode<'_>, context: &mut CopContext<'_, '_>) {
    let location = node.location();
    let source = context.source_file().at(&location);
    if !source.contains('\n') || !source.contains('?') {
        return;
    }
    let Some(question) = source
        .find(" ?")
        .map(|at| at + 1)
        .or_else(|| source.find('?'))
    else {
        return;
    };
    let Some(colon) = ternary_colon(source, question) else {
        return;
    };
    if !(source[question..].contains('\n')
        || source[..question].contains('\n') && source[..question].contains("=="))
    {
        return;
    }
    let condition = source[..question].trim();
    let truthy = source[question + 1..colon].trim();
    let falsey = source[colon + 1..].trim();
    if condition.is_empty() || truthy.is_empty() || falsey.is_empty() {
        return;
    }
    let line_start = context.source_file().line_start(location.start_offset());
    let indentation = context.source()[line_start..location.start_offset()]
        .bytes()
        .take_while(u8::is_ascii_whitespace)
        .count();
    let indent = " ".repeat(indentation);
    let single_line = context.parent().is_some_and(|parent| {
        parent.as_return_node().is_some()
            || parent.as_break_node().is_some()
            || parent.as_next_node().is_some()
            || parent
                .as_call_node()
                .is_some_and(|call| !call_name(&call).ends_with(b"="))
    });
    let replacement = if single_line {
        format!("{condition} ? {truthy} : {falsey}")
    } else {
        let converted = convert_multiline_ternary(source)
            .unwrap_or_else(|| format!("if {condition}\n  {truthy}\nelse\n  {falsey}\nend"));
        converted
            .lines()
            .enumerate()
            .map(|(index, line)| {
                if index == 0 {
                    line.to_string()
                } else {
                    format!("{indent}{line}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let message = if single_line {
        "Avoid multi-line ternary operators, use single-line instead."
    } else {
        "Avoid multi-line ternary operators, use `if` or `unless` instead."
    };
    let nested = context.ancestors().iter().any(|ancestor| {
        ancestor
            .as_if_node()
            .is_some_and(|ancestor| context.source_file().at(&ancestor.location()).contains('?'))
    });
    if nested {
        context.replace_indirectly(message, &location, &location, replacement);
    } else {
        context.replace(message, &location, &location, replacement);
    }
}
