use super::*;

define_cops! {
    MethodLength => "Metrics/MethodLength" => any_node(method_length),
    BlockLength => "Metrics/BlockLength" => node(as_block_node, block_length),
    AbcSize => "Metrics/AbcSize" => any_node(abc_size),
}

fn method_length(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (name, body, location) = if let Some(node) = node.as_def_node() {
        (
            node.name().as_slice().to_vec(),
            node.body(),
            node.location(),
        )
    } else if let Some(block) = node.as_block_node() {
        let Some(call) = context
            .nearest_call()
            .filter(|call| call_name(call) == b"define_method")
        else {
            return;
        };
        let name = first_argument(&call)
            .map(|argument| {
                node_source(context.source(), &argument)
                    .trim_start_matches(':')
                    .as_bytes()
                    .to_vec()
            })
            .unwrap_or_default();
        (name, block.body(), call.location())
    } else {
        return;
    };
    if context.policy().allows_method(&name) {
        return;
    }
    let maximum = context.config_usize("Max", 10);
    let count = body.map_or(0, |body| {
        let source = context.source_file().at(&body.location());
        let mut count = code_lines(source, false, context.config_bool("CountComments", false));
        if context
            .config_values("CountAsOne")
            .iter()
            .any(|value| value == "array")
        {
            count = count.saturating_sub(folded_extra_lines(source, '[', ']'));
        }
        if context
            .config_values("CountAsOne")
            .iter()
            .any(|value| value == "hash")
        {
            count = count.saturating_sub(folded_extra_lines(source, '{', '}'));
        }
        count
    });
    if count > maximum {
        context.report(
            format!("Method has too many lines. [{count}/{maximum}]"),
            location,
        );
    }
}

fn block_length(node: &ruby_prism::BlockNode<'_>, context: &mut CopContext<'_, '_>) {
    let owning_call = context.nearest_call();
    if owning_call.as_ref().is_some_and(|call| {
        matches!(call_name(call), b"define_method" | b"new")
            && (call_name(call) != b"new"
                || root_constant(call.receiver(), b"Class")
                || root_constant(call.receiver(), b"Module")
                || root_constant(call.receiver(), b"Struct")
                || root_constant(call.receiver(), b"Data"))
    }) {
        return;
    }
    if owning_call.as_ref().is_some_and(|call| {
        let method = String::from_utf8_lossy(call_name(call));
        let full_name = block_method_name(call, context.source());
        context.policy().allows_method(call_name(call))
            || ["AllowedMethods", "IgnoredMethods", "ExcludedMethods"]
                .iter()
                .flat_map(|key| context.config_values(key))
                .any(|allowed| allowed == method.as_ref() || allowed == &full_name)
            || ["AllowedPatterns", "IgnoredMethods"]
                .iter()
                .flat_map(|key| context.config_values(key))
                .any(|pattern| {
                    let pattern = pattern.trim_matches(['^', '$']);
                    method.contains(pattern) || full_name.contains(pattern)
                })
    }) {
        return;
    }
    let maximum = context.config_usize("Max", 25);
    let count = node.body().map_or(0, |body| {
        let source = context.source_file().at(&body.location());
        let mut count = code_lines(source, false, context.config_bool("CountComments", false));
        if context
            .config_values("CountAsOne")
            .iter()
            .any(|value| value == "array")
        {
            count = count.saturating_sub(folded_extra_lines(source, '[', ']'));
        }
        if context
            .config_values("CountAsOne")
            .iter()
            .any(|value| value == "hash")
        {
            count = count.saturating_sub(folded_extra_lines(source, '{', '}'));
        }
        count
    });
    if count > maximum {
        let offense = owning_call.map_or_else(
            || node.location().start_offset()..node.location().end_offset(),
            |call| call.location().start_offset()..call.location().end_offset(),
        );
        context.report(
            format!("Block has too many lines. [{count}/{maximum}]"),
            offense,
        );
    }
}

fn block_method_name(call: &CallNode<'_>, source: &str) -> String {
    let Some(message) = call.message_loc() else {
        return String::from_utf8_lossy(call_name(call)).into_owned();
    };
    let start = call.receiver().map_or_else(
        || message.start_offset(),
        |receiver| receiver.location().start_offset(),
    );
    source[start..message.end_offset()]
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn code_lines(source: &str, exclude_edges: bool, count_comments: bool) -> usize {
    let lines = source.lines().collect::<Vec<_>>();
    let slice = if exclude_edges && lines.len() >= 2 {
        &lines[1..lines.len() - 1]
    } else {
        lines.as_slice()
    };
    slice
        .iter()
        .filter(|line| {
            let line = line.trim();
            !line.is_empty() && (count_comments || !line.starts_with('#'))
        })
        .count()
}

fn folded_extra_lines(source: &str, open: char, close: char) -> usize {
    let Some(start) = source.find(open) else {
        return 0;
    };
    let Some(end_relative) = source[start..].find(close) else {
        return 0;
    };
    source[start..start + end_relative].matches('\n').count()
}

fn abc_size(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (name, body, location) = if let Some(definition) = node.as_def_node() {
        (
            definition.name().as_slice().to_vec(),
            definition.body(),
            definition.location(),
        )
    } else if let Some(block) = node.as_block_node() {
        let Some(call) = context
            .nearest_call()
            .filter(|call| call_name(call) == b"define_method")
        else {
            return;
        };
        let name = first_argument(&call)
            .map(|argument| {
                node_source(context.source(), &argument)
                    .trim_start_matches(':')
                    .as_bytes()
                    .to_vec()
            })
            .unwrap_or_default();
        (name, block.body(), call.location())
    } else {
        return;
    };
    if context.policy().allows_method(&name) {
        return;
    }
    let body = body
        .map(|body| context.source_file().at(&body.location()).to_string())
        .unwrap_or_default();
    let assignments = assignment_count(&body);
    let conditions = condition_count(&body);
    let branches = branch_count(
        &body,
        assignments,
        context.config_bool("CountRepeatedAttributes", true),
    );
    let score =
        ((assignments * assignments + branches * branches + conditions * conditions) as f64).sqrt();
    let maximum = context
        .config_value("Max")
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(17.0);
    if score <= maximum {
        return;
    }
    let name = String::from_utf8_lossy(&name);
    let score = metric_number(score);
    let maximum = metric_number(maximum);
    context.report(
        format!(
            "Assignment Branch Condition size for `{name}` is too high. [<{assignments}, {branches}, {conditions}> {score}/{maximum}]"
        ),
        location,
    );
}

fn metric_number(value: f64) -> String {
    let integer_digits = if value < 1.0 {
        1
    } else {
        value.log10().floor() as usize + 1
    };
    let precision = 4_usize.saturating_sub(integer_digits).min(2);
    let formatted = format!("{value:.precision$}");
    if formatted.contains('.') {
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        formatted
    }
}

fn assignment_count(source: &str) -> usize {
    let direct = source
        .match_indices('=')
        .filter(|(at, _)| {
            let before = source.as_bytes().get(at.saturating_sub(1)).copied();
            let after = source.as_bytes().get(at + 1).copied();
            !matches!(before, Some(b'=' | b'!' | b'<' | b'>')) && after != Some(b'=')
        })
        .count();
    let block_parameters = source
        .lines()
        .filter_map(|line| {
            line.split_once('|')
                .and_then(|(_, tail)| tail.split_once('|'))
        })
        .map(|(parameters, _)| {
            parameters
                .split(',')
                .filter(|parameter| !parameter.trim().is_empty())
                .count()
        })
        .sum::<usize>();
    direct + block_parameters
}

fn condition_count(source: &str) -> usize {
    let source = source
        .lines()
        .map(|line| line.split('#').next().unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n");
    [
        " if ", " unless ", "&&", "||", "==", " != ", " > ", " < ", "when ",
    ]
    .iter()
    .map(|needle| source.matches(needle).count())
    .sum::<usize>()
        + source.matches("&.").count()
        + source.matches(" do").count()
        + source
            .lines()
            .filter(|line| {
                let line = line.trim_start();
                line.starts_with("if ") || line.starts_with("unless ")
            })
            .count()
}

fn branch_count(source: &str, assignments: usize, count_repeated_attributes: bool) -> usize {
    let code = source
        .lines()
        .map(|line| line.split('#').next().unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n");
    let safe = code.matches("&.").count();
    let dots = code.matches('.').count().saturating_sub(safe);
    let indexes = code.matches('[').count() * 2;
    let mut bare = 0usize;
    for line in code.lines() {
        let line = line.trim();
        if count_repeated_attributes
            && line
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'?' | b'!'))
            && !matches!(
                line,
                "true"
                    | "false"
                    | "nil"
                    | "self"
                    | "end"
                    | "else"
                    | "break"
                    | "next"
                    | "redo"
                    | "retry"
            )
        {
            bare += 1;
        }
        if matches!(
            line.split_whitespace().next(),
            Some("p" | "puts" | "print" | "yield" | "raise")
        ) {
            bare += 1;
        }
        if let Some((before, after)) = line
            .split_once(" if ")
            .or_else(|| line.split_once(" unless "))
        {
            if !before.contains(['=', '.']) {
                bare += 1;
            }
            if after
                .trim()
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'?' | b'!'))
            {
                bare += 1;
            }
        }
    }
    let _ = assignments;
    safe * 2 + dots + indexes + bare
}
