use super::*;

mod elsif_conversion;
use elsif_conversion::*;

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
