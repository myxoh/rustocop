use super::*;

define_cops! {
    MultipleComparison => "Style/MultipleComparison" => source(multiple_comparison),
    ExplicitBlockArgument => "Style/ExplicitBlockArgument" => source(explicit_block_argument),
}

fn multiple_comparison(context: &mut CopContext<'_, '_>) {
    let threshold = context.config_usize("ComparisonsThreshold", 2);
    for (offset, line) in context.source_file().lines() {
        let code = line.split('#').next().unwrap_or(line);
        for operator in [" || ", " or "] {
            let comparisons = code.split(operator).collect::<Vec<_>>();
            if comparisons.len() < threshold {
                continue;
            }
            let mut pairs = Vec::new();
            let mut first = None;
            let mut last = None;
            for comparison in &comparisons {
                let trimmed = comparison.trim();
                let Some((left, right)) = trimmed.split_once(" == ") else {
                    pairs.clear();
                    break;
                };
                let left = left.split_whitespace().next_back().unwrap_or(left);
                pairs.push((left, right.trim().trim_end_matches([')', ';'])));
                let at = line.find(trimmed).unwrap_or(0);
                first.get_or_insert(at);
                last = Some(at + trimmed.len());
            }
            if pairs.len() < threshold {
                continue;
            }
            let identifier = |value: &str| {
                value
                    .bytes()
                    .next()
                    .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
                    && value
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            };
            if pairs
                .iter()
                .all(|pair| identifier(pair.0) && identifier(pair.1))
            {
                continue;
            }
            let variable = [pairs[0].0, pairs[0].1].into_iter().find(|candidate| {
                pairs
                    .iter()
                    .all(|pair| pair.0 == *candidate || pair.1 == *candidate)
            });
            let Some(variable) = variable else {
                continue;
            };
            if variable.contains(['[', ']']) && !context.config_bool("AllowMethodComparison", true)
            {
                continue;
            }
            if variable.contains("&.") && !context.config_bool("AllowMethodComparison", true) {
                continue;
            }
            if variable.starts_with(['\'', '"', ':'])
                || variable
                    .bytes()
                    .next()
                    .is_some_and(|byte| byte.is_ascii_digit())
            {
                continue;
            }
            let allow_methods = context.config_bool("AllowMethodComparison", true);
            let literal = |value: &str| {
                value.starts_with(['\'', '"', ':'])
                    || value.as_bytes().first().is_some_and(u8::is_ascii_digit)
            };
            let effective_pairs = pairs
                .iter()
                .filter(|pair| {
                    let other = if pair.0 == variable { pair.1 } else { pair.0 };
                    !allow_methods || literal(other)
                })
                .collect::<Vec<_>>();
            if effective_pairs.len() < threshold {
                continue;
            }
            let values = effective_pairs
                .iter()
                .map(|pair| if pair.0 == variable { pair.1 } else { pair.0 })
                .collect::<Vec<_>>();
            if allow_methods
                && values
                    .iter()
                    .any(|value| value.contains('.') || value.contains("&."))
            {
                continue;
            }
            let first_pair = effective_pairs[0];
            let candidate = format!("{} == {}", first_pair.0, first_pair.1);
            let first = line.find(&candidate).unwrap_or(first.unwrap_or(0));
            let start = offset
                + line[first..]
                    .find(&candidate)
                    .map_or(first, |at| first + at);
            let end = offset + last.unwrap_or(0);
            context.replace(
                "Avoid comparing a variable with multiple items in a conditional, use `Array#include?` instead.",
                start..end,
                start..end,
                format!("[{}].include?({variable})", values.join(", ")),
            );
            break;
        }
    }
}

fn explicit_block_argument(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for (def_offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("def ") {
            continue;
        }
        let Some(relative_end) = source[def_offset..].find("\nend") else {
            continue;
        };
        let method_end = def_offset + relative_end + 4;
        let body = &source[def_offset..method_end];
        if body
            .lines()
            .skip(1)
            .any(|nested| nested.trim_start().starts_with("def "))
        {
            continue;
        }
        if explicit_inline_blocks(context, def_offset, line, method_end) {
            continue;
        }
        if let Some(block_relative) = body.find(" { |") {
            let block_start = def_offset + block_relative;
            let Some(pipe_relative) = source[block_start + 4..].find('|') else {
                continue;
            };
            let pipe = block_start + 4 + pipe_relative;
            let args = source[block_start + 4..pipe].trim();
            let Some(close_relative) = source[pipe + 1..method_end].find('}') else {
                continue;
            };
            let close = pipe + 1 + close_relative;
            if source[pipe + 1..close].trim() != format!("yield {args}") {
                continue;
            }
            let call_start = source[..block_start]
                .rfind('\n')
                .map_or(def_offset, |at| at + 1);
            let indent = &source[call_start
                ..call_start + source[call_start..].len()
                    - source[call_start..].trim_start().len()];
            let call = source[call_start..block_start].trim();
            let replacement_call = if let Some(call) = call.strip_suffix(')') {
                format!("{}{call}, &block)", indent)
            } else {
                format!("{indent}{call}(&block)")
            };
            let signature_insert = if let Some(close) = line.rfind(')') {
                def_offset + close
            } else {
                def_offset + line.len()
            };
            let signature_text = if line.contains('(') {
                ", &block"
            } else {
                "(&block)"
            };
            context.replace_many(
                "Consider using explicit block argument in the surrounding method's signature over `yield`.",
                call_start + indent.len()..close + 1,
                vec![
                    (call_start..close + 1, replacement_call),
                    (signature_insert..signature_insert, signature_text.to_string()),
                ],
            );
            continue;
        }
        let Some(block_start_relative) = body.find(" do |") else {
            continue;
        };
        let block_start = def_offset + block_start_relative;
        let Some(args_end_relative) = source[block_start + 5..].find('|') else {
            continue;
        };
        let args_end = block_start + 5 + args_end_relative;
        let args = source[block_start + 5..args_end].trim();
        let Some(block_end_relative) = source[args_end + 1..method_end].find("\n  end") else {
            continue;
        };
        let block_end = args_end + 1 + block_end_relative;
        let block_body = source[args_end + 1..block_end].trim();
        if block_body != format!("yield {args}") {
            continue;
        }
        let call_start = source[..block_start]
            .rfind('\n')
            .map_or(def_offset, |at| at + 1);
        let indent_len = source[call_start..].len() - source[call_start..].trim_start().len();
        let indent = &source[call_start..call_start + indent_len];
        let call = source[call_start..block_start].trim();
        let replacement_call = format!("{indent}{call}(&block)");
        let signature_insert = if let Some(close) = line.rfind(')') {
            def_offset + close
        } else {
            def_offset + line.len()
        };
        let signature_text = if line.contains('(') {
            ", &block"
        } else {
            "(&block)"
        };
        context.replace_many(
            "Consider using explicit block argument in the surrounding method's signature over `yield`.",
            call_start + indent_len..block_end + 6,
            vec![
                (call_start..block_end + 6, replacement_call),
                (signature_insert..signature_insert, signature_text.to_string()),
            ],
        );
    }
}

fn explicit_inline_blocks(
    context: &mut CopContext<'_, '_>,
    def_offset: usize,
    signature: &str,
    method_end: usize,
) -> bool {
    let existing_block = signature
        .split('&')
        .nth(1)
        .map(|tail| {
            tail.bytes()
                .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                .map(char::from)
                .collect::<String>()
        })
        .filter(|name| !name.is_empty());
    let block_name = existing_block.as_deref().unwrap_or("block");
    let mut candidates = Vec::<(std::ops::Range<usize>, String)>::new();
    for (line_offset, line) in context.source_file().lines() {
        if line_offset <= def_offset || line_offset >= method_end {
            continue;
        }
        let Some(open) = line.find(" {") else {
            continue;
        };
        let Some(close) = line.rfind('}') else {
            continue;
        };
        let block = line[open + 2..close].trim();
        let forwards = if let Some(parameters) = block.strip_prefix('|') {
            let Some((parameters, body)) = parameters.split_once('|') else {
                continue;
            };
            body.trim() == format!("yield {}", parameters.trim())
        } else {
            block == "yield"
        };
        if !forwards {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let call = line[indent..open].trim_end();
        let replacement = if call == "super" && signature.contains('(') {
            let parameters = signature
                .split_once('(')
                .and_then(|(_, rest)| rest.rsplit_once(')'))
                .map(|(parameters, _)| {
                    parameters
                        .split(',')
                        .map(str::trim)
                        .filter(|parameter| !parameter.is_empty())
                        .map(|parameter| {
                            parameter
                                .split_once('=')
                                .map_or(parameter, |(name, _)| name.trim())
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            let separator = if parameters.is_empty() { "" } else { ", " };
            format!("super({parameters}{separator}&{block_name})")
        } else if let Some(call) = call.strip_suffix(')') {
            let inner = call.trim_end_matches(',');
            let separator = if inner.ends_with('(') { "" } else { ", " };
            format!("{inner}{separator}&{block_name})")
        } else {
            format!("{}(&{block_name})", call.trim_end_matches(','))
        };
        candidates.push((line_offset + indent..line_offset + close + 1, replacement));
    }
    if candidates.is_empty() {
        return false;
    }
    let signature_insert = if let Some(close) = signature.rfind(')') {
        def_offset + close
    } else {
        def_offset + signature.len()
    };
    let signature_text = if signature.trim_end().ends_with("()") {
        "&block"
    } else if signature.contains('(') {
        ", &block"
    } else {
        "(&block)"
    };
    let mut edits = vec![candidates[0].clone()];
    if existing_block.is_none() {
        edits.push((
            signature_insert..signature_insert,
            signature_text.to_string(),
        ));
    }
    let message = "Consider using explicit block argument in the surrounding method's signature over `yield`.";
    context.replace_many(message, candidates[0].0.clone(), edits);
    for (range, replacement) in candidates.iter().skip(1) {
        context.replace(message, range.clone(), range.clone(), replacement);
    }
    true
}
